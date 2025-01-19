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

use crate::core::lang::{
    lexer::{Position, Token},
    reference::{CompoundIdentifier, RefSet},
    sv::token::{keyword::Keyword, operator::Operator, token::SystemVerilogToken},
    verilog::{error::VerilogError, token::identifier::Identifier},
};

#[derive(Debug, PartialEq)]
pub struct Config {
    name: Identifier,
    refs: RefSet,
    pos: Position,
}

impl Config {
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    pub fn get_refs(&self) -> &RefSet {
        &self.refs
    }

    pub fn extend_refs(&mut self, refs: RefSet) {
        self.refs.extend(refs);
    }
}

impl Config {
    pub fn from_tokens<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        // take config name
        let config_name = match tokens.next().take().unwrap().take() {
            SystemVerilogToken::Identifier(id) => id,
            _ => return Err(VerilogError::Vague),
        };

        // take terminator ';'
        tokens.next().take().unwrap();

        let mut refs = RefSet::new();

        // parse until finding `endconfig`
        while let Some(t) = tokens.next() {
            if t.as_type().is_eof() == true {
                return Err(VerilogError::ExpectingKeyword(Keyword::Endconfig));
            } else if t.as_type().check_keyword(&Keyword::Endconfig) {
                // exit the loop for parsing the config
                break;
            // parse to try to find a module name
            } else if t.as_type().check_keyword(&Keyword::Design) {
                refs.extend(Self::parse_design_statement(tokens)?);
            }
        }

        Ok(Config {
            name: config_name,
            refs: refs,
            pos: pos,
        })
    }

    fn parse_design_statement<I>(tokens: &mut Peekable<I>) -> Result<RefSet, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        let mut refs = RefSet::new();
        while let Some(t) = tokens.next() {
            if t.as_type().check_delimiter(&Operator::Terminator) {
                break;
            } else if let Some(name) = t.take().take_identifier() {
                // skip the library name if this is what we are looking at
                if let Some(t_next) = tokens.peek() {
                    if t_next.as_type().check_delimiter(&Operator::Dot) == true {
                        continue;
                    }
                }
                refs.insert(CompoundIdentifier::new_minimal_verilog(name));
            }
        }
        Ok(refs)
    }
}
