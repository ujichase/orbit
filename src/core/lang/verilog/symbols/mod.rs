//
//  Copyright (C) 2022-2024  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use std::iter::Peekable;

use super::super::sv::token::operator::Operator;
use super::error::VerilogError;
use super::interface::{self, ParamList};
use super::token::identifier::Identifier;
use super::token::tokenizer::VerilogTokenizer;
use crate::core::lang::lexer::{Position, Token};
use crate::core::lang::parser::{Parse, Symbol};
use crate::core::lang::reference::{CompoundIdentifier, RefSet};
use crate::core::lang::sv::symbols::SystemVerilogSymbol;
use crate::core::lang::sv::token::keyword::Keyword;
use crate::core::lang::sv::token::token::SystemVerilogToken;
use crate::core::lang::verilog::interface::{Port, PortList};
use crate::core::lang::verilog::token::token::VerilogToken;
use std::str::FromStr;

pub mod config;
pub mod module;
pub mod primitive;

use config::Config;
use module::Module;
use primitive::Primitive;

pub type Statement = Vec<Token<SystemVerilogToken>>;

fn into_tokens(stmt: Statement) -> Vec<SystemVerilogToken> {
    stmt.into_iter().map(|t| t.take()).collect()
}

fn statement_to_string(stmt: &Statement) -> String {
    stmt.iter().fold(String::new(), |mut acc, x| {
        acc.push_str(&x.as_type().to_string());
        acc.push(' ');
        acc
    })
}

/// Design elements of the Verilog Language.
#[derive(Debug, PartialEq)]
pub enum VerilogSymbol {
    Module(Module),
    Config(Config),
    Primitive(Primitive),
}

impl VerilogSymbol {
    pub fn as_name(&self) -> Option<&Identifier> {
        match &self {
            Self::Module(m) => Some(m.get_name()),
            Self::Config(c) => Some(c.get_name()),
            Self::Primitive(p) => Some(p.get_name()),
        }
    }

    pub fn get_position(&self) -> &Position {
        match self {
            Self::Module(m) => m.get_position(),
            Self::Config(c) => c.get_position(),
            Self::Primitive(p) => p.get_position(),
        }
    }

    pub fn as_module(&self) -> Option<&Module> {
        match &self {
            Self::Module(m) => match m.get_name().is_nonuser_name() {
                true => None,
                false => Some(m),
            },
            _ => None,
        }
    }

    pub fn get_refs(&self) -> &RefSet {
        match &self {
            Self::Module(m) => m.get_refs(),
            Self::Config(c) => c.get_refs(),
            Self::Primitive(p) => p.get_refs(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct VerilogParser {
    symbols: Vec<Symbol<VerilogSymbol>>,
}

impl VerilogParser {
    /// Quietly ignores any errors and returns the list of symbols.
    pub fn read_lazy(s: &str) -> Self {
        let symbols = VerilogParser::parse(VerilogTokenizer::from_source_code(&s).into_tokens());
        Self {
            symbols: symbols
                .into_iter()
                .filter_map(|f| if f.is_ok() { Some(f.unwrap()) } else { None })
                .collect(),
        }
    }

    /// Reports an error if one is discovered in the list of symbols or in the tokenizing.
    pub fn read(s: &str) -> Result<Self, VerilogError> {
        let symbols = VerilogParser::parse(
            VerilogTokenizer::from_str(&s)?
                .into_tokens()
                .into_iter()
                .filter(|s| s.as_type().is_comment() == false)
                .collect(),
        );
        let result: Result<Vec<Symbol<VerilogSymbol>>, VerilogError> =
            symbols.into_iter().collect();
        Ok(Self { symbols: result? })
    }

    pub fn into_symbols(self) -> Vec<VerilogSymbol> {
        self.symbols.into_iter().map(|f| f.take()).collect()
    }
}

impl Parse<VerilogToken> for VerilogParser {
    type SymbolType = VerilogSymbol;
    type SymbolError = VerilogError;

    fn parse(
        tokens: Vec<Token<VerilogToken>>,
    ) -> Vec<Result<Symbol<Self::SymbolType>, Self::SymbolError>> {
        // up cast the tokens into SystemVerilog tokens (since SV is a superset)
        let tokens: Vec<Token<SystemVerilogToken>> = tokens
            .into_iter()
            .map(|m| {
                let (pos, tkn) = m.decouple();
                Token::new(SystemVerilogToken::from(tkn), pos)
            })
            .collect();

        let mut symbols = Vec::new();
        let mut tokens = tokens.into_iter().peekable();

        while let Some(t) = tokens.next() {
            // take attribute and ignore if okay
            if t.as_type().check_delimiter(&Operator::AttrL) {
                match VerilogSymbol::parse_attr(&mut tokens, t.into_position()) {
                    Ok(_) => (),
                    Err(e) => symbols.push(Err(e)),
                }
            }
            // create module symbol
            else if t.as_type().check_keyword(&Keyword::Module)
                || t.as_type().check_keyword(&Keyword::Macromodule)
            {
                symbols.push(
                    match VerilogSymbol::parse_module(&mut tokens, t.into_position()) {
                        Ok(module) => Ok(Symbol::new(module)),
                        Err(e) => Err(e),
                    },
                );
            // create primitive symbol
            } else if t.as_type().check_keyword(&Keyword::Primitive) {
                symbols.push(
                    match VerilogSymbol::parse_primitive(&mut tokens, t.into_position()) {
                        Ok(prim) => Ok(Symbol::new(prim)),
                        Err(e) => Err(e),
                    },
                );
            // create config symbol
            } else if t.as_type().check_keyword(&Keyword::Config) {
                symbols.push(
                    match VerilogSymbol::parse_config(&mut tokens, t.into_position()) {
                        Ok(config) => Ok(Symbol::new(config)),
                        Err(e) => Err(e),
                    },
                );
            // skip any potential illegal/unknown tokens at global scale
            } else if t.as_type().is_eof() == false {
                // println!("{:?}", t);
                // symbols.push(Err(VerilogError::Vague))
                continue;
            }
        }
        // println!("{:#?}", symbols);
        symbols
    }
}

impl VerilogSymbol {
    /// Parses an `Config` design element from the config's identifier to
    /// the END closing statement.
    fn parse_config<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<VerilogSymbol, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(VerilogSymbol::Config(Config::from_tokens(tokens, pos)?))
    }

    /// Parses an `Primitive` design element from the primitive's identifier to
    /// the END closing statement.
    fn parse_primitive<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<VerilogSymbol, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(VerilogSymbol::Primitive(Primitive::from_tokens(
            tokens, pos,
        )?))
    }

    fn parse_assignment<I>(
        tokens: &mut Peekable<I>,
        take_separator: bool,
    ) -> Result<Statement, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        let mut stmt = Vec::new();
        // keep taking tokens until the closing attribute
        while let Some(t) = tokens.peek() {
            if t.as_ref().check_delimiter(&Operator::Terminator)
                || t.as_ref().check_delimiter(&Operator::Comma)
                || t.as_ref().check_delimiter(&Operator::ParenR)
            {
                // do not take the ';' operator or ',' operator unless told to
                if take_separator == true {
                    tokens.next();
                }
                break;
            } else if t.as_ref().is_eof() == true {
                tokens.next();
                // expecting closing attribute operator
                return Err(VerilogError::ExpectingOperator(Operator::Terminator));
            // parse nested parentheses
            } else if t.as_ref().check_delimiter(&Operator::ParenL) {
                let t = tokens.next().unwrap();
                stmt.extend(Self::parse_until_operator(tokens, t, Operator::ParenR)?);
            } else {
                let t = tokens.next().unwrap();
                stmt.push(t);
            }
        }
        Ok(stmt)
    }

    /// Parses evenly until finding the balancing operator `end_op` to counter-act all
    /// of the equivalent `beg_t` operators.
    ///
    /// This function's last token to consume is the `end_op`, if it exists in balance with
    /// `beg_t`.
    pub fn parse_until_operator<I>(
        tokens: &mut Peekable<I>,
        beg_t: Token<SystemVerilogToken>,
        end_op: Operator,
    ) -> Result<Statement, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        let mut counter = 0;
        let mut stmt = vec![beg_t];
        let beg_op = stmt
            .first()
            .unwrap()
            .as_type()
            .as_delimiter()
            .unwrap()
            .clone();
        // keep taking tokens until the closing attribute
        while let Some(t) = tokens.next() {
            if t.as_ref().check_delimiter(&end_op) == true {
                stmt.push(t);
                if counter == 0 {
                    break;
                }
                counter -= 1;
            } else if t.as_ref().check_delimiter(&beg_op) {
                counter += 1;
                stmt.push(t);
            } else if t.as_ref().is_eof() == true {
                // expecting closing attribute operator
                if counter > 1 {
                    return Err(VerilogError::ExpectingOperator(Operator::ParenR));
                } else if counter < 0 {
                    return Err(VerilogError::ExpectingOperator(Operator::ParenL));
                } else {
                    return Err(VerilogError::ExpectingOperator(end_op));
                }
            } else {
                stmt.push(t);
            }
        }
        Ok(stmt)
    }

    fn parse_attr<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Statement, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        let mut stmt = vec![Token::new(
            SystemVerilogToken::Operator(Operator::AttrL),
            pos,
        )];
        // keep taking tokens until the closing attribute
        while let Some(t) = tokens.next() {
            if t.as_ref().check_delimiter(&Operator::AttrR) == true {
                stmt.push(t);
                break;
            } else if stmt.len() == 1 && t.as_ref().check_delimiter(&Operator::ParenR) == true {
                stmt.push(t);
                break;
            } else if t.as_ref().is_eof() == true {
                // expecting closing attribute operator
                return Err(VerilogError::ExpectingOperator(Operator::AttrR));
            }
            stmt.push(t);
        }
        Ok(stmt)
    }

    /// Parses an `Entity` primary design unit from the entity's identifier to
    /// the END closing statement.
    fn parse_module<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<VerilogSymbol, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(VerilogSymbol::Module(Module::from_tokens(
            tokens, pos, "verilog",
        )?))
    }

    fn is_timeunits_declaration(kw: Option<&Keyword>) -> bool {
        match kw {
            Some(kw) => match kw {
                Keyword::Timeunit | Keyword::Timeprecision => true,
                _ => false,
            },
            None => false,
        }
    }

    /// Parses a systemverilog-style "module" declaration, which can include a parameter list, and port list.
    ///
    /// It assumes the first token to consume is the start of one of these lists ('#' or '('), or is just the terminator ';'.
    /// The last token to be consumed by this function is the ';' delimiter.
    ///
    /// Also can handle and discard a timeunits declaration
    pub fn parse_module_declaration<I>(
        tokens: &mut Peekable<I>,
    ) -> Result<(ParamList, PortList, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        let mut param_list = ParamList::new();
        let mut port_list = PortList::new();
        let mut refs = RefSet::new();

        // check if there are parameters
        while let Some(t) = tokens.next() {
            if t.as_ref().is_eof() == true {
                return Err(VerilogError::ModDecIncomplete);
            // parse parameter list (optional)
            } else if t.as_ref().check_delimiter(&Operator::Pound) == true {
                let t_next = tokens.next().unwrap();
                if t_next.as_ref().check_delimiter(&Operator::ParenL) == true {
                    // parse parameter list
                    let (params, param_refs) =
                        Self::parse_module_param_list(tokens, t_next.locate().line())?;
                    param_list.extend(params);
                    refs.extend(param_refs);
                } else {
                    return Err(VerilogError::ExpectingOperator(Operator::ParenL));
                }
            // parse port list (optional?)
            } else if t.as_ref().check_delimiter(&Operator::ParenL) == true {
                let (ports, port_refs) = Self::parse_module_port_list(tokens, t.locate().line())?;
                port_list.extend(ports);
                refs.extend(port_refs);
            // handle the timeunits declaration (optional)
            } else if Self::is_timeunits_declaration(t.as_ref().as_keyword()) == true {
                // take all until a terminator
                if let Some(stmt) = Self::into_next_statement(t, tokens)? {
                    Self::handle_statement(
                        stmt,
                        Some(&mut param_list),
                        Some(&mut port_list),
                        &mut refs,
                        None,
                    )?;
                }
            // take the lifetime and continue
            } else if t.as_ref().check_keyword(&Keyword::Automatic)
                || t.as_ref().check_keyword(&Keyword::Static)
            {
                continue;
            // stop parsing the declaration
            } else if t.as_ref().check_delimiter(&Operator::Terminator) == true {
                break;
            }
        }

        Ok((param_list, port_list, refs))
    }

    pub fn parse_module_architecture<I>(
        tokens: &mut Peekable<I>,
        mut params: &mut ParamList,
        mut ports: &mut PortList,
    ) -> Result<(RefSet, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        let mut refs = RefSet::new();
        let mut deps = RefSet::new();

        while let Some(t) = tokens.next() {
            // expecting `endmodule`
            if t.as_ref().is_eof() == true {
                return Err(VerilogError::ExpectingKeyword(Keyword::Endmodule));
            // exit from the module architecture
            } else if t.as_ref().check_keyword(&Keyword::Endmodule) == true {
                break;
            } else if let Some(stmt) = Self::into_next_statement(t, tokens)? {
                // println!("[arch]: {}", statement_to_string(&stmt));
                Self::handle_statement(
                    stmt,
                    Some(&mut params),
                    Some(&mut ports),
                    &mut refs,
                    Some(&mut deps),
                )?;
            }
        }
        Ok((refs, deps))
    }

    pub fn into_next_statement<I>(
        init: Token<SystemVerilogToken>,
        tokens: &mut Peekable<I>,
    ) -> Result<Option<Statement>, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        let mut stmt = Statement::new();
        stmt.push(init);

        let mut now_line = None;
        loop {
            // review the last token we have added to the current statement
            let t = stmt.last().unwrap();
            let next_line = t.locate().line();

            // whoops... we should not have ran out of tokens here!
            if t.as_ref().is_eof() == true {
                return Err(VerilogError::Vague);
            // finish this statement
            } else if Self::is_statement_separator(t.as_type()) {
                let has_code_label = if let Some(t_next) = tokens.peek() {
                    // take the optional code segment's label
                    t_next.as_type().check_delimiter(&Operator::Colon)
                } else {
                    false
                };
                // take the `:` `label`
                if has_code_label == true {
                    stmt.push(tokens.next().unwrap());
                    stmt.push(tokens.next().unwrap());
                }
                break;
            // take a parentheses
            } else if Self::is_start_to_parentheses_statement(t.as_type()) {
                // expecting '('
                let opening_p = tokens.next().unwrap();
                if opening_p.as_ref().check_delimiter(&Operator::ParenL) {
                    // take the parentheses
                    stmt.extend(Self::parse_until_operator(
                        tokens,
                        opening_p,
                        Operator::ParenR,
                    )?);
                }
            // take everything in the parentheses
            } else if t.as_ref().check_delimiter(&Operator::ParenL) == true {
                let opening_p = stmt.pop().unwrap();
                stmt.extend(Self::parse_until_operator(
                    tokens,
                    opening_p,
                    Operator::ParenR,
                )?);
            // take all symbols until new line when handling a new directive on a new line
            } else if (now_line.is_none() || next_line > now_line.unwrap())
                && t.as_ref().is_directive() == true
            {
                let dir = stmt.pop().unwrap();
                let directive_stuff = Self::parse_compiler_directive_statement(tokens, dir)?;
                stmt.extend(directive_stuff);
                // println!("directive: {}", statement_to_string(&stmt));
                return Ok(Some(stmt));
            }

            // push a new token onto the statment
            if let Some(t_next) = tokens.next() {
                stmt.push(t_next);
            } else {
                break;
            }
            now_line = Some(next_line);
        }
        Ok(Some(stmt))
    }

    /// Assumes the last token consumed was a compiler directive at the beginning of a new line.
    pub fn parse_compiler_directive_statement<I>(
        tokens: &mut Peekable<I>,
        init: Token<SystemVerilogToken>,
    ) -> Result<Statement, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        let mut stmt = Statement::new();

        let mut next_line = init.locate().line();
        stmt.push(init);

        while let Some(t_next) = tokens.peek() {
            if t_next.locate().line() > next_line
                && stmt.last().unwrap().as_type()
                    != &SystemVerilogToken::Identifier(Identifier::Escaped(String::new()))
            {
                break;
            } else {
                // println!("[directive]: {:?}", t_next);
                next_line = t_next.locate().line();
                stmt.push(tokens.next().unwrap());
            }
        }
        Ok(stmt)
    }

    pub fn handle_statement(
        stmt: Statement,
        params: Option<&mut ParamList>,
        ports: Option<&mut PortList>,
        refs: &mut RefSet,
        deps: Option<&mut RefSet>,
    ) -> Result<(), VerilogError> {
        if stmt.is_empty() == true {
            return Ok(());
        }

        // println!("{}", statement_to_string(&stmt));

        // try as import statement
        if stmt
            .first()
            .unwrap()
            .as_type()
            .check_keyword(&Keyword::Import)
        {
            let mut tokens = stmt.into_iter().skip(1).peekable();
            let i_refs = SystemVerilogSymbol::parse_import_statement(&mut tokens)?;
            refs.extend(i_refs);
            return Ok(());
        }

        // update references that may appear in the statement
        if let Some(s_refs) = SystemVerilogSymbol::extract_refs_from_statement(&stmt) {
            refs.extend(s_refs);
        }

        // try as a module instantiation
        if let Some((dep, is_valid_mod)) = Self::as_module_instance(&stmt) {
            // println!("detected dependency! {}", dep);
            if is_valid_mod == true && dep.is_system_name() == false {
                if let Some(deps) = deps {
                    deps.insert(CompoundIdentifier::new_minimal_verilog(dep.clone()));
                }
            }
            refs.insert(CompoundIdentifier::new_minimal_verilog(dep.clone()));
            return Ok(());
        }

        // try as a port
        if let Some(mut ports) = ports {
            if let Some(def_ports) = Self::as_port_definition(&stmt, &ports) {
                def_ports
                    .into_iter()
                    .for_each(|p| interface::update_port_list(&mut ports, p, false));
            }
            return Ok(());
        }

        // try as a paramater
        if let Some(mut params) = params {
            if let Some(def_params) = Self::as_param_definition(&stmt, &params) {
                def_params
                    .into_iter()
                    .for_each(|p| interface::update_port_list(&mut params, p, false));
            }
            return Ok(());
        }

        // reset the statement
        Ok(())
    }

    fn as_port_definition(stmt: &Statement, ports: &PortList) -> Option<PortList> {
        // println!("as port? {}", statement_to_string(&stmt));
        let mut tokens = stmt.clone().into_iter().peekable();
        // verify the start token is valid
        match tokens.peek()?.as_type() {
            SystemVerilogToken::Identifier(name) => match interface::does_exist(&ports, name) {
                true => return None,
                false => (),
            },
            SystemVerilogToken::Keyword(kw) => match Port::is_port_direction(Some(kw)) {
                true => (),
                false => return None,
            },
            _ => return None,
        }
        match Self::parse_module_port_list(&mut tokens, 0) {
            Ok((decl_ports, _)) => Some(decl_ports),
            Err(_) => None,
        }
    }

    fn as_param_definition(stmt: &Statement, params: &ParamList) -> Option<ParamList> {
        // println!("as param? {}", statement_to_string(&stmt));
        let mut tokens = stmt.clone().into_iter().peekable();
        // verify the start token is valid
        match tokens.peek()?.as_type() {
            SystemVerilogToken::Identifier(name) => match interface::does_exist(&params, name) {
                true => return None,
                false => (),
            },
            SystemVerilogToken::Keyword(kw) => match kw == &Keyword::Parameter {
                true => (),
                false => return None,
            },
            _ => return None,
        }
        match Self::parse_module_param_list(&mut tokens, 0) {
            Ok((decl_params, _)) => Some(decl_params),
            Err(_) => None,
        }
    }

    fn is_valid_net_type(op: Option<&Keyword>) -> bool {
        let op = match op {
            Some(op) => op,
            None => return false,
        };
        match op {
            // supply0 | supply1 | tri | triand | trior | tri0 | tri1 | uwire | wire | wand | wor
            Keyword::Wire
            | Keyword::Reg
            | Keyword::Supply0
            | Keyword::Supply1
            | Keyword::Tri
            | Keyword::Triand
            | Keyword::Trior
            | Keyword::Tri0
            | Keyword::Tri1
            | Keyword::Uwire
            | Keyword::Wand
            | Keyword::Wor => true,
            _ => false,
        }
    }

    fn is_valid_data_type(tkn: &SystemVerilogToken) -> bool {
        match tkn.as_identifier().is_some() {
            true => true,
            false => match tkn.as_keyword() {
                Some(kw) => match kw {
                    Keyword::Integer
                    | Keyword::Real
                    | Keyword::Time
                    | Keyword::Realtime
                    | Keyword::Logic
                    | Keyword::Bit
                    | Keyword::Byte
                    | Keyword::Shortint
                    | Keyword::Int
                    | Keyword::Longint
                    | Keyword::String
                    | Keyword::Shortreal => true,
                    _ => false,
                },
                None => false,
            },
        }
    }

    /// Returns the name of the module that is being instantiated in this statement, if
    /// one exists.
    fn as_module_instance(stmt: &Statement) -> Option<(&Identifier, bool)> {
        let mod_name = stmt.first()?.as_ref().as_identifier()?;
        // are there parameters defined
        let mut stmt_iter = stmt.iter().skip(1);

        let mut state = 0;
        let mut counter = 0;
        let mut came_from_param_token = false;
        let mut has_port_decl = false;
        let mut has_port_body = false;
        while let Some(t) = stmt_iter.next() {
            // println!("{}", t.as_ref().to_string());
            // take the parameters
            match state {
                // take either name or parameters
                0 => {
                    if t.as_ref().check_delimiter(&Operator::Pound) {
                        came_from_param_token = true;
                        state = 1;
                    } else if t.as_ref().as_identifier().is_some() {
                        came_from_param_token = false;
                        state = 1;
                    } else if t.as_ref().check_delimiter(&Operator::Comma) {
                        state = 0;
                    } else if t.as_ref().check_delimiter(&Operator::Terminator) {
                        break;
                    } else {
                        state = -1;
                    }
                }
                // enter parameters or ports listings
                1 => {
                    // take port/parameter list
                    if t.as_ref().check_delimiter(&Operator::ParenL) {
                        counter = 0;
                        state = 3;
                        if came_from_param_token == false {
                            has_port_decl = true;
                        }
                    // take range specification
                    } else if t.as_ref().check_delimiter(&Operator::BrackL) {
                        counter = 0;
                        state = 4;
                    } else if t.as_ref().check_delimiter(&Operator::Terminator) {
                        break;
                    } else if t.as_ref().check_delimiter(&Operator::Comma) {
                        state = 0;
                    } else {
                        state = -1;
                    }
                }
                // take until closing parenthesis
                3 => {
                    if t.as_ref().check_delimiter(&Operator::ParenL) {
                        counter += 1;
                    } else if t.as_ref().check_delimiter(&Operator::ParenR) {
                        if counter == 0 {
                            state = 0;
                        } else {
                            counter -= 1;
                        }
                    } else if has_port_decl == true {
                        has_port_body = true;
                    }
                }
                // take until closing bracket
                4 => {
                    if t.as_ref().check_delimiter(&Operator::BrackL) {
                        counter += 1;
                    } else if t.as_ref().check_delimiter(&Operator::BrackR) {
                        if counter == 0 {
                            // go to state 1 next
                            came_from_param_token = false;
                            state = 1;
                        } else {
                            counter -= 1;
                        }
                    }
                }
                _ => break,
            }
        }
        match state >= 0 && counter == 0 {
            true => Some((mod_name, has_port_body)),
            false => None,
        }
    }

    /// Checks if this is special token to take a statement using parentheses
    fn is_start_to_parentheses_statement(t: &SystemVerilogToken) -> bool {
        match t {
            SystemVerilogToken::Keyword(k) => match k {
                Keyword::If
                | Keyword::For
                | Keyword::Casex
                | Keyword::Casez
                | Keyword::While
                | Keyword::Repeat
                | Keyword::Case => true,
                _ => false,
            },
            SystemVerilogToken::Operator(o) => match o {
                Operator::At => true,
                _ => false,
            },
            _ => false,
        }
    }

    fn is_statement_separator(t: &SystemVerilogToken) -> bool {
        match t {
            SystemVerilogToken::Keyword(k) => match k {
                Keyword::Initial
                | Keyword::Begin
                | Keyword::End
                | Keyword::Else
                | Keyword::Join
                | Keyword::JoinAny
                | Keyword::JoinNone
                | Keyword::Fork
                | Keyword::Forkjoin
                | Keyword::Endconfig
                | Keyword::Endfunction
                | Keyword::Endgenerate
                | Keyword::Endmodule
                | Keyword::Endprimitive
                | Keyword::Endclocking
                | Keyword::Endinterface
                | Keyword::Endchecker
                | Keyword::Endsequence
                | Keyword::Endspecify
                | Keyword::Endclass
                | Keyword::Endgroup
                | Keyword::Endproperty
                | Keyword::Endprogram
                | Keyword::Endtable
                | Keyword::Endtask
                | Keyword::Endcase => true,
                _ => false,
            },
            SystemVerilogToken::Operator(o) => match o {
                Operator::Terminator | Operator::AttrR => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn parse_module_param_list<I>(
        tokens: &mut Peekable<I>,
        last_line: usize,
    ) -> Result<(ParamList, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        // println!("{}", "PARSE PARAMS");

        let mut params = ParamList::new();
        let mut current_param_config = Port::new_param();
        let mut refs = RefSet::new();

        let mut counter = 0;
        let mut identified_param = false;

        let mut last_token_line = last_line;

        while let Some(t) = tokens.next() {
            let next_token_line = t.locate().line().clone();
            if next_token_line > last_token_line {
                // take all symbols until new line when handling a new directive
                if t.as_type().is_directive() == true {
                    let mut stmt = Statement::new();
                    stmt.push(t);
                    while let Some(t_next) = tokens.peek() {
                        if t_next.locate().line() > next_token_line {
                            break;
                        } else {
                            stmt.push(tokens.next().unwrap());
                        }
                    }
                    // println!("{}", statement_to_string(&stmt));
                    // get any refs from the statement with the compiler directive
                    if let Some(s_refs) = SystemVerilogSymbol::extract_refs_from_statement(&stmt) {
                        refs.extend(s_refs);
                    }
                    continue;
                }
            }
            last_token_line = next_token_line;

            if t.as_ref().is_eof() == true {
                return Err(VerilogError::ExpectingOperator(Operator::ParenR));
            // exit the param checking
            } else if t.as_ref().check_delimiter(&Operator::ParenR) == true {
                if counter == 0 {
                    break;
                }
                counter -= 1;
            } else if t.as_ref().check_delimiter(&Operator::ParenL) == true {
                counter += 1;
            // we are dealing with a param list
            } else if let Some(name) = t.as_ref().as_identifier() {
                if let Some(t_next) = tokens.peek() {
                    // then this is indeed the name of a param
                    // this is indeed the name of a datatype, specifically, a modport
                    if t_next.as_type().check_delimiter(&Operator::Dot) {
                        // set the interface name
                        current_param_config.set_data_type(t.as_ref().clone());
                        // update refs because it is an interface
                        refs.insert(CompoundIdentifier::new_minimal_verilog(name.clone()));
                        // collect the '.'
                        let _ = tokens.next().unwrap();
                        // TODO: verify the next item is an identifier to be a modport name?
                        current_param_config.set_modport(tokens.next().unwrap().take());
                    // this is indeed the name of a type, specifically being called from a package
                    } else if t_next.as_type().check_delimiter(&Operator::ScopeResolution) {
                        // set the package name
                        current_param_config.set_data_type(t.as_ref().clone());
                        // update refs because it is a package
                        refs.insert(CompoundIdentifier::new_minimal_verilog(name.clone()));
                        // collect the '::'
                        let _ = tokens.next().unwrap();
                        // TODO: verify the next item is an identifier to be a datatype name?
                        current_param_config.set_nested_type(tokens.next().unwrap().take());
                    // then this is indeed the name of a port
                    } else if t_next.as_type().check_delimiter(&Operator::Comma)
                        || t_next.as_type().check_delimiter(&Operator::BlockAssign)
                        || t_next.as_type().check_delimiter(&Operator::ParenR)
                        || t_next.as_type().check_delimiter(&Operator::BrackL)
                        || t_next.as_type().check_delimiter(&Operator::Terminator)
                        || t_next.as_type().is_directive()
                    {
                        // fix any misaligned data parsing
                        if identified_param == true {
                            current_param_config.fix_type(name.clone());
                            params.last_mut().unwrap().fix_type(name.clone());
                        // assume it is the name of a param (may correct later)
                        } else {
                            identified_param = true;
                            params.push(Port::with(name.clone(), true));
                            params.last_mut().unwrap().inherit(&current_param_config);
                            // determine if this port was declared with ANSI-style
                            if current_param_config.is_ansi_style() == true {
                                params.last_mut().unwrap().set_ansi();
                            }
                        }
                    // this may be a datatype!
                    } else {
                        current_param_config.set_data_type(t.as_ref().clone());
                    }
                }
            // proceed
            } else if t.as_ref().check_delimiter(&Operator::Comma) {
                // clear the default value (if exists)
                current_param_config.clear_default();
                identified_param = false;
                continue;
            // handle an attribute
            } else if t.as_ref().check_delimiter(&Operator::AttrL) {
                Self::parse_attr(tokens, t.into_position())?;
            // we are dealing with parameter declarations
            } else if t.as_ref().check_keyword(&Keyword::Parameter) {
                current_param_config = Port::new_param();
                current_param_config.set_direction(t.as_ref().as_keyword().unwrap().clone());
            // collect a range
            } else if t.as_ref().check_delimiter(&Operator::BrackL) {
                let stmt = Self::parse_until_operator(tokens, t, Operator::BrackR)?;
                // update references that might have appeared in range
                if let Some(s_refs) = SystemVerilogSymbol::extract_refs_from_statement(&stmt) {
                    refs.extend(s_refs);
                }
                // check if this is an unpacked array or packed array
                match identified_param {
                    true => {
                        params
                            .last_mut()
                            .unwrap()
                            .set_unpacked_range(into_tokens(stmt.clone()));
                        current_param_config.set_unpacked_range(into_tokens(stmt));
                    }
                    false => current_param_config.set_range(into_tokens(stmt)),
                }
            } else if t.as_ref().check_delimiter(&Operator::Dot) {
                return Err(VerilogError::UnhandledDotInDecl);
            // collect a default value
            } else if t.as_ref().check_delimiter(&Operator::Lte) {
                return Err(VerilogError::UnhandledAssignInDecl);
            } else if t.as_ref().check_delimiter(&Operator::BlockAssign) {
                let stmt = Self::parse_assignment(tokens, false)?;
                // update references that may appear in the assignment
                if let Some(s_refs) = SystemVerilogSymbol::extract_refs_from_statement(&stmt) {
                    refs.extend(s_refs);
                }
                // set the default for the last known param!
                if let Some(p) = params.last_mut() {
                    p.set_default(into_tokens(stmt));
                }
            } else if t.as_ref().check_keyword(&Keyword::Reg) {
                current_param_config.set_net_type(t.as_ref().as_keyword().unwrap().clone());
            } else if t.as_ref().check_keyword(&Keyword::Signed) {
                current_param_config.set_signed();
            } else if Self::is_valid_net_type(t.as_ref().as_keyword()) {
                current_param_config.set_net_type(t.as_ref().as_keyword().unwrap().clone());
            } else if Self::is_valid_data_type(t.as_ref()) {
                current_param_config.set_data_type(t.as_ref().clone());
            }
        }
        // println!("{:?}", params);
        Ok((params, refs))
    }

    pub fn parse_module_port_list<I>(
        tokens: &mut Peekable<I>,
        last_line: usize,
    ) -> Result<(PortList, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        // println!("{}", "PARSE PORTS");

        let mut ports = PortList::new();
        let mut current_port_config = Port::new_port();
        let mut refs = RefSet::new();

        let mut counter = 0;
        let mut identified_port = false;
        let mut last_token_line = last_line;

        while let Some(t) = tokens.next() {
            let next_token_line = t.locate().line().clone();
            if next_token_line > last_token_line {
                // take all symbols until new line when handling a new directive
                if t.as_type().is_directive() == true {
                    let mut stmt = Statement::new();
                    stmt.push(t);
                    while let Some(t_next) = tokens.peek() {
                        if t_next.locate().line() > next_token_line {
                            break;
                        } else {
                            stmt.push(tokens.next().unwrap());
                        }
                    }
                    // println!("{}", statement_to_string(&stmt));
                    // get any refs from the statement with the compiler directive
                    if let Some(s_refs) = SystemVerilogSymbol::extract_refs_from_statement(&stmt) {
                        refs.extend(s_refs);
                    }
                    continue;
                }
            }
            last_token_line = next_token_line;

            if t.as_ref().is_eof() == true {
                return Err(VerilogError::ExpectingOperator(Operator::ParenR));
            // exit the port checking
            } else if t.as_ref().check_delimiter(&Operator::ParenR) == true {
                if counter == 0 {
                    break;
                }
                counter -= 1;
            } else if t.as_ref().check_delimiter(&Operator::ParenL) == true {
                counter += 1;
            // we are dealing with a port list
            } else if let Some(name) = t.as_ref().as_identifier() {
                if let Some(t_next) = tokens.peek() {
                    // this is indeed the name of a datatype, specifically, a modport
                    if t_next.as_type().check_delimiter(&Operator::Dot) {
                        // set the interface name
                        current_port_config.set_data_type(t.as_ref().clone());
                        // update refs because it is an interface
                        refs.insert(CompoundIdentifier::new_minimal_verilog(name.clone()));
                        // collect the '.'
                        let _ = tokens.next().unwrap();
                        // TODO: verify the next item is an identifier to be a modport name?
                        current_port_config.set_modport(tokens.next().unwrap().take());
                    // this is indeed the name of a type, specifically being called from a package
                    } else if t_next.as_type().check_delimiter(&Operator::ScopeResolution) {
                        // set the package name
                        current_port_config.set_data_type(t.as_ref().clone());
                        // update refs because it is a package
                        refs.insert(CompoundIdentifier::new_minimal_verilog(name.clone()));
                        // collect the '::'
                        let _ = tokens.next().unwrap();
                        // TODO: verify the next item is an identifier to be a datatype name?
                        current_port_config.set_nested_type(tokens.next().unwrap().take());
                    // then this is indeed the name of a port
                    } else if t_next.as_type().check_delimiter(&Operator::Comma)
                        || t_next.as_type().check_delimiter(&Operator::BlockAssign)
                        || t_next.as_type().check_delimiter(&Operator::ParenR)
                        || t_next.as_type().check_delimiter(&Operator::BrackL)
                        || t_next.as_type().check_delimiter(&Operator::Terminator)
                        || t_next.as_type().is_directive()
                    {
                        // fix any misaligned data parsing
                        if identified_port == true {
                            current_port_config.fix_type(name.clone());
                            ports.last_mut().unwrap().fix_type(name.clone());
                        // assume it is the name of a port (may correct later)
                        } else {
                            identified_port = true;
                            ports.push(Port::with(name.clone(), false));
                            ports.last_mut().unwrap().inherit(&current_port_config);
                            // determine if this port was declared with ANSI-style
                            if current_port_config.is_ansi_style() == true {
                                ports.last_mut().unwrap().set_ansi();
                            }
                        }
                    // this may be a datatype!
                    } else {
                        current_port_config.set_data_type(t.as_ref().clone());
                    }
                }
            // proceed to the next decl
            } else if t.as_ref().check_delimiter(&Operator::Comma) {
                // clear the default value (if exists)
                current_port_config.clear_default();
                identified_port = false;
                continue;
            // we are dealing with port connections
            } else if t.as_ref().check_delimiter(&Operator::Dot) {
                return Err(VerilogError::UnhandledDotInDecl);
            } else if t.as_ref().check_delimiter(&Operator::Lte) {
                return Err(VerilogError::UnhandledAssignInDecl);
            // handle an attribute
            } else if t.as_ref().check_delimiter(&Operator::AttrL) {
                Self::parse_attr(tokens, t.into_position())?;
            // we are dealing with port declarations
            } else if Port::is_port_direction(t.as_ref().as_keyword()) {
                current_port_config = Port::new_port();
                current_port_config.set_direction(t.as_ref().as_keyword().unwrap().clone());
            // collect a range (can be multi-dimensional)
            } else if t.as_ref().check_delimiter(&Operator::BrackL) {
                let stmt = Self::parse_until_operator(tokens, t, Operator::BrackR)?;
                // update references that may appear in the assignment
                if let Some(s_refs) = SystemVerilogSymbol::extract_refs_from_statement(&stmt) {
                    refs.extend(s_refs);
                }
                // check if this is an unpacked array or packed array
                match identified_port {
                    true => {
                        ports
                            .last_mut()
                            .unwrap()
                            .set_unpacked_range(into_tokens(stmt.clone()));
                        current_port_config.set_unpacked_range(into_tokens(stmt));
                    }
                    false => current_port_config.set_range(into_tokens(stmt)),
                }
            // collect a default value
            } else if t.as_ref().check_delimiter(&Operator::BlockAssign) {
                let stmt = Self::parse_assignment(tokens, false)?;
                // update references that may appear in the assignment
                if let Some(s_refs) = SystemVerilogSymbol::extract_refs_from_statement(&stmt) {
                    refs.extend(s_refs);
                }
                // set the default for the last known port!
                if let Some(p) = ports.last_mut() {
                    p.set_default(into_tokens(stmt));
                }
            } else if t.as_ref().check_keyword(&Keyword::Reg) {
                current_port_config.set_net_type(t.as_ref().as_keyword().unwrap().clone());
            } else if t.as_ref().check_keyword(&Keyword::Signed) {
                current_port_config.set_signed();
            } else if Self::is_valid_net_type(t.as_ref().as_keyword()) {
                current_port_config.set_net_type(t.as_ref().as_keyword().unwrap().clone());
            } else if Self::is_valid_data_type(t.as_ref()) {
                current_port_config.set_data_type(t.as_ref().clone());
            }
        }

        Ok((ports, refs))
    }

    fn parse_port_connection<I>(
        _tokens: &mut Peekable<I>,
    ) -> Result<(Vec<Statement>, Vec<Statement>, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        todo!()
    }
}
