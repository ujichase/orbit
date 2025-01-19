//
//  Copyright (C) 2022-2025  Chase Ruskin
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

use checker::Checker;
use class::Class;
use config::Config;
use interface::Interface;
use module::Module;
use package::Package;
use primitive::Primitive;
use program::Program;

use super::error::SystemVerilogError;
use super::token::identifier::Identifier;
use super::token::operator::Operator;
use super::token::tokenizer::SystemVerilogTokenizer;
use crate::core::lang::lexer::{Position, Token};
use crate::core::lang::parser::{Parse, Symbol};
use crate::core::lang::reference::{CompoundIdentifier, RefSet};
use crate::core::lang::sv::token::keyword::Keyword;
use crate::core::lang::sv::token::token::SystemVerilogToken;
use crate::core::lang::verilog::symbols::VerilogSymbol;
use std::str::FromStr;

pub type Statement = Vec<Token<SystemVerilogToken>>;

pub mod checker;
pub mod class;
pub mod config;
pub mod interface;
pub mod module;
pub mod package;
pub mod primitive;
pub mod program;

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

/// Design elements of the SystemVerilog Language.
#[derive(Debug, PartialEq)]
pub enum SystemVerilogSymbol {
    Module(Module),
    Config(Config),
    Package(Package),
    Interface(Interface),
    Class(Class),
    Primitive(Primitive),
    Checker(Checker),
    Program(Program),
}

impl SystemVerilogSymbol {
    pub fn as_name(&self) -> Option<&Identifier> {
        match &self {
            Self::Module(m) => Some(m.get_name()),
            Self::Config(c) => Some(c.get_name()),
            Self::Package(p) => Some(p.get_name()),
            Self::Interface(p) => Some(p.get_name()),
            Self::Class(c) => Some(c.get_name()),
            Self::Primitive(p) => Some(p.get_name()),
            Self::Checker(c) => Some(c.get_name()),
            Self::Program(p) => Some(p.get_name()),
        }
    }

    pub fn get_position(&self) -> &Position {
        match self {
            Self::Module(m) => m.get_position(),
            Self::Config(c) => c.get_position(),
            Self::Package(p) => p.get_position(),
            Self::Interface(p) => p.get_position(),
            Self::Class(c) => c.get_position(),
            Self::Primitive(p) => p.get_position(),
            Self::Checker(c) => c.get_position(),
            Self::Program(p) => p.get_position(),
        }
    }

    pub fn as_module(&self) -> Option<&Module> {
        match &self {
            Self::Module(m) => Some(m),
            _ => None,
        }
    }

    pub fn get_refs(&self) -> &RefSet {
        match &self {
            Self::Module(m) => m.get_refs(),
            Self::Config(c) => c.get_refs(),
            Self::Package(p) => p.get_refs(),
            Self::Interface(p) => p.get_refs(),
            Self::Class(c) => c.get_refs(),
            Self::Primitive(p) => p.get_refs(),
            Self::Checker(c) => c.get_refs(),
            Self::Program(p) => p.get_refs(),
        }
    }

    pub fn extend_refs(&mut self, refs: RefSet) {
        match self {
            Self::Module(m) => m.extend_refs(refs),
            Self::Config(c) => c.extend_refs(refs),
            Self::Package(p) => p.extend_refs(refs),
            Self::Interface(p) => p.extend_refs(refs),
            Self::Class(c) => c.extend_refs(refs),
            Self::Primitive(p) => p.extend_refs(refs),
            Self::Checker(c) => c.extend_refs(refs),
            Self::Program(p) => p.extend_refs(refs),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SystemVerilogParser {
    symbols: Vec<Symbol<SystemVerilogSymbol>>,
}

impl SystemVerilogParser {
    /// Quietly ignores any errors and returns the list of symbols.
    pub fn read_lazy(s: &str) -> Self {
        let symbols =
            SystemVerilogParser::parse(SystemVerilogTokenizer::from_source_code(&s).into_tokens());
        Self {
            symbols: symbols
                .into_iter()
                .filter_map(|f| if f.is_ok() { Some(f.unwrap()) } else { None })
                .collect(),
        }
    }

    /// Reports an error if one is discovered in the list of symbols or in the tokenizing.
    pub fn read(s: &str) -> Result<Self, SystemVerilogError> {
        let symbols = SystemVerilogParser::parse(
            SystemVerilogTokenizer::from_str(&s)?
                .into_tokens()
                .into_iter()
                .filter(|s| s.as_type().is_comment() == false)
                .collect(),
        );
        let result: Result<Vec<Symbol<SystemVerilogSymbol>>, SystemVerilogError> =
            symbols.into_iter().collect();
        Ok(Self { symbols: result? })
    }

    pub fn into_symbols(self) -> Vec<SystemVerilogSymbol> {
        self.symbols.into_iter().map(|f| f.take()).collect()
    }
}

impl Parse<SystemVerilogToken> for SystemVerilogParser {
    type SymbolType = SystemVerilogSymbol;
    type SymbolError = SystemVerilogError;

    fn parse(
        tokens: Vec<Token<SystemVerilogToken>>,
    ) -> Vec<Result<Symbol<Self::SymbolType>, Self::SymbolError>> {
        let mut symbols = Vec::new();
        let mut tokens = tokens.into_iter().peekable();

        let mut global_refs = RefSet::new();

        let mut now_line = None;

        while let Some(t) = tokens.next() {
            let next_line = t.locate().line();
            let is_first_token_on_line = now_line.is_none() || next_line > now_line.unwrap();
            now_line = Some(next_line);
            // println!("[global]: {:?}", t);
            // create module design element
            if t.as_type().check_keyword(&Keyword::Module)
                || t.as_type().check_keyword(&Keyword::Macromodule)
            {
                symbols.push(
                    match SystemVerilogSymbol::parse_module(&mut tokens, t.into_position()) {
                        Ok(module) => Ok(Symbol::new(module)),
                        Err(e) => Err(e),
                    },
                );
            // create package design element
            } else if t.as_type().check_keyword(&Keyword::Package) {
                symbols.push(
                    match SystemVerilogSymbol::parse_package(&mut tokens, t.into_position()) {
                        Ok(module) => Ok(Symbol::new(module)),
                        Err(e) => Err(e),
                    },
                );
            // create a class design element
            } else if (t.as_type().check_keyword(&Keyword::Virtual)
                && tokens.peek().is_some()
                && tokens
                    .peek()
                    .unwrap()
                    .as_type()
                    .check_keyword(&Keyword::Class))
                || (t.as_type().check_keyword(&Keyword::Interface)
                    && tokens.peek().is_some()
                    && tokens
                        .peek()
                        .unwrap()
                        .as_type()
                        .check_keyword(&Keyword::Class))
                || t.as_type().check_keyword(&Keyword::Class)
            {
                symbols.push(
                    match SystemVerilogSymbol::parse_class(&mut tokens, t.into_position()) {
                        Ok(module) => Ok(Symbol::new(module)),
                        Err(e) => Err(e),
                    },
                )
            // create primitive design element
            } else if t.as_type().check_keyword(&Keyword::Primitive) {
                symbols.push(
                    match SystemVerilogSymbol::parse_primitive(&mut tokens, t.into_position()) {
                        Ok(prim) => Ok(Symbol::new(prim)),
                        Err(e) => Err(e),
                    },
                )
            // create program design element
            } else if t.as_type().check_keyword(&Keyword::Program) {
                symbols.push(
                    match SystemVerilogSymbol::parse_program(&mut tokens, t.into_position()) {
                        Ok(prog) => Ok(Symbol::new(prog)),
                        Err(e) => Err(e),
                    },
                )
            // create config design element
            } else if t.as_type().check_keyword(&Keyword::Config) {
                symbols.push(
                    match SystemVerilogSymbol::parse_config(&mut tokens, t.into_position()) {
                        Ok(config) => Ok(Symbol::new(config)),
                        Err(e) => Err(e),
                    },
                )
            // create checker design element
            } else if t.as_type().check_keyword(&Keyword::Checker) {
                symbols.push(
                    match SystemVerilogSymbol::parse_checker(&mut tokens, t.into_position()) {
                        Ok(check) => Ok(Symbol::new(check)),
                        Err(e) => Err(e),
                    },
                )
            // create interface design element
            } else if t.as_type().check_keyword(&Keyword::Interface) {
                symbols.push(
                    match SystemVerilogSymbol::parse_interface(&mut tokens, t.into_position()) {
                        Ok(intf) => Ok(Symbol::new(intf)),
                        Err(e) => Err(e),
                    },
                )
            // take a global import statement
            } else if t.as_type().check_keyword(&Keyword::Import) {
                // verify the import statement parsed okay
                let i_refs = match SystemVerilogSymbol::parse_import_statement(&mut tokens) {
                    Ok(i) => Some(i),
                    Err(e) => {
                        symbols.push(Err(e));
                        None
                    }
                };
                // append to this file's global references
                if let Some(i_refs) = i_refs {
                    global_refs.extend(i_refs);
                }
            // take attribute and ignore if okay
            } else if t.as_type().check_delimiter(&Operator::AttrL) {
                match SystemVerilogSymbol::parse_attr(&mut tokens, t.into_position()) {
                    Ok(_) => (),
                    Err(e) => symbols.push(Err(e)),
                }
            // take compiler directive that is first token on a line
            } else if t.as_ref().is_directive() == true && is_first_token_on_line == true {
                match VerilogSymbol::parse_compiler_directive_statement(&mut tokens, t) {
                    Ok(stmt) => {
                        if let Some(d_refs) =
                            SystemVerilogSymbol::extract_refs_from_statement(&stmt)
                        {
                            global_refs.extend(d_refs);
                        }
                    }
                    Err(e) => symbols.push(Err(e)),
                }
            // skip any potential illegal/unknown tokens at global scale
            } else if t.as_type().is_eof() == false {
                // symbols.push(Err(VerilogError::Vague))
                continue;
            }
        }

        // update all known symbols with the global reference statements
        if global_refs.is_empty() == false {
            symbols
                .iter_mut()
                .filter_map(|s| match s {
                    Ok(r) => Some(r.as_ref_mut()),
                    Err(_) => None,
                })
                .for_each(|s| {
                    s.extend_refs(global_refs.clone());
                });
        }

        symbols
    }
}

impl SystemVerilogSymbol {
    fn parse_module<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(Self::Module(Module::from_tokens(
            tokens,
            pos,
            "systemverilog",
        )?))
    }

    fn parse_class<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(Self::Class(Class::from_tokens(tokens, pos)?))
    }

    fn parse_program<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(Self::Program(Program::from_tokens(tokens, pos)?))
    }

    fn parse_checker<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(Self::Checker(Checker::from_tokens(tokens, pos)?))
    }

    fn parse_package<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(Self::Package(Package::from_tokens(tokens, pos)?))
    }

    fn parse_config<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(Self::Config(Config::from_tokens(tokens, pos)?))
    }

    fn parse_primitive<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(Self::Primitive(Primitive::from_tokens(tokens, pos)?))
    }

    fn parse_interface<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(Self::Interface(Interface::from_tokens(tokens, pos)?))
    }
}

impl SystemVerilogSymbol {
    fn parse_attr<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<Statement, SystemVerilogError>
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
                return Err(SystemVerilogError::ExpectingOperator(Operator::AttrR));
            }
            stmt.push(t);
        }
        Ok(stmt)
    }

    /// Parses a statement that is for importing packages.
    ///
    /// This function assumes the last token consumed was the `import` keyword.
    /// The last token this function will consume is the `;` operator.
    pub fn parse_import_statement<I>(tokens: &mut Peekable<I>) -> Result<RefSet, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        let mut refs = RefSet::new();
        let mut is_start_of_item = true;
        while let Some(t) = tokens.next() {
            // whoops... this shouldn't be the end of the file!
            if t.as_type().is_eof() {
                return Err(SystemVerilogError::ExpectingOperator(Operator::Terminator));
            // insert the package identifier!
            } else if is_start_of_item && t.as_type().as_identifier().is_some() {
                refs.insert(CompoundIdentifier::new_minimal_verilog(
                    t.take().take_identifier().unwrap(),
                ));
                is_start_of_item = false;
            // reset the package item marker
            } else if t.as_type().check_delimiter(&Operator::Comma) {
                is_start_of_item = true;
            // stop parsing tokens
            } else if t.as_type().check_delimiter(&Operator::Terminator) {
                break;
            }
        }

        Ok(refs)
    }

    /// Extracts any references found in a statement, if they exist.
    ///
    /// References can be found hidden in statements where the package identifier is
    /// the token immediately before a scope resolution operator `::`.
    pub fn extract_refs_from_statement(stmt: &Statement) -> Option<RefSet> {
        // return none if we cannot find the scope resolution operator
        stmt.iter()
            .find(|c| c.as_type().check_delimiter(&Operator::ScopeResolution))?;

        let mut refs = RefSet::new();
        let mut iter = stmt.iter();

        let mut prev_t = iter.next()?;
        // check if there is a scope resolution operator, then chec
        while let Some(t) = iter.next() {
            // if currently at `::`, then the previous token was a package identifier!
            if t.as_type().check_delimiter(&Operator::ScopeResolution) {
                if let Some(pkg_id) = prev_t.as_type().as_identifier() {
                    refs.insert(CompoundIdentifier::new_minimal_verilog(pkg_id.clone()));
                }
                // update the previous token
                prev_t = t;
            }
        }
        match refs.is_empty() {
            true => None,
            false => Some(refs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ut_code_example_1() {
        let code = r#"// code file
module dimm(addr, ba, rasx, casx, csx, wex, cke, clk, dqm, data, dev_id);

parameter [31:0] MEM_WIDTH = 16, MEM_SIZE = 8; // in mbytes
input [10:0] addr;
input ba, rasx, casx, csx, wex, cke, clk; input [ 7:0] dqm;
inout [63:0] data;
input [ 4:0] dev_id;
genvar i;

case ({MEM_SIZE, MEM_WIDTH})
    {32'd8, 32'd16}: // 8Meg x 16 bits wide
        begin: memory
            for (i=0; i<4; i=i+1) begin:word16
                sms_08b216t0 p(.clk(clk), .csb(csx), .cke(cke),.ba(ba),
                            .addr(addr), .rasb(rasx), .casb(casx),
                            .web(wex), .udqm(dqm[2*i+1]), .ldqm(dqm[2*i]),
                            .dqi(data[15+16*i:16*i]), .dev_id(dev_id));
                // The hierarchical instance names are:
                // memory.word16[3].p, memory.word16[2].p,
                // memory.word16[1].p, memory.word16[0].p, 
            end
            // and the task memory.read_mem
            task read_mem;
                input [31:0] address;
                output [63:0] data;
                begin // call read_mem in sms module
                    word[3].p.read_mem(address, data[63:48]);
                    word[2].p.read_mem(address, data[47:32]);
                    word[1].p.read_mem(address, data[31:16]);
                    endword[0].p.read_mem(address, data[15: 0]); 
                end
            endtask
        end
    {32'd16, 32'd8}: // 16Meg x 8 bits wide 
        begin: memory
            for (i=0; i<8; i=i+1) begin:word8
                sms_16b208t0 p(.clk(clk), .csb(csx), .cke(cke),.ba(ba),
                                .addr(addr), .rasb(rasx), .casb(casx),
                                .web(wex), .dqm(dqm[i]),
                                .dqi(data[7+8*i:8*i]), .dev_id(dev_id));
                   // The hierarchical instance names are
                   // memory.word8[7].p, memory.word8[6].p,
                   // ...
                   // memory.word8[1].p, memory.word8[0].p,
            end // and the task memory.read_mem
            task read_mem;
                input [31:0] address;
                output [63:0] data;
                begin // call read_mem in sms module
                    byte[7].p.read_mem(address, data[63:56]);
                    byte[6].p.read_mem(address, data[55:48]);
                    byte[5].p.read_mem(address, data[47:40]);
                    byte[4].p.read_mem(address, data[39:32]);
                    byte[3].p.read_mem(address, data[31:24]);
                    byte[2].p.read_mem(address, data[23:16]);
                    byte[1].p.read_mem(address, data[15: 8]);
                    endbyte[0].p.read_mem(address, data[ 7: 0]); 
                end
            endtask
        end
    // Other memory cases ...
    endcase
endmodule
        "#;
        let symbols = SystemVerilogParser::read(&code).unwrap().into_symbols();
        let sub_mod_instances = symbols
            .first()
            .unwrap()
            .as_module()
            .unwrap()
            .get_edge_list_entities();
        assert_eq!(
            sub_mod_instances,
            vec![
                CompoundIdentifier::new_minimal_verilog(
                    Identifier::from_str("sms_08b216t0").unwrap()
                ),
                CompoundIdentifier::new_minimal_verilog(
                    Identifier::from_str("sms_16b208t0").unwrap()
                ),
            ]
        );
    }
}
