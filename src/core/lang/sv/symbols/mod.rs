use std::iter::Peekable;

use package::Package;

use super::error::SystemVerilogError;
use super::token::identifier::Identifier;
use super::token::operator::Operator;
use super::token::tokenizer::SystemVerilogTokenizer;
use crate::core::lang::lexer::{Position, Token};
use crate::core::lang::parser::{Parse, Symbol};
use crate::core::lang::reference::{CompoundIdentifier, RefSet};
use crate::core::lang::sv::token::keyword::Keyword;
use crate::core::lang::sv::token::token::SystemVerilogToken;
use crate::core::lang::verilog::symbols::config::Config;
use std::str::FromStr;

use super::super::verilog::symbols::module::Module;

pub type Statement = Vec<Token<SystemVerilogToken>>;

pub mod package;

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
}

impl SystemVerilogSymbol {
    pub fn as_name(&self) -> Option<&Identifier> {
        match &self {
            Self::Module(m) => Some(m.get_name()),
            Self::Config(c) => Some(c.get_name()),
            Self::Package(p) => Some(p.get_name()),
        }
    }

    pub fn get_position(&self) -> &Position {
        match self {
            Self::Module(m) => m.get_position(),
            Self::Config(c) => c.get_position(),
            Self::Package(p) => p.get_position(),
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
        }
    }

    pub fn extend_refs(&mut self, refs: RefSet) {
        match self {
            Self::Module(m) => m.extend_refs(refs),
            Self::Config(c) => c.extend_refs(refs),
            Self::Package(p) => p.extend_refs(refs),
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

        while let Some(t) = tokens.next() {
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
            // create config design element
            } else if t.as_type().check_keyword(&Keyword::Config) {
                symbols.push(
                    match SystemVerilogSymbol::parse_config(&mut tokens, t.into_position()) {
                        Ok(config) => Ok(Symbol::new(config)),
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
        Ok(Self::Module(Module::from_tokens(tokens, pos)?))
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
