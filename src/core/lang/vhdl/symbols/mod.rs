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

use std::collections::HashSet;
use std::fmt::Display;

use std::str::FromStr;

use self::architecture::Architecture;
use self::configuration::Configuration;
use self::entity::Entity;
use self::package::Package;
use self::packagebody::PackageBody;

use super::super::lexer::*;
use super::super::parser::*;

use crate::core::lang::reference::{CompoundIdentifier, RefSet};
use crate::core::lang::vhdl::interface::*;
use crate::core::lang::vhdl::token::*;

pub mod architecture;
pub mod configuration;
pub mod context;
pub mod entity;
pub mod package;
pub mod packagebody;

#[derive(Debug, PartialEq)]
pub enum VhdlSymbol {
    // primary design units
    Entity(Entity),
    Context(Context),
    Package(Package),
    Configuration(Configuration),
    // secondary design units
    Architecture(Architecture),
    PackageBody(PackageBody),
}

impl From<Entity> for VhdlSymbol {
    fn from(value: Entity) -> Self {
        Self::Entity(value)
    }
}

impl VhdlSymbol {
    /// Casts `self` to identifier.
    pub fn get_name(&self) -> Option<&Identifier> {
        match self {
            Self::Entity(e) => Some(&e.get_name()),
            Self::Architecture(a) => Some(&a.get_name()),
            Self::Package(p) => Some(&p.get_name()),
            Self::PackageBody(_) => None,
            Self::Configuration(c) => Some(&c.get_name()),
            Self::Context(c) => Some(&c.name),
        }
    }

    /// References the starting location in the file for the primary design unit.
    pub fn get_position(&self) -> &Position {
        match self {
            Self::Entity(unit) => unit.get_position(),
            Self::Architecture(unit) => unit.get_position(),
            Self::Package(unit) => unit.get_position(),
            Self::PackageBody(unit) => unit.get_position(),
            Self::Configuration(unit) => unit.get_position(),
            Self::Context(unit) => unit.get_position(),
        }
    }

    /// Checks if the symbol is a primary design unit.
    pub fn is_primary(&self) -> bool {
        self.as_package().is_some()
            || self.as_entity().is_some()
            || self.as_configuration().is_some()
            || self.as_context().is_some()
    }

    /// Casts `self` to package.
    pub fn as_package(&self) -> Option<&Package> {
        match self {
            Self::Package(p) => Some(p),
            _ => None,
        }
    }

    /// Casts `self` to entity.
    pub fn as_entity(&self) -> Option<&Entity> {
        match self {
            Self::Entity(e) => Some(e),
            _ => None,
        }
    }

    /// Casts `self` to configuration.
    pub fn as_configuration(&self) -> Option<&Configuration> {
        match self {
            Self::Configuration(cfg) => Some(cfg),
            _ => None,
        }
    }

    /// Casts `self` to context.
    pub fn as_context(&self) -> Option<&Context> {
        match self {
            Self::Context(cxt) => Some(cxt),
            _ => None,
        }
    }

    /// Transforms `self` to entity.
    pub fn into_entity(self) -> Option<Entity> {
        match self {
            Self::Entity(e) => Some(e),
            _ => None,
        }
    }

    /// Transforms `self` into architecture.
    pub fn into_architecture(self) -> Option<Architecture> {
        match self {
            Self::Architecture(arch) => Some(arch),
            _ => None,
        }
    }

    /// Transforms `self` into package.
    pub fn into_package(self) -> Option<Package> {
        match self {
            Self::Package(p) => Some(p),
            _ => None,
        }
    }

    /// Casts `self` to architecture.
    pub fn as_architecture(&self) -> Option<&Architecture> {
        match self {
            Self::Architecture(arch) => Some(arch),
            _ => None,
        }
    }

    pub fn steal_refs(&mut self, refs: RefSet) -> RefSet {
        match self {
            Self::Entity(e) => e.get_refs_mut().extend(refs),
            Self::Architecture(a) => a.get_refs_mut().extend(refs),
            Self::Package(p) => p.get_refs_mut().extend(refs),
            Self::PackageBody(pb) => pb.get_refs_mut().extend(refs),
            Self::Context(cx) => cx.refs.extend(refs),
            Self::Configuration(cf) => cf.get_refs_mut().extend(refs),
        };
        RefSet::new()
    }

    pub fn get_refs(&self) -> &RefSet {
        match self {
            Self::Entity(e) => e.get_refs(),
            Self::Architecture(a) => a.get_refs(),
            Self::Package(p) => p.get_refs(),
            Self::PackageBody(pb) => pb.get_refs(),
            Self::Context(cx) => cx.get_refs(),
            Self::Configuration(cf) => cf.get_refs(),
        }
    }
}

impl Display for VhdlSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Entity(e) => format!(
                "entity {} {{ generics={:?} ports={:?} }}",
                &e.get_name(),
                e.get_generics(),
                e.get_ports()
            ),
            Self::PackageBody(pb) => format!("package body- {}", pb),
            Self::Architecture(a) => format!(
                "architecture {} for entity {}",
                &a.get_name(),
                &a.get_owner()
            ),
            Self::Package(p) => format!("package {}", &p),
            Self::Configuration(c) => format!(
                "configuration {} for entity {}",
                &c.get_name(),
                &c.get_owner()
            ),
            Self::Context(c) => format!("context {}", &c.name),
        };
        write!(f, "{}", s)
    }
}

use super::error::VhdlError;

#[derive(Debug, PartialEq)]
pub struct Context {
    name: Identifier,
    refs: RefSet,
    pos: Position,
}

impl Context {
    /// Accesses the references for the entity.
    pub fn get_refs(&self) -> &RefSet {
        &self.refs
    }

    pub fn get_position(&self) -> &Position {
        &self.pos
    }
}

#[derive(Debug, PartialEq)]
pub enum ContextUsage {
    ContextDeclaration(Context),
    ContextReference(RefSet),
}

#[derive(Debug, PartialEq)]
pub enum ContextStatement {
    LibraryClause,
    UseClause(UseClause),
    // @TODO Context_reference
}

#[derive(Debug, PartialEq)]
pub struct UseClause {
    imports: Vec<SelectedName>,
}

/*
    @NOTE In order to detect if a package was used, the best bet is to just
    iterate through the the tokens and collect all simple names, i.e.
    library.name.name. , then try check against the data structure if the name
    matches anywhere. If so, then it is considered a reference and is needed in
    that design.
*/

/*
    @NOTE To check instantiations, check for all identifiers against the
    list of known public identifiers from external API.

    Example: given a ip with public primary design units: adder, adder_pkg
*/

#[derive(Debug, PartialEq)]
struct SelectedName(Vec<Identifier>);

impl SelectedName {
    /// Returns the final identifier in the chain.
    fn get_suffix(&self) -> &Identifier {
        self.0.last().unwrap()
    }

    fn take_suffix(mut self) -> Identifier {
        self.0.pop().unwrap()
    }

    /// Casts the list of identifiers into a list of `CompoundIdentifiers`.
    ///
    /// If `sep_last` is `true`, then an extra compound will be made with just the
    /// suffix and no prefix.
    fn into_compound_identifiers(self, sep_last: bool) -> RefSet {
        let mut result = RefSet::new();

        let mut iter = self.0.into_iter().peekable();
        while let Some(iden) = iter.next() {
            match iter.peek() {
                Some(next) => {
                    result.insert(CompoundIdentifier::new_vhdl(iden, next.clone()));
                }
                None => {
                    if sep_last == true {
                        result.insert(CompoundIdentifier::new_minimal_vhdl(iden));
                    }
                }
            }
        }

        result
    }
}

#[derive(Debug, PartialEq)]
pub struct VHDLParser {
    symbols: Vec<Symbol<VhdlSymbol>>,
}

impl Parse<VhdlToken> for VHDLParser {
    type SymbolType = VhdlSymbol;
    type SymbolError = VhdlError;

    fn parse(
        tokens: Vec<Token<VhdlToken>>,
    ) -> Vec<Result<Symbol<Self::SymbolType>, Self::SymbolError>> {
        let mut symbols = Vec::new();
        let mut tokens = tokens.into_iter().peekable();

        let mut global_refs = RefSet::new();

        while let Some(t) = tokens.next() {
            // create entity symbol
            if t.as_ref().check_keyword(&Keyword::Entity) {
                // get the position
                symbols.push(
                    match VhdlSymbol::parse_entity(&mut tokens, t.into_position()) {
                        Ok(mut ent) => {
                            // println!("info: detected {}", ent);
                            // println!("global refs: {:?}", global_refs);
                            global_refs = ent.steal_refs(global_refs);
                            Ok(Symbol::new(ent))
                        }
                        Err(e) => Err(e),
                    },
                );
            // create architecture symbol
            } else if t.as_ref().check_keyword(&Keyword::Architecture) {
                symbols.push(
                    match VhdlSymbol::parse_architecture(&mut tokens, t.into_position()) {
                        Ok(mut arch) => {
                            global_refs = arch.steal_refs(global_refs);
                            // println!("info: detected {}", arch);
                            Ok(Symbol::new(arch))
                        }
                        Err(e) => Err(e),
                    },
                );
            // create configuration symbol
            } else if t.as_ref().check_keyword(&Keyword::Configuration) {
                symbols.push(
                    match VhdlSymbol::parse_configuration(&mut tokens, t.into_position()) {
                        Ok(config) => Ok(Symbol::new(config)),
                        Err(e) => Err(e),
                    },
                );
            // create package symbol
            } else if t.as_ref().check_keyword(&Keyword::Package) {
                symbols.push(
                    match VhdlSymbol::route_package_parse(&mut tokens, t.into_position()) {
                        Ok(mut pack) => {
                            global_refs = pack.steal_refs(global_refs);
                            // println!("info: detected {}", pack);
                            Ok(Symbol::new(pack))
                        }
                        Err(e) => Err(e),
                    },
                );
            // create a context symbol or context reference
            } else if t.as_ref().check_keyword(&Keyword::Context) {
                match VhdlSymbol::parse_context(&mut tokens, t.into_position()) {
                    ContextUsage::ContextDeclaration(dec) => {
                        let mut context = VhdlSymbol::Context(dec);
                        // println!("info: detected {}", context);
                        global_refs = context.steal_refs(global_refs);
                        symbols.push(Ok(Symbol::new(context)));
                    }
                    ContextUsage::ContextReference(c_refs) => {
                        global_refs.extend(c_refs);
                    }
                };
            // handle global statements (`USE`, `LIBRARY` statements, or invalid code)
            } else {
                // update global references list
                global_refs.extend(VhdlSymbol::parse_statement(&mut tokens).1);
            }
        }
        // println!("{:#?}", symbols);
        symbols
    }
}

impl VHDLParser {
    /// Quietly ignores any errors and returns the list of symbols.
    pub fn read_lazy(s: &str) -> Self {
        let symbols = VHDLParser::parse(VhdlTokenizer::from_source_code(&s).into_tokens());
        Self {
            symbols: symbols
                .into_iter()
                .filter_map(|f| if f.is_ok() { Some(f.unwrap()) } else { None })
                .collect(),
        }
    }

    /// Reports an error if one is discovered in the list of symbols or in the tokenizing.
    pub fn read(s: &str) -> Result<Self, VhdlError> {
        let symbols = VHDLParser::parse(VhdlTokenizer::from_str(&s)?.into_tokens());
        let result: Result<Vec<Symbol<VhdlSymbol>>, VhdlError> = symbols.into_iter().collect();
        Ok(Self { symbols: result? })
    }

    pub fn into_symbols(self) -> Vec<VhdlSymbol> {
        self.symbols.into_iter().map(|f| f.take()).collect()
    }
}

use std::iter::Peekable;

use super::highlight::*;

type TokenPair = (Statement, RefSet);

/// A `Statement` is a vector of tokens, similiar to how a `String` is a vector
/// of characters.
#[derive(PartialEq, Clone)]
struct Statement(Vec<Token<VhdlToken>>);

impl Statement {
    /// References the mutable list of vhdl tokens.
    fn get_tokens_mut(&mut self) -> &mut Vec<Token<VhdlToken>> {
        &mut self.0
    }
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // determine which delimiters to not add trailing spaces to
        let is_spaced_token = |d: &Delimiter| match d {
            Delimiter::ParenL | Delimiter::ParenR => false,
            _ => true,
        };
        // iterate through the tokens
        let mut iter = self.0.iter().peekable();
        while let Some(t) = iter.next() {
            let trailing_space = match t.as_ref() {
                VhdlToken::Delimiter(d) => is_spaced_token(d),
                _ => {
                    // make sure the next token is not a tight token (no-spaced)
                    if let Some(m) = iter.peek() {
                        match m.as_ref() {
                            VhdlToken::Delimiter(d) => is_spaced_token(d),
                            _ => true,
                        }
                    } else {
                        true
                    }
                }
            };
            write!(f, "{}", t.as_ref().to_string())?;
            if trailing_space == true && iter.peek().is_some() {
                write!(f, " ")?
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for t in &self.0 {
            write!(f, "{} ", t.as_ref().to_string())?
        }
        Ok(())
    }
}

impl Statement {
    fn new() -> Self {
        Self(Vec::new())
    }

    /// References the tokens as VHDLTokens.
    fn as_types(&self) -> Vec<&VhdlToken> {
        self.0.iter().map(|f| f.as_type()).collect()
    }

    /// References the list of tokens.
    fn get_tokens(&self) -> &Vec<Token<VhdlToken>> {
        &self.0
    }
}

impl VhdlSymbol {
    /// Parses an `Entity` primary design unit from the entity's identifier to
    /// the END closing statement.
    fn parse_entity<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<VhdlSymbol, VhdlError>
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        Ok(VhdlSymbol::Entity(Entity::from_tokens(tokens, pos)?))
    }

    /// Parses a package declaration, from the <package> IS to the END keyword.
    ///
    /// Assumes the last consumed token was PACKAGE keyword and the next token
    /// is the identifier for the package name.
    fn parse_package_declaration<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<VhdlSymbol, VhdlError>
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        let mut refs = RefSet::new();
        // take package name
        let pack_name = tokens.next().take().unwrap().take();
        // println!("{:?}", pack_name);
        // take the IS keyword
        if tokens
            .next()
            .take()
            .unwrap()
            .as_type()
            .check_keyword(&Keyword::Is)
            == false
        {
            // panic!("expecting keyword IS")
            return Err(VhdlError::Vague);
        }

        // check if there is a NEW keyword to return instantiation
        if tokens.peek().is_some()
            && tokens
                .peek()
                .unwrap()
                .as_type()
                .check_keyword(&Keyword::New)
                == true
        {
            // parse the statement to take the package instantiation line
            let (_clause, c_refs) = Self::parse_statement(tokens);
            // construct a new package
            return Ok(VhdlSymbol::Package(Package::new(
                match pack_name {
                    VhdlToken::Identifier(id) => id,
                    _ => {
                        // expecting identifier
                        return Err(VhdlError::Vague);
                    }
                },
                c_refs,
                pos,
            )));
        }

        // check if there is a generic clause
        let generics = if tokens.peek().is_some()
            && tokens
                .peek()
                .unwrap()
                .as_type()
                .check_keyword(&Keyword::Generic)
                == true
        {
            // take the GENERIC keyword token
            tokens.next();
            Self::parse_interface_list(tokens)?
        } else {
            Vec::new()
        };

        // grab references from generic interface list
        let generics = generics
            .into_iter()
            .map(|f| {
                refs.extend(f.1);
                f.0 .0
            })
            .collect::<Vec<Vec<Token<VhdlToken>>>>();

        // println!("{:?}", generics);

        // compose the declarative items
        while let Some(t) = tokens.peek() {
            // check for nested package declarations
            if t.as_type().check_keyword(&Keyword::Package) {
                // consume PACKAGE keyword
                let inner_pos = tokens.next().unwrap().into_position();
                // parse nested package declaration and grab references
                let inner_pack = Self::parse_package_declaration(tokens, inner_pos)?;
                refs.extend(inner_pack.into_package().unwrap().into_refs());
            // grab component declarations
            } else if t.as_type().check_keyword(&Keyword::Component) {
                let _comp = Self::parse_component(tokens);
                // println!("component declared: {}", comp);
            } else if t.as_type().check_keyword(&Keyword::End) {
                Self::parse_statement(tokens);
                break;
            } else {
                refs.extend(Self::parse_statement(tokens).1);
            }
        }

        // println!("*--- unit {}", pack_name);
        Ok(VhdlSymbol::Package(
            Package::new(
                match pack_name {
                    VhdlToken::Identifier(id) => id,
                    _ => {
                        // expecting identifier
                        return Err(VhdlError::Vague);
                    }
                },
                refs,
                pos,
            )
            .generics(Generics(InterfaceDeclarations::from_double_listed_tokens(
                generics,
            ))),
        ))
    }

    /// Creates a `Context` struct for primary design unit: context.
    ///
    /// Assumes the next token to consume is the context's identifier.
    fn parse_context<I>(tokens: &mut Peekable<I>, pos: Position) -> ContextUsage
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        // grab the identifier name
        let iden = tokens.next().unwrap().take().take_identifier().unwrap();
        // check the next token is the `is` keyword for declaration
        if tokens.peek().unwrap().as_ref().check_keyword(&Keyword::Is) == true {
            ContextUsage::ContextDeclaration(Context {
                name: iden,
                refs: Self::parse_context_declaration(tokens),
                pos: pos,
            })
        // parse statement
        } else {
            let mut subtokens = vec![Token::new(VhdlToken::Identifier(iden), Position::new())];
            while let Some(t) = tokens.next() {
                if t.as_ref().check_delimiter(&Delimiter::Terminator) == true {
                    subtokens.push(t);
                    break;
                }
                subtokens.push(t);
            }
            let stmt = Self::parse_statement(&mut subtokens.into_iter().peekable());
            ContextUsage::ContextReference(stmt.1)
        }
    }

    /// Creates a `Context` struct for primary design unit: context.
    ///
    /// Assumes the next token to consume is the keyword `IS`. Stops at the `end`.
    fn parse_context_declaration<I>(tokens: &mut Peekable<I>) -> RefSet
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        let mut result = RefSet::new();

        while let Some(t) = tokens.next() {
            let (clause, c_refs) = Self::parse_statement(tokens);
            if t.as_ref().check_keyword(&Keyword::End) == true {
                if Self::is_primary_ending(&clause) == true {
                    break;
                }
            } else {
                // get references
                result.extend(c_refs);
            }
        }
        result
    }

    /// Collects identifiers into a single vector, stopping at a non-identifier token.
    ///
    /// Assumes the first token to consume is an identifier, and continues to collect
    /// if the next token is a DOT delimiter.
    fn compose_name<I>(tokens: &mut Peekable<I>) -> SelectedName
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        let mut selected_name = Vec::new();
        // take first token as identifier
        let tk_id = tokens.next().expect("expecting name after '.'");
        if let Some(id) = tk_id.take().take_identifier() {
            selected_name.push(id);
        }
        while let Some(t) = tokens.peek() {
            // consume DOT and expect next identifier
            if t.as_type().check_delimiter(&Delimiter::Dot) {
                // consume DOT
                tokens.next();
                // expect identifier or bail
                let tk_id = tokens.next().expect("expecting name after '.'");

                if tk_id.as_type().check_keyword(&Keyword::All) {
                    // @TODO remember in `name` struct that all was used.
                    break;
                } else if let Some(id) = tk_id.take().take_identifier() {
                    selected_name.push(id);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        SelectedName(selected_name)
    }

    /// Parses a package body, taking BODY keyword up until the END keyword.
    ///
    /// Package declarations within this scope can be ignored because their visibility
    /// is not reached outside of the body.
    fn parse_package_body<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<PackageBody, VhdlError>
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        // take the 'body' keyword
        tokens.next();
        // take package name
        let pack_name = tokens.next().take().unwrap().take();
        // println!("*--- package {}", pack_name);
        // take the IS keyword
        if tokens
            .next()
            .take()
            .unwrap()
            .as_type()
            .check_keyword(&Keyword::Is)
            == false
        {
            // panic!("expecting keyword IS")
            return Err(VhdlError::Vague);
        }
        let (b_refs, _b_deps) = VhdlSymbol::parse_body(tokens, &Self::is_primary_ending, false);
        Ok(PackageBody::new(
            match pack_name {
                VhdlToken::Identifier(id) => id,
                _ => {
                    // panic!("expected an identifier")
                    return Err(VhdlError::Vague);
                }
            },
            b_refs,
            pos,
        ))
    }

    /// Detects identifiers configured in the configuration statement section or architecture
    /// declaration section.
    ///
    /// Assumes the first token to consume is 'for' and there is a ':' token to follow.
    fn parse_configuration_spec(statement: Statement) -> Option<RefSet> {
        let mut tokens = Self::statement_to_tokens(statement).into_iter().peekable();
        // force keyword 'for'
        if tokens.next()?.take().check_keyword(&Keyword::For) == false {
            return None;
        }
        // take tokens until ':'
        while let Some(tkn) = tokens.next() {
            if tkn.as_ref().check_delimiter(&Delimiter::Colon) == true {
                break;
            }
        }
        // take the component's name that is being replaced
        tokens.next()?.take().as_identifier()?;

        // take the keyword 'use'
        if tokens.next()?.take().check_keyword(&Keyword::Use) == false {
            return None;
        }

        // entity aspect
        // entity_aspect ::=
        //      entity entity_name [ ( architecture_identifier) ]
        //      | configuration configuration_name
        //      | open
        if tokens.peek()?.as_ref().check_keyword(&Keyword::Entity) == true
            || tokens
                .peek()?
                .as_ref()
                .check_keyword(&Keyword::Configuration)
                == true
        {
            // take keyword ENTITY or CONFIGURATION
            tokens.next();
            // take the compound identifiers
            let mut deps = RefSet::new();
            // take entity identifier
            deps.extend(Self::compose_name(&mut tokens).into_compound_identifiers(true));
            // take remaining possible references
            match Self::update_deps_from_statement(&mut deps, &mut tokens) {
                true => Some(deps),
                false => None,
            }
        } else {
            None
        }
    }

    /// Transforms a single statement into a list of tokens.
    ///
    /// Appends a terminator `;` to the end of the token list.
    fn statement_to_tokens(clause: Statement) -> Vec<Token<VhdlToken>> {
        let mut tokens = clause.0;
        // adds terminating delimiter ';'
        tokens.push(Token::new(
            VhdlToken::Delimiter(Delimiter::Terminator),
            Position::new(),
        ));
        tokens
    }

    fn into_terminated_stream(mut tokens: Vec<Token<VhdlToken>>) -> Vec<Token<VhdlToken>> {
        // adds terminating delimiter ';'
        tokens.push(Token::new(
            VhdlToken::Delimiter(Delimiter::Terminator),
            Position::new(),
        ));
        tokens
    }

    /// Updates a dependencies list `deps` by parsing the token stream.
    ///
    /// Assumes the token stream is built from a single statement. Assumes the first token in the stream is an
    /// indentifier that could be a dependency, if the first dependency detects is not a compound identifier with that matching suffix.
    ///
    /// Returns true if found no fault in the statement.
    fn update_deps_from_statement<I>(deps: &mut RefSet, tokens: &mut Peekable<I>) -> bool
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        // steal the '(' if it is next to prevent performing sensitivity list
        while let Some(t) = tokens.peek() {
            // println!("{:?}", t.as_type());
            if t.as_type().check_delimiter(&Delimiter::ParenL) == true {
                tokens.next();
            } else {
                break;
            }
        }
        deps.extend(Self::parse_statement(tokens).1);
        true
    }

    /// Detects identifiers instantiated in the architecture statement sections.
    ///
    /// Assumes the next token to consume is instance name of the instantiation and
    /// the token to follow is the COLON ':' delimiter.
    fn parse_instantiation(stream: Vec<Token<VhdlToken>>) -> Option<RefSet> {
        // println!("{:?}", stream);
        // force certain delimiters to not exist
        if stream
            .iter()
            .filter(|f| {
                f.as_ref().check_delimiter(&Delimiter::SigAssign)
                    || f.as_ref().check_delimiter(&Delimiter::VarAssign)
            })
            .count()
            > 0
        {
            return None;
        }
        let mut tokens = Self::into_terminated_stream(stream).into_iter().peekable();
        // force identifier (instance name)
        tokens.next()?.take().as_identifier()?;
        // force colon
        if tokens.next()?.take().check_delimiter(&Delimiter::Colon) == false {
            return None;
        };
        // check what is instantiated
        match tokens.peek()?.as_type() {
            VhdlToken::Identifier(_) => {
                let mut deps: HashSet<CompoundIdentifier> = RefSet::new();
                // take entity identifier
                deps.extend(Self::compose_name(&mut tokens).into_compound_identifiers(true));
                // make sure we valid continuing tokens
                if let Some(p) = tokens.peek() {
                    if p.as_type().check_delimiter(&Delimiter::ParenR) == true {
                        return None;
                    }
                    if p.as_type().check_delimiter(&Delimiter::Terminator) == true {
                        return None;
                    }
                    if p.as_type().check_delimiter(&Delimiter::VarAssign) == true {
                        return None;
                    }
                    if p.as_type().check_delimiter(&Delimiter::SigAssign) == true {
                        return None;
                    }
                }
                // take remaining possible references
                match Self::update_deps_from_statement(&mut deps, &mut tokens) {
                    true => Some(deps),
                    false => None,
                }
            }
            VhdlToken::Keyword(kw) => {
                if kw == &Keyword::Component
                    || kw == &Keyword::Entity
                    || kw == &Keyword::Configuration
                {
                    tokens.next();
                    match tokens.peek()?.as_type() {
                        VhdlToken::Identifier(_) => {
                            let mut deps = RefSet::new();
                            // take entity identifier
                            deps.extend(
                                Self::compose_name(&mut tokens).into_compound_identifiers(true),
                            );
                            // take remaining possible references
                            match Self::update_deps_from_statement(&mut deps, &mut tokens) {
                                true => Some(deps),
                                false => None,
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_configuration<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<VhdlSymbol, VhdlError>
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        let config_name = match tokens.next().take().unwrap().take() {
            VhdlToken::Identifier(id) => id,
            _ => {
                // panic!("expected an identifier")
                return Err(VhdlError::Vague);
            }
        };
        let entity_name = VhdlSymbol::parse_owner_design_unit(tokens)?;

        // force taking the `is` keyword
        if tokens.next().unwrap().as_type().check_keyword(&Keyword::Is) == false {
            // panic!("expecting keyword 'is'")
            return Err(VhdlError::Vague);
        }

        let mut deps = RefSet::new();
        let mut refs = RefSet::new();
        // parse configuration section
        while let Some(t) = tokens.peek() {
            if t.as_type().check_keyword(&Keyword::End) {
                let (clause, _c_refs) = Self::parse_statement(tokens);
                if Self::is_primary_ending(&clause) {
                    break;
                }
            // enter a block configuration
            } else if t.as_type().check_keyword(&Keyword::For) {
                // take the 'for' keyword
                tokens.next().unwrap();
                deps.extend(Self::parse_block_configuration(tokens));
            // @todo handle `use` clauses
            } else {
                refs.extend(Self::parse_statement(tokens).1);
            }
        }

        Ok(VhdlSymbol::Configuration(Configuration::new(
            config_name,
            entity_name,
            deps,
            refs,
            pos,
        )))
    }

    fn parse_block_configuration<I>(tokens: &mut Peekable<I>) -> RefSet
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        let mut refs = RefSet::new();
        // take the identifier
        tokens.next().unwrap();
        // if next token is '(', take until leveling out to ')'
        if tokens
            .peek()
            .unwrap()
            .as_type()
            .check_delimiter(&Delimiter::ParenL)
        {
            tokens.next();
            let mut balance = 1;
            while let Some(t) = tokens.next() {
                if t.as_ref().check_delimiter(&Delimiter::ParenL) == true {
                    balance += 1;
                } else if t.as_ref().check_delimiter(&Delimiter::ParenR) == true {
                    balance -= 1;
                }
                if balance == 0 {
                    break;
                }
            }
        }
        while let Some(t) = tokens.peek() {
            if t.as_type().check_keyword(&Keyword::End) {
                let (clause, _c_refs) = Self::parse_statement(tokens);
                // exit the block configuration
                if Self::is_sub_ending(&clause) {
                    break;
                }
            } else {
                // take configuration specification by composing statement
                let (clause, c_refs) = Self::parse_statement(tokens);
                if let Some(idens) = Self::parse_configuration_spec(clause) {
                    refs.extend(idens);
                    // take next `end for`
                    let _ending = Self::parse_statement(tokens);
                } else {
                    refs.extend(c_refs);
                }
            }
        }
        refs
    }

    /// Consumes tokens after the USE keyword.
    ///
    /// Assumes the last token consumed was USE and composes a statement of imports.
    ///
    /// @dead_code
    fn parse_use_clause<I>(tokens: &mut Peekable<I>) -> UseClause
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        // collect first selected_name
        let mut imports = Vec::new();
        imports.push(Self::compose_name(tokens));
        while let Some(t) = tokens.next() {
            // take the comma, then next selected name
            if t.as_type().check_delimiter(&Delimiter::Comma) {
                imports.push(Self::compose_name(tokens));
            }
            if t.as_type().check_delimiter(&Delimiter::Terminator) {
                break;
            }
        }
        UseClause { imports: imports }
    }

    /// Parses an secondary design unit: architecture.
    ///
    /// Assumes the next token to consume is the architecture's identifier.
    fn parse_architecture<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<VhdlSymbol, VhdlError>
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        let arch_name = match tokens.next().take().unwrap().take() {
            VhdlToken::Identifier(id) => id,
            _ => {
                // panic!("expected an identifier")
                return Err(VhdlError::Vague);
            }
        };
        let entity_name = VhdlSymbol::parse_owner_design_unit(tokens)?;
        // println!("*--- unit {}", arch_name);

        let (arch_refs, arch_deps) =
            VhdlSymbol::parse_declaration(tokens, &Self::is_primary_ending)?;
        Ok(VhdlSymbol::Architecture(Architecture::new(
            arch_name,
            entity_name,
            arch_refs,
            arch_deps,
            pos,
        )))
    }

    /// Checks if the statement `stmt` is the code to enter a valid sub-declaration section.
    fn is_sub_declaration(stmt: &Statement) -> bool {
        let first = match stmt.0.first() {
            Some(tk) => tk.as_ref(),
            None => return false,
        };
        // verify first keyword
        if let Some(keyword) = first.as_keyword() {
            match keyword {
                Keyword::Function | Keyword::Procedure => (),
                _ => return false,
            }
        } else {
            return false;
        };
        // verify last keyword is BEGIN
        let last = match stmt.0.last() {
            Some(tk) => tk.as_ref(),
            None => return false,
        };
        if let Some(keyword) = last.as_keyword() {
            keyword == &Keyword::Begin
        } else {
            false
        }
    }

    /// Parses together a series of tokens into a single `Statement`.
    ///
    /// Statements end on a ';' and do not include the ';' token. If the EOF
    /// is reached before completing a statement, it is omitted and a blank
    /// statement is returned.
    fn parse_statement<I>(tokens: &mut Peekable<I>) -> TokenPair
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        let mut clause = Statement::new();
        let mut refs: HashSet<CompoundIdentifier> = RefSet::new();

        // determine if the statement will be a sensitivity list
        let mut paren_count: i32 = 0;
        let is_sensitivity_list = {
            if let Some(t) = tokens.peek() {
                t.as_type().check_delimiter(&Delimiter::ParenL) == true
            } else {
                false
            }
        };

        // traverse through token stream
        while let Some(t) = tokens.next() {
            // gather sensitivity list as its own statement
            if is_sensitivity_list == true
                && (t.as_type().check_delimiter(&Delimiter::ParenL)
                    || t.as_type().check_delimiter(&Delimiter::ParenR))
            {
                clause.get_tokens_mut().push(t);
                if clause
                    .get_tokens()
                    .last()
                    .unwrap()
                    .as_type()
                    .check_delimiter(&Delimiter::ParenL)
                {
                    paren_count += 1;
                    // add token
                } else if clause
                    .get_tokens()
                    .last()
                    .unwrap()
                    .as_type()
                    .check_delimiter(&Delimiter::ParenR)
                {
                    paren_count -= 1;
                    if paren_count == 0 {
                        return (clause, refs);
                    }
                }
            // exit upon encountering terminator ';'
            } else if t.as_type().check_delimiter(&Delimiter::Terminator) {
                // println!("{:?}", clause);
                return (clause, refs);
            // extra keywords to help break up statements early
            } else if t.as_type().check_keyword(&Keyword::Generate)
                || t.as_type().check_keyword(&Keyword::Process)
                || t.as_type().check_keyword(&Keyword::Begin)
                || (clause.get_tokens().first().is_some()
                    && clause
                        .get_tokens()
                        .first()
                        .unwrap()
                        .as_type()
                        .check_keyword(&Keyword::When)
                    && t.as_type().check_delimiter(&Delimiter::Arrow))
            {
                // add the breaking token to the statement before exiting
                clause.get_tokens_mut().push(t);
                // println!("{:?}", clause);
                return (clause, refs);
            } else {
                // check for compound identifiers as references to other design units
                let mut took_dot: Option<Token<VhdlToken>> = None;
                if let Some(id) = t.as_type().as_identifier() {
                    // check if next token is a 'dot' delimiter
                    if tokens.peek().is_some()
                        && tokens
                            .peek()
                            .unwrap()
                            .as_type()
                            .check_delimiter(&Delimiter::Dot)
                    {
                        // consume the dot
                        took_dot = tokens.next();
                        // mark as a reference if the next token is an identifier
                        if tokens.peek().is_some() {
                            if let Some(id2) = tokens.peek().unwrap().as_type().as_identifier() {
                                // store the resource reference
                                refs.insert(CompoundIdentifier::new_vhdl(id.clone(), id2.clone()));
                            }
                        }
                    }
                }
                clause.get_tokens_mut().push(t);
                if let Some(dot) = took_dot {
                    clause.get_tokens_mut().push(dot);
                }
            }
        }
        // println!("{:?}", clause);
        // return empty statement if unable to close with terminator ';'
        (Statement::new(), RefSet::new())
    }

    /// Parses the OF keyword and then returns the following IDENTIFIER.
    ///
    /// The Identifier should correspond to the architecture's entity name.
    fn parse_owner_design_unit<I>(tokens: &mut Peekable<I>) -> Result<Identifier, VhdlError>
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        // force taking the 'of' keyword
        if tokens.next().unwrap().as_type().check_keyword(&Keyword::Of) == false {
            // panic!("expecting 'of' keyword")
            Err(VhdlError::Vague)
        } else {
            // return the name of the primary design unit
            match tokens.next().take().unwrap().take() {
                VhdlToken::Identifier(id) => Ok(id),
                // panic!("expected an identifier")
                _ => Err(VhdlError::Vague),
            }
        }
    }

    /// Parses a subprogram header, stopping at first occurence of IS or BEGIN keyword
    fn parse_subprogram_header<I>(_: &mut Peekable<I>) -> ()
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        // optionally take
        todo!("implement parsing for vhdl subprogram header");
    }

    /// Returns a list of interface items as `Statements`.
    ///
    /// Assumes the last token consumed was either GENERIC or PORT keywords and
    /// stops at the last statement in the respective list.
    fn parse_interface_list<I>(tokens: &mut Peekable<I>) -> Result<Vec<TokenPair>, VhdlError>
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        // expect the opening '('
        if tokens
            .next()
            .unwrap()
            .as_type()
            .check_delimiter(&Delimiter::ParenL)
            == false
        {
            // panic!("expecting '(' delimiter")
            return Err(VhdlError::Vague);
        }
        // collect statements until finding the ')', END, BEGIN, or PORT.
        let mut io: Vec<TokenPair> = Vec::new();
        while let Some(t) = tokens.peek() {
            if let Some((io_clause, _io_refs)) = io.last() {
                // safe to unwrap because of outer condition
                if io_clause
                    .get_tokens()
                    .last()
                    .unwrap()
                    .as_type()
                    .check_delimiter(&Delimiter::ParenR)
                {
                    // compute number of balancing LHS and RHS parentheses
                    let balance = io_clause.get_tokens().iter().fold(0, |acc, x| {
                        if x.as_type().check_delimiter(&Delimiter::ParenL) {
                            acc + 1
                        } else if x.as_type().check_delimiter(&Delimiter::ParenR) {
                            acc - 1
                        } else {
                            acc
                        }
                    });
                    // check if there is an uneven amount of RHS parentheses to indicate end of interface
                    if balance == -1 {
                        let index = io.len() - 1;
                        // pop off extra RHS parenthesis and exit
                        let (last_clause, _last_refs) = io.get_mut(index).unwrap();
                        if last_clause.get_tokens_mut().pop().is_none() == true {
                            // .expect("expecting closing ')'");
                            return Err(VhdlError::Vague);
                        }
                        break;
                    }
                }
            }
            // the RHS parenthesis is a single statement (';' used on last interface item)
            if t.as_type().check_delimiter(&Delimiter::ParenR) {
                Self::parse_statement(tokens);
                break;
            // collect statements
            } else {
                io.push(Self::parse_statement(tokens));
                // println!("{}", statements.last().unwrap());
            }
        }
        // println!("{:?}", statements);
        Ok(io)
    }

    /// Consumes tokens after `IS` until finding `BEGIN` or `END`.
    ///
    /// Assumes the next token to consume is `IS` and throws it away. This will
    /// search for interface lists found after GENERIC and PORT keywords.
    fn parse_entity_declaration<I>(
        tokens: &mut Peekable<I>,
    ) -> Result<(Vec<Statement>, Vec<Statement>, RefSet, RefSet), VhdlError>
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        // println!("*--- declaration section");
        // force taking the 'is' keyword
        if tokens.next().unwrap().as_type().check_keyword(&Keyword::Is) == false {
            // panic!("expecting 'is' keyword")
            return Err(VhdlError::Vague);
        }
        // check entity_header before entering entity declarative part
        // check for generics
        if tokens.peek().is_none() {
            // panic!("expecting END keyword")
            return Err(VhdlError::Vague);
        }
        let generics = if tokens
            .peek()
            .unwrap()
            .as_type()
            .check_keyword(&Keyword::Generic)
        {
            tokens.next();
            Self::parse_interface_list(tokens)?
        } else {
            Vec::new()
        };
        // check for ports
        if tokens.peek().is_none() {
            // panic!("expecting END keyword")
            return Err(VhdlError::Vague);
        }
        let ports = if tokens
            .peek()
            .unwrap()
            .as_type()
            .check_keyword(&Keyword::Port)
        {
            tokens.next();
            Self::parse_interface_list(tokens)?
        } else {
            Vec::new()
        };

        let mut entity_refs = RefSet::new();
        let mut entity_deps = RefSet::new();

        let ports = ports
            .into_iter()
            .map(|f| {
                entity_refs.extend(f.1);
                f.0
            })
            .collect();
        let generics = generics
            .into_iter()
            .map(|f| {
                entity_refs.extend(f.1);
                f.0
            })
            .collect();

        while let Some(t) = tokens.peek() {
            // stop the declaration section and enter a statement section
            if t.as_type().check_keyword(&Keyword::Begin) {
                tokens.next();
                let (b_refs, b_deps) = Self::parse_body(tokens, &Self::is_primary_ending, false);
                entity_refs.extend(b_refs);
                entity_deps.extend(b_deps);
                break;
            // the declaration is over and there is no statement section
            } else if t.as_type().check_keyword(&Keyword::End) {
                let (clause, _c_refs) = Self::parse_statement(tokens);
                if Self::is_primary_ending(&clause) {
                    break;
                }
            // find a nested package (throw away for now)
            } else if t.as_type().check_keyword(&Keyword::Package) {
                let inner_pos = tokens.next().unwrap().into_position();
                let pack_name = Self::route_package_parse(tokens, inner_pos)?;
                // add references found from the package
                entity_refs.extend(pack_name.into_package().unwrap().into_refs());
            // build statements to throw away
            } else {
                entity_refs.extend(Self::parse_statement(tokens).1);
            }
        }
        Ok((generics, ports, entity_refs, entity_deps))
    }

    /// Checks if the keyword `kw` is a potential start to a subprogram.
    fn is_subprogram(kw: &Keyword) -> bool {
        match kw {
            Keyword::Function | Keyword::Procedure | Keyword::Pure | Keyword::Impure => true,
            _ => false,
        }
    }

    /// Checks if the statement `stmt` is the ending to a subprogram.
    fn is_subprogram_ending(stmt: &Statement) -> bool {
        // verify the first token is the 'end' keyword
        match stmt.0.get(0) {
            Some(i) => match i.as_ref().as_keyword() {
                Some(kw) => {
                    if kw != &Keyword::End {
                        return false;
                    }
                }
                None => return false,
            },
            None => return false,
        }
        // verify the next token is the keyword for subprogram
        match stmt.0.get(1) {
            Some(i) => match i.as_ref().as_keyword() {
                Some(kw) => match kw {
                    Keyword::Procedure | Keyword::Function => true,
                    _ => false,
                },

                None => true,
            },
            None => true,
        }
    }

    /// Parses through a subprogram (procedure or function).
    ///
    /// Returns (`deps`, `refs`).
    fn parse_subprogram<I>(tokens: &mut Peekable<I>) -> Result<(RefSet, RefSet), VhdlError>
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        let mut refs = RefSet::new();
        let mut deps = RefSet::new();

        let mut is_return_yet: bool = false;
        while let Some(t) = tokens.peek() {
            // trigger when the statement will be the end of the declaration line
            if t.as_type().check_keyword(&Keyword::Return) == true {
                is_return_yet = true;
            }
            // determine when to branch to declaration section or body section
            if t.as_type().check_keyword(&Keyword::Is) {
                // println!("OUT SUB: {:?}", t);
                let (inner_refs, inner_deps) =
                    Self::parse_declaration(tokens, &Self::is_subprogram_ending)?;
                refs.extend(inner_refs);
                deps.extend(inner_deps);
                break;
            } else if is_return_yet && t.as_type().check_delimiter(&Delimiter::Terminator) {
                // println!("OUT SUB: {:?}", t);
                break;
            } else {
                // println!("IN SUB: {:?}", t);
                tokens.next();
            }
        }
        // @todo: capture references from declaration and body sections
        Ok((refs, deps))
    }

    /// Consumes tokens after `IS` until finding `BEGIN` or `END`.
    ///
    /// Assumes the next token to consume is `IS` and throws it away.
    ///
    /// Returns (`deps`, `refs`)
    fn parse_declaration<I>(
        tokens: &mut Peekable<I>,
        eval_exit: &dyn Fn(&Statement) -> bool,
    ) -> Result<(RefSet, RefSet), VhdlError>
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        // println!("*--- declaration section");
        // force taking the 'is' keyword
        if tokens.next().unwrap().as_type().check_keyword(&Keyword::Is) == false {
            // panic!("expecting 'is' keyword")
            return Err(VhdlError::Vague);
        }
        let mut refs = RefSet::new();
        let mut deps = RefSet::new();

        while let Some(t) = tokens.peek() {
            // println!("dec: {:?}", t);
            // stop the declaration section and enter a statement section
            if t.as_type().check_keyword(&Keyword::Begin) {
                tokens.next();
                // combine refs from declaration and from body
                let (body_refs, body_deps) = Self::parse_body(tokens, &eval_exit, false);
                refs.extend(body_refs);
                deps.extend(body_deps);

                // println!("{}", "stop reading tokens");
                // STOP READING TOKENS
                break;
            // the declaration is over and there is no statement section
            } else if t.as_type().check_keyword(&Keyword::End) {
                let (clause, _c_refs) = Self::parse_statement(tokens);
                // println!("end of dec: {:?}", stmt);
                if eval_exit(&clause) {
                    break;
                }
            // find component names (could be in package or architecture declaration)
            } else if t.as_type().check_keyword(&Keyword::Component) {
                let _comp_name = Self::parse_component(tokens);
                // println!("**** INFO: Found component: \"{}\"", comp_name);

                // find a nested package
            } else if t.as_type().check_keyword(&Keyword::Package) {
                let inner_pos = tokens.next().unwrap().into_position();
                let _pack_name = Self::route_package_parse(tokens, inner_pos);
                // println!("**** INFO: detected nested package \"{}\"", pack_name);

                // detect subprograms
            } else if t.as_type().as_keyword().is_some()
                && Self::is_subprogram(t.as_type().as_keyword().unwrap()) == true
            {
                // println!("{}", "sub program");
                let (sub_refs, sub_deps) = Self::parse_subprogram(tokens)?;
                refs.extend(sub_refs);
                deps.extend(sub_deps);
            // build statements to throw away
            } else {
                let (clause, res_refs) = Self::parse_statement(tokens);
                // add resource references
                // println!("st {:?}", stmt);
                // check if using a configuration specification
                if let Some(i_refs) = Self::parse_configuration_spec(clause) {
                    refs.extend(i_refs);
                } else {
                    refs.extend(res_refs);
                }
            }
        }
        Ok((refs, deps))
    }

    /// Checks if the statement is a valid primary unit END statement.
    ///
    /// Rejects inferior END statements that require an explicit keyword to following
    /// the END keyword. Primary END statements follow: `END [keyword] [identifier];`.
    fn is_primary_ending(stmt: &Statement) -> bool {
        let keyword = if let Some(t) = stmt.0.get(1) {
            t.as_type()
        } else {
            return true; // only having "end;" is valid
        };
        match keyword {
            // list mandatory keywords expected after the 'end' keyword for non-primary endings
            VhdlToken::Keyword(kw) => match kw {
                Keyword::Loop
                | Keyword::Generate
                | Keyword::Process
                | Keyword::Postponed
                | Keyword::If
                | Keyword::Block
                | Keyword::Protected
                | Keyword::Record
                | Keyword::Case
                | Keyword::Component
                | Keyword::For => false,
                _ => true,
            },
            _ => true,
        }
    }

    /// Checks if the statement is a valid non-primary unit END statement.
    ///
    /// This is the negation of `is_primary_ending`.
    fn is_sub_ending(stmt: &Statement) -> bool {
        !Self::is_primary_ending(stmt)
    }

    /// Checks if the keyword indicates a subprogram statement.
    fn enter_subprogram(kw: &Keyword) -> bool {
        match kw {
            Keyword::Function | Keyword::Procedure | Keyword::Impure | Keyword::Pure => true,
            _ => false,
        }
    }

    /// Parses a component declaration, consuming the tokens `COMPONENT` until the end.
    ///
    /// Assumes the first token to consume is `COMPONENT`.
    fn parse_component<I>(tokens: &mut Peekable<I>) -> Result<CompoundIdentifier, VhdlError>
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        if tokens
            .next()
            .unwrap()
            .as_type()
            .check_keyword(&Keyword::Component)
            == false
        {
            // panic!("assumes first token is COMPONENT keyword");
            return Err(VhdlError::Vague);
        }
        // take component name
        let comp_name = tokens.next().take().unwrap().take();
        // println!("*--- found component {}", comp_name);
        // take 'is' keyword (optional)
        if tokens.peek().unwrap().as_type().check_keyword(&Keyword::Is) {
            tokens.next();
        }
        // collect port names and generic names until hitting 'END'
        while let Some(t) = tokens.peek() {
            if t.as_type().check_keyword(&Keyword::End) {
                let _stmt = Self::parse_statement(tokens);
                break;
            // collect generic interface
            } else if t.as_type().check_keyword(&Keyword::Generic) {
                // take the GENERIC token
                tokens.next();
                let _generics = Self::parse_interface_list(tokens)?;
            // collect port interface
            } else if t.as_type().check_keyword(&Keyword::Port) {
                // take the PORT token
                tokens.next();
                let _ports = Self::parse_interface_list(tokens)?;
            } else {
                let _stmt = Self::parse_statement(tokens);
                // println!("{:?}", stmt);
            }
        }
        match comp_name {
            VhdlToken::Identifier(id) => Ok(CompoundIdentifier::new_minimal_vhdl(id)),
            _ => {
                // panic!("expected an identifier")
                Err(VhdlError::Vague)
            }
        }
    }

    /// Routes the parsing to either package body or package declaration,
    /// depending on the next token being BODY keyword or identifier.
    fn route_package_parse<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<VhdlSymbol, VhdlError>
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        if &VhdlToken::Keyword(Keyword::Body) == tokens.peek().unwrap().as_type() {
            Ok(VhdlSymbol::PackageBody(VhdlSymbol::parse_package_body(
                tokens, pos,
            )?))
        } else {
            Ok(VhdlSymbol::parse_package_declaration(tokens, pos)?)
        }
    }

    /// Parses a body, consuming tokens from `BEGIN` until `END`.
    ///
    /// Builds statements and stops after finding the `END` keyword statement. If
    /// the `END` keyword statement is detected, it will have to pass the `eval_exit`
    /// function to properly exit scope. Assumes the last token consumed was `BEGIN`.
    fn parse_body<I>(
        tokens: &mut Peekable<I>,
        eval_exit: &dyn Fn(&Statement) -> bool,
        is_subprogram: bool,
    ) -> (RefSet, RefSet)
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        // collect names that were referenced
        let mut refs = RefSet::new();
        // collect component names
        let mut deps = RefSet::new();
        // println!("*--- statement section");
        while let Some(t) = tokens.peek() {
            if t.as_type().check_keyword(&Keyword::End) == true {
                let (clause, _c_refs) = Self::parse_statement(tokens);
                // println!("IN BODY AT END: {:?}", stmt);
                if eval_exit(&clause) == true {
                    break;
                }
            // enter a subprogram
            } else if t.as_type().check_keyword(&Keyword::Function)
                || t.as_type().check_keyword(&Keyword::Begin)
                || t.as_type().check_keyword(&Keyword::Procedure)
            {
                let is_subprogram = Self::is_subprogram(t.as_type().as_keyword().unwrap());
                let next_eval_exit = match is_subprogram {
                    true => Self::is_subprogram_ending,
                    false => Self::is_sub_ending,
                };
                let (_clause, c_refs) = Self::parse_statement(tokens);
                // println!("ENTERING SUBPROGRAM {:?}", _clause);
                // catch any references in the given statement
                refs.extend(c_refs);

                // println!("REFS BEFORE: {:?}", refs);
                let (inner_refs, inner_deps) =
                    Self::parse_body(tokens, &next_eval_exit, is_subprogram);
                // println!("DEPS: {:?}", inner_deps);
                // update any references caught
                refs.extend(inner_refs);
                deps.extend(inner_deps);
                // println!("REFS AFTER: {:?}", refs);
                // println!("EXITING SUBPROGRAM");
                // find component names (could be in package)
            } else if t.as_type().check_keyword(&Keyword::Component) {
                let _comp_name = Self::parse_component(tokens);
                // println!("**** INFO: Found component: \"{}\"", comp_name);
                // find packages
            } else if t.as_type().check_keyword(&Keyword::Package) {
                let inner_pos = tokens.next().unwrap().into_position();
                let _symbol = Self::route_package_parse(tokens, inner_pos);
                // println!("**** INFO: Detected nested package \"{}\"", symbol);
                // build statements
            } else {
                let (clause, c_refs) = Self::parse_statement(tokens);
                // println!("IN BODY: {:?}", clause);
                refs.extend(c_refs);
                // check if statement is an instantiation
                if is_subprogram == false {
                    if let Some(i_refs) = Self::parse_instantiation(clause.0) {
                        // println!("info: detected dependency \"{:?}\"", i_refs);
                        deps.extend(i_refs);
                    }
                }
            }
        }
        // println!("DEPS: {:?}", deps);
        // println!("REFS: {:?}", refs);
        (refs, deps)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::lang::vhdl::token::literal::*;

    #[test]
    fn parse_use_clause() {
        let s = "use eel4712c.pkg1, eel4712c.pkg2; entity";
        let mut tokens = VhdlTokenizer::from_source_code(&s)
            .into_tokens()
            .into_iter()
            .peekable();
        // take USE
        tokens.next();
        let using_imports = VhdlSymbol::parse_use_clause(&mut tokens);
        assert_eq!(
            using_imports,
            UseClause {
                imports: vec![
                    SelectedName(vec![
                        Identifier::Basic("eel4712c".to_owned()),
                        Identifier::Basic("pkg1".to_owned()),
                    ]),
                    SelectedName(vec![
                        Identifier::Basic("eel4712c".to_owned()),
                        Identifier::Basic("pkg2".to_owned()),
                    ]),
                ]
            }
        );
        assert_eq!(
            tokens.next().unwrap().as_type(),
            &VhdlToken::Keyword(Keyword::Entity)
        );
    }

    #[test]
    fn compose_statement_2() {
        let s = "P1, P2: inout BIT); constant Delay: TIME := 1 ms;";
        let tokens = VhdlTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        assert_eq!(
            VhdlSymbol::parse_statement(&mut iter).0.as_types(),
            vec![
                &VhdlToken::Identifier(Identifier::Basic("P1".to_owned())),
                &VhdlToken::Delimiter(Delimiter::Comma),
                &VhdlToken::Identifier(Identifier::Basic("P2".to_owned())),
                &VhdlToken::Delimiter(Delimiter::Colon),
                &VhdlToken::Keyword(Keyword::Inout),
                &VhdlToken::Identifier(Identifier::Basic("BIT".to_owned())),
                &VhdlToken::Delimiter(Delimiter::ParenR),
            ]
        );
        assert_eq!(
            iter.next().unwrap().as_type(),
            &VhdlToken::Keyword(Keyword::Constant)
        );
    }

    #[test]
    fn parse_simple_name() {
        let s = "eel4712c.nor_gate port";
        let mut tokens = VhdlTokenizer::from_source_code(&s)
            .into_tokens()
            .into_iter()
            .peekable();
        let sel_name = VhdlSymbol::compose_name(&mut tokens);
        assert_eq!(
            sel_name,
            SelectedName(vec![
                Identifier::Basic("eel4712c".to_owned()),
                Identifier::Basic("nor_gate".to_owned()),
            ])
        );
        assert_eq!(
            tokens.next().unwrap().as_type(),
            &VhdlToken::Keyword(Keyword::Port)
        );
    }

    #[test]
    fn parse_simple_name_with_all() {
        // @TODO signify within a 'name' struct that the all keyword was used
        let s = "eel4712c.all +";
        let mut tokens = VhdlTokenizer::from_source_code(&s)
            .into_tokens()
            .into_iter()
            .peekable();
        let sel_name = VhdlSymbol::compose_name(&mut tokens);
        assert_eq!(
            sel_name,
            SelectedName(vec![Identifier::Basic("eel4712c".to_owned()),])
        );
        assert_eq!(
            tokens.next().unwrap().as_type(),
            &VhdlToken::Delimiter(Delimiter::Plus)
        );
    }

    #[test]
    fn parse_port_interface_difficult_ending() {
        let s = "\
port (P1, P2: inout BIT); 
constant Delay: TIME := 1 ms;";
        let mut tokens = VhdlTokenizer::from_source_code(&s)
            .into_tokens()
            .into_iter()
            .peekable();
        tokens.next(); // take PORT
        let ports = VhdlSymbol::parse_interface_list(&mut tokens).unwrap();
        let ports: Vec<String> = ports.into_iter().map(|m| m.0.to_string()).collect();
        assert_eq!(ports, vec!["P1 , P2 : inout BIT",]);
        assert_eq!(
            tokens.next().unwrap().as_type(),
            &VhdlToken::Keyword(Keyword::Constant)
        );
    }

    #[test]
    fn parse_ports_both() {
        let s = "\
generic ( N: positive );
port(
    a: in  std_logic_vector(N-1 downto 0);
    b: in  std_logic_vector(N-1 downto 0);
    c: out std_logic_vector(N-1 downto 0)
);
end;";
        let mut tokens = VhdlTokenizer::from_source_code(&s)
            .into_tokens()
            .into_iter()
            .peekable();
        tokens.next(); // take GENERIC
        let generics = VhdlSymbol::parse_interface_list(&mut tokens).unwrap();
        // convert to strings for easier verification
        let generics: Vec<String> = generics.into_iter().map(|m| m.0.to_string()).collect();
        assert_eq!(generics, vec!["N : positive",]);
        // take PORT
        tokens.next();
        let ports = VhdlSymbol::parse_interface_list(&mut tokens).unwrap();
        // convert to strings for easier verification
        let ports: Vec<String> = ports.into_iter().map(|m| m.0.to_string()).collect();
        assert_eq!(
            ports,
            vec![
                "a : in std_logic_vector(N - 1 downto 0)",
                "b : in std_logic_vector(N - 1 downto 0)",
                "c : out std_logic_vector(N - 1 downto 0)",
            ]
        );
        assert_eq!(
            tokens.next().unwrap().as_type(),
            &VhdlToken::Keyword(Keyword::End)
        );
    }

    #[test]
    fn parse_generics_only() {
        let s = "\
generic ( N: positive );
begin
end;";
        let mut tokens = VhdlTokenizer::from_source_code(&s)
            .into_tokens()
            .into_iter()
            .peekable();
        tokens.next(); // take GENERIC
        let generics = VhdlSymbol::parse_interface_list(&mut tokens).unwrap();
        // convert to strings for easier verification
        let generics: Vec<String> = generics.into_iter().map(|m| m.0.to_string()).collect();
        assert_eq!(generics, vec!["N : positive",]);
        assert_eq!(
            tokens.next().unwrap().as_type(),
            &VhdlToken::Keyword(Keyword::Begin)
        );
    }

    #[test]
    fn parse_component() {
        // ends with 'end component nor_gate;' Statement
        let s = "\
component nor_gate is end component nor_gate;

signal ready: std_logic;";
        let mut tokens = VhdlTokenizer::from_source_code(&s)
            .into_tokens()
            .into_iter()
            .peekable();
        let comp = VhdlSymbol::parse_component(&mut tokens).unwrap();
        assert_eq!(comp.to_string(), "nor_gate");
        assert_eq!(
            tokens.next().unwrap().as_type(),
            &VhdlToken::Keyword(Keyword::Signal)
        );

        // ends with 'end;' statement
        let s = "\
component nor_gate end;

signal ready: std_logic;";
        let mut tokens = VhdlTokenizer::from_source_code(&s)
            .into_tokens()
            .into_iter()
            .peekable();
        let comp = VhdlSymbol::parse_component(&mut tokens).unwrap();
        assert_eq!(comp.to_string(), "nor_gate");
        assert_eq!(
            tokens.next().unwrap().as_type(),
            &VhdlToken::Keyword(Keyword::Signal)
        );

        // ends with 'end component nor_gate;' statement
        let s = "\
-- declare DUT component
component nor_gate 
    generic( N: positive );
    port(
        a: in  std_logic_vector(N-1 downto 0);
        b: in  std_logic_vector(N-1 downto 0);
        c: out std_logic_vector(N-1 downto 0)
    );
end component nor_gate;

signal ready: std_logic;";
        let mut tokens = VhdlTokenizer::from_source_code(&s)
            .into_tokens()
            .into_iter()
            .peekable();
        let comp = VhdlSymbol::parse_component(&mut tokens).unwrap();
        assert_eq!(comp.to_string(), "nor_gate");
        assert_eq!(
            tokens.next().unwrap().as_type(),
            &VhdlToken::Keyword(Keyword::Signal)
        );
    }

    use super::super::super::lexer::Position;

    #[test]
    fn is_primary_ending() {
        // 'case' is not a primary ending
        let stmt = Statement(vec![
            Token::new(VhdlToken::Keyword(Keyword::End), Position::new()),
            Token::new(VhdlToken::Keyword(Keyword::Case), Position::new()),
            Token::new(
                VhdlToken::Identifier(Identifier::Basic("case_label".to_owned())),
                Position::new(),
            ),
        ]);
        assert_eq!(VhdlSymbol::is_primary_ending(&stmt), false);

        // primary endings can omit keyword and identifier label
        let stmt = Statement(vec![Token::new(
            VhdlToken::Keyword(Keyword::End),
            Position::new(),
        )]);
        assert_eq!(VhdlSymbol::is_primary_ending(&stmt), true);

        // primary endings can include their keyword
        let stmt = Statement(vec![
            Token::new(VhdlToken::Keyword(Keyword::End), Position::new()),
            Token::new(VhdlToken::Keyword(Keyword::Architecture), Position::new()),
            Token::new(
                VhdlToken::Identifier(Identifier::Basic("architecture_name".to_owned())),
                Position::new(),
            ),
        ]);
        assert_eq!(VhdlSymbol::is_primary_ending(&stmt), true);

        // primary endings can have their keyword omitted and also include the identifier label
        let stmt = Statement(vec![
            Token::new(VhdlToken::Keyword(Keyword::End), Position::new()),
            Token::new(
                VhdlToken::Identifier(Identifier::Basic("architecture_name".to_owned())),
                Position::new(),
            ),
        ]);
        assert_eq!(VhdlSymbol::is_primary_ending(&stmt), true);
    }

    #[test]
    fn entity() {
        let s = "\
 nor_gate is
    generic(
        N: positive
    );
    port(
        a: in  std_logic_vector(N-1 downto 0);
        b: in  std_logic_vector(N-1 downto 0);
        c: out std_logic_vector(N-1 downto 0)
    );
end entity nor_gate;";
        let mut tokens = VhdlTokenizer::from_source_code(&s)
            .into_tokens()
            .into_iter()
            .peekable();
        let e = Entity::from_tokens(&mut tokens, Position::place(1, 2)).unwrap();
        assert_eq!(e.get_position(), &Position::place(1, 2));
        assert_eq!(e.get_name(), &Identifier::Basic(String::from("nor_gate")));
        assert_eq!(e.get_generics().0.len(), 1);
        assert_eq!(e.get_ports().0.len(), 3);
    }

    use std::str::FromStr;

    #[test]
    fn resource_refs() {
        let s = "a : in std_logic_vector(3 downto 0) := work.pack1.p1;";
        let tokens = VhdlTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        assert_eq!(
            VhdlSymbol::parse_statement(&mut iter).1,
            RefSet::from([
                CompoundIdentifier::new_vhdl(
                    Identifier::from_str("work").unwrap(),
                    Identifier::from_str("pack1").unwrap()
                ),
                CompoundIdentifier::new_vhdl(
                    Identifier::from_str("pack1").unwrap(),
                    Identifier::from_str("p1").unwrap()
                ),
            ])
        );

        let s = "use work.package_name;";
        let tokens = VhdlTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        assert_eq!(
            VhdlSymbol::parse_statement(&mut iter).1,
            RefSet::from([CompoundIdentifier::new_vhdl(
                Identifier::from_str("work").unwrap(),
                Identifier::from_str("package_name").unwrap()
            ),])
        );

        let s = "use MKS.MEASUREMENTS, STD.STANDARD;";
        let tokens = VhdlTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        assert_eq!(
            VhdlSymbol::parse_statement(&mut iter).1,
            RefSet::from([
                CompoundIdentifier::new_vhdl(
                    Identifier::from_str("MKS").unwrap(),
                    Identifier::from_str("MEASUREMENTS").unwrap()
                ),
                CompoundIdentifier::new_vhdl(
                    Identifier::from_str("STD").unwrap(),
                    Identifier::from_str("STANDARD").unwrap()
                ),
            ])
        );
    }

    #[test]
    fn compose_statement() {
        let s = "a : in std_logic_vector(3 downto 0);";
        let tokens = VhdlTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        assert_eq!(
            VhdlSymbol::parse_statement(&mut iter).0.as_types(),
            vec![
                &VhdlToken::Identifier(Identifier::Basic("a".to_owned())),
                &VhdlToken::Delimiter(Delimiter::Colon),
                &VhdlToken::Keyword(Keyword::In),
                &VhdlToken::Identifier(Identifier::Basic("std_logic_vector".to_owned())),
                &VhdlToken::Delimiter(Delimiter::ParenL),
                &VhdlToken::AbstLiteral(AbstLiteral::Decimal("3".to_owned())),
                &VhdlToken::Keyword(Keyword::Downto),
                &VhdlToken::AbstLiteral(AbstLiteral::Decimal("0".to_owned())),
                &VhdlToken::Delimiter(Delimiter::ParenR),
            ]
        );

        let s = "a : in std_logic_vector(3 downto 0); ready: out std_logic);";
        let tokens = VhdlTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        assert_eq!(
            VhdlSymbol::parse_statement(&mut iter).0.as_types(),
            vec![
                &VhdlToken::Identifier(Identifier::Basic("a".to_owned())),
                &VhdlToken::Delimiter(Delimiter::Colon),
                &VhdlToken::Keyword(Keyword::In),
                &VhdlToken::Identifier(Identifier::Basic("std_logic_vector".to_owned())),
                &VhdlToken::Delimiter(Delimiter::ParenL),
                &VhdlToken::AbstLiteral(AbstLiteral::Decimal("3".to_owned())),
                &VhdlToken::Keyword(Keyword::Downto),
                &VhdlToken::AbstLiteral(AbstLiteral::Decimal("0".to_owned())),
                &VhdlToken::Delimiter(Delimiter::ParenR),
            ]
        );

        let s = "process(all) is begin end process;";
        let mut tokens = VhdlTokenizer::from_source_code(&s)
            .into_tokens()
            .into_iter()
            .peekable();
        let _ = VhdlSymbol::parse_statement(&mut tokens);
        assert_eq!(
            tokens.next().unwrap().as_type(),
            &VhdlToken::Delimiter(Delimiter::ParenL)
        );
        let _ = VhdlSymbol::parse_statement(&mut tokens);
        assert_eq!(
            tokens.next().unwrap().as_type(),
            &VhdlToken::Keyword(Keyword::End)
        );
    }

    #[test]
    fn print_statement() {
        let s = "a : in std_logic_vector ( 3 downto 0);";
        let tokens = VhdlTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        let st = VhdlSymbol::parse_statement(&mut iter);
        assert_eq!(st.0.to_string(), "a : in std_logic_vector(3 downto 0)");
    }

    #[test]
    fn configuration() {
        let s = r#"
use work.all;

configuration HA_Config of HA_Entity is
    for HA_Arch
        for HA_Inst : HA_Comp
            use entity HA_Comp_Entity(HA_Comp_Arch_1);
        end for;
        use work.some_pkg.all;
        for HA_Inst : HA_Comp
            use entity HA_Comp_Entity2(HA_Comp_Arch_1);
        end for;
    end for;
end HA_Config;    
"#;
        let symbols = VHDLParser::parse(VhdlTokenizer::from_source_code(&s).into_tokens());
        assert_eq!(
            symbols
                .first()
                .unwrap()
                .as_ref()
                .unwrap()
                .as_ref()
                .as_configuration()
                .unwrap()
                .edges(),
            &RefSet::from([
                CompoundIdentifier::new_minimal_vhdl(Identifier::Basic(String::from(
                    "HA_Comp_Entity"
                ))),
                CompoundIdentifier::new_vhdl(
                    Identifier::Basic(String::from("work")),
                    Identifier::Basic(String::from("some_pkg"))
                ),
                CompoundIdentifier::new_minimal_vhdl(Identifier::Basic(String::from(
                    "HA_Comp_Entity2"
                )))
            ])
        );
    }

    #[test]
    fn configuration_spec() {
        let s = r#"
for L1: XOR_GATE use entity WORK.XOR_GATE(Behavior) -- or L1 = 'others' = 'L1, L2, ...' = 'all'
        generic map (3 ns, 3 ns)
        port map (I1 => work.lab1_pkg.MAGIC_NUM, I2 => I2, O => O);    
"#;
        let tokens = VhdlTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        let st = VhdlSymbol::parse_statement(&mut iter);
        println!("{}", st.0);
        let iden = VhdlSymbol::parse_configuration_spec(st.0);
        assert_eq!(
            iden.unwrap(),
            RefSet::from([
                CompoundIdentifier::new_vhdl(
                    Identifier::Basic(String::from("WORK")),
                    Identifier::Basic(String::from("XOR_GATE"))
                ),
                CompoundIdentifier::new_minimal_vhdl(Identifier::Basic(String::from("XOR_GATE"))),
                CompoundIdentifier::new_vhdl(
                    Identifier::Basic(String::from("work")),
                    Identifier::Basic(String::from("lab1_pkg"))
                ),
                CompoundIdentifier::new_vhdl(
                    Identifier::Basic(String::from("lab1_pkg")),
                    Identifier::Basic(String::from("MAGIC_NUM"))
                ),
            ])
        );

        let s = r#"
for all: xor_gate use configuration cfg1;    
"#;
        let tokens = VhdlTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        let st = VhdlSymbol::parse_statement(&mut iter);
        let iden = VhdlSymbol::parse_configuration_spec(st.0);
        assert_eq!(
            iden.unwrap(),
            RefSet::from([CompoundIdentifier::new_minimal_vhdl(Identifier::Basic(
                String::from("cfg1")
            ))])
        );
    }

    #[test]
    fn playground_fn_in_arch_dec() {
        let s = r#"
--------------------------------------------------------------------------------
--! Project   : crus.eel5721.lab1
--! Engineer  : Chase Ruskin
--! Created   : 2022-09-06
--! Testbench : fib_tb
--! Details   :
--!     Tests the `fib` design entity for generating fibonacci sequence values.
--!     
--!     The range of values tested are 0 <= n < 47. 47 is omitted from the range
--!     due to complications loading the resulting value into a 32-bit vector.
--!
--!     Uses file i/o to load a text file "inputs.dat" of randomized `n` values 
--!     for input to be checked against the expected results for fib(n) loaded 
--!     from another text file "outputs.dat".
--------------------------------------------------------------------------------
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;
use std.textio.all;

entity fib_tb is 
end entity fib_tb;


architecture sim of fib_tb is
    --! internal testbench signals
    signal clk: std_logic := '0';
    signal rst: std_logic := '0';
    -- the simulation will halt when `halt` becomes '1'
    signal halt: std_logic := '0';

    constant period: time := 10 ns;

    --! unit-under-test (UUT) interface wires
    signal go     : std_logic;
    signal n      : std_logic_vector(5 downto 0);
    signal result : std_logic_vector(31 downto 0);
    signal done   : std_logic;

    impure function read_int_to_logic_vector(
        file f : text;
        len : in positive
    )
    return std_logic_vector is
        variable text_line : line;
        variable text_int  : integer;
    begin
        readline(f, text_line);
        read(text_line, text_int);
        return std_logic_vector(to_unsigned(text_int, len));
    end function;

begin
    --! UUT instantiation
    UUT : entity work.fib
    port map (
        clk    => clk,
        rst    => rst,
        go     => go,
        n      => n,
        result => result,
        done   => done
    );

    --! generate clock with 50% duty cycle
    clk <= not clk after period/2 when halt = '0';

    --! initial reset to start from known state
    boot: process
    begin
        rst <= '1';
        wait for period*4;
        rst <= '0';
        wait;
    end process;

    --! feed inputs into the UUT to begin processing
    input: process
        file inputs: text open read_mode is "inputs.dat";
        variable text_line : line;
        variable text_int  : integer;
    begin
        wait until rst = '0';

        while not endfile(inputs) loop
            -- drive the current input `n`
            readline(inputs, text_line);
            read(text_line, text_int);
            n <= std_logic_vector(to_unsigned(text_int, 6));

            -- assert the go signal to begin the simulation
            go <= '1';
            wait until rising_edge(clk);
            go <= '0';
            n <= (others => '0');
            wait until rising_edge(clk);

            -- assert that done has been set low
            assert done = '0' report "Done is not lowered during computation." severity error;

            wait until done = '1';

            -- allow 2 cycles to check done remaining asserted in bench process
            wait until rising_edge(clk);
            wait until rising_edge(clk);
        end loop;
        wait;
    end process;

    --! assert the received outputs match expected model values
    bench: process
        file outputs: text open read_mode is "outputs.dat";
        variable text_line    : line;
        variable text_int     : integer;
        variable ideal_result : std_logic_vector(31 downto 0);
        variable ideal_result_us : unsigned(31 downto 0);
    begin
        wait until rst = '0';

        while not endfile(outputs) loop
            readline(outputs, text_line);
            read(text_line, text_int);
            ideal_result := std_logic_vector(to_unsigned(text_int, 32));

            -- wait until done is asserted
            wait until done = '1';

            assert ideal_result = result report "Mismatched result." severity error;
            
            -- assert done remains '1'
            wait until rising_edge(clk);
            assert done = '1' report "Done is lowered too soon." severity error;        
        end loop;
        -- stop the simulation
        halt <= '1';
        report "Simulation complete.";
        wait;
    end process;

end architecture sim;
        "#;

        let syms = VHDLParser::parse(VhdlTokenizer::from_source_code(&s).into_tokens());
        println!("{:?}", syms);
        assert_eq!(
            syms.get(1)
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
                .as_ref()
                .as_architecture()
                .unwrap()
                .edges()
                .len(),
            1
        );
    }

    #[test]
    #[ignore]
    fn parse_basic() {
        let s = "\
-- design file for a nor_gate
library ieee;
use ieee.std_logic_1164.all;

entity nor_gate is -- comment on this line
    generic(
        N: positive
    );
    port(
        a : in std_logic_vector(N-1 downto 0);
        b : in std_logic_vector(N-1 downto 0);
        c : out std_logic_vector(N-1 downto 0)
    );
begin
end entity nor_gate;

architecture rtl of nor_gate is
    constant GO_ADDR_MMAP:integer:=2#001_1100.001#E14;
    constant freq_hz : unsigned := 50_000_000;
    signal   MAGIC_NUM_3 : bit_vector(3 downto 0) := 0sx\"\";
    constant MAGIC_NUM_1 : integer := 2#10101#; -- test constants against tokenizer
    constant MAGIC_NUM_2 : std_logic_vector(7 downto 0) := 0; -- 8c\"11\";
begin
    c <= a nor \\In\\;

end architecture rtl; /* long comment */

entity nor_gate_tb is end;

architecture tb of nor_gate_tb is 
-- declare DUT component
component nor_gate 
	generic(
		N: positive
	);
	port(
		a: in  std_logic_vector(N-1 downto 0);
		b: in  std_logic_vector(N-1 downto 0);
		c: out std_logic_vector(N-1 downto 0)
	);
end component nor_gate;
begin 
	DUT : component nor_gate 
    generic map     (
		N   => N
	) port map(
		a => w_a,
		b => w_b,
		c => w_c
	);

end;

package P is

function F return INTEGER;
attribute FOREIGN of F: function is \"implementation-dependent information\"; 

end package P;

package outer is
    package inner is 
        component some_component is
        end component;
    end package;
end package;

entity X is
port (P1, P2: inout BIT); 
constant Delay: TIME := 1 ms;
begin
CheckTiming (P1, P2, 2*Delay); end X ;

package TimeConstants is 
constant tPLH : Time := 9 ns;
constant tPHL : Time := 10 ns;
constant tPLZ :  Time := 12 ns;
constant tPZL :  Time := 7 ns;
constant tPHZ :  Time := 8 ns;
constant tPZH : Time := 8 ns;
end TimeConstants ;

package body TriState is

    package ent is 

    end;
    function BitVal (Value: Tri) return Bit is
        constant Bits : Bit_Vector := \"0100\"; 
    begin
        return Bits(Tri'Pos(Value)); 
    end;

    function TriVal (Value: Bit) return Tri is 
    begin
        return Tri'Val(Bit'Pos(Value)); 
    end;

    function Resolve (Sources: TriVector) return Tri is 
        variable V: Tri := 'Z';
    begin
        for i in Sources'Range loop
            if Sources(i) /= 'Z' then 
                if V = 'Z' then
                    V := Sources(i);
                else 
                    return 'E';
                end if; 
            end if;
        end loop; 
        return V;
    end;

end package body TriState;

architecture test of nor_gate is 

begin 

GEN_ADD: for I in 0 to 7 generate

LOWER_BIT: if I=0 generate
  U0: HALFADD port map
     (A(I),B(I),S(I),C(I));
end generate LOWER_BIT;

UPPER_BITS: if I>0 generate
  UX: FULLADD port map
     (A(I),B(I),C(I-1),S(I),C(I));
end generate UPPER_BITS;

end generate GEN_ADD;

end architecture;

architecture rtl of complex_multiplier is
begin

    mult_structure : case implementation generate 
        when single_cycle => 
            signal real_pp1, real_pp2 : ...;
            ...;
            begin
                real_mult1 : component multiplier
                    port map ( ... ); 
            end;
        when multicycle =>
            signal real_pp1, real_pp2 : ...;
            ...;
            begin
                mult : component multiplier
                    port map ( ... );
            end;
        when pipelined => 
            mult1 : component multiplier
                port map ( ... );
    end generate mutl_structure;

end architecture rtl;
";
        let _ = VHDLParser::parse(VhdlTokenizer::from_source_code(&s).into_tokens());
        panic!("manually inspect token list")
    }

    #[test]
    fn test_if_gen() {
        let data = std::fs::read_to_string("./tests/t5/if_gen.vhd").unwrap();

        let syms = VHDLParser::read(&data).unwrap().into_symbols();
        println!("symbols list: {:?}", syms);
        println!(
            "architecture refs: {:?}",
            syms[1].as_architecture().unwrap().get_refs()
        );
        // verify we captured the dependency outside the if_gen and inside the if_gen (2 * 2)
        assert_eq!(syms[1].as_architecture().unwrap().get_refs().len(), 2);
    }

    #[test]
    fn test_entity_after_package() {
        let data = std::fs::read_to_string("./tests/t5/ent_after_pkg.vhd").unwrap();
        let syms = VHDLParser::read(&data).unwrap().into_symbols();
        // capture all units (primary and secondary)
        assert_eq!(syms.len(), 6);
    }

    #[test]
    fn test_procedure_in_process() {
        let data = std::fs::read_to_string("./tests/t5/proced_in_proc.vhd").unwrap();
        let syms = VHDLParser::read(&data).unwrap().into_symbols();
        // capture all units (primary and secondary)
        println!("{:?}", syms);
        // verify we captured all 3 sub-entities following procedures
        assert_eq!(syms[1].as_architecture().unwrap().get_refs().len(), 3);
    }
}
