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
    reference::RefSet,
    sv::{
        error::SystemVerilogError,
        token::{
            identifier::Identifier, keyword::Keyword, operator::Operator, token::SystemVerilogToken,
        },
    },
    verilog::symbols::VerilogSymbol,
};

use super::SystemVerilogSymbol;

#[derive(Debug, PartialEq)]
pub struct Package {
    name: Identifier,
    refs: RefSet,
    pos: Position,
}

impl Package {
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

impl Package {
    pub fn from_tokens<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        // take optional lifetime specifier
        if let Some(maybe_lifetime) = tokens.peek() {
            if maybe_lifetime.as_type().check_keyword(&Keyword::Automatic)
                || maybe_lifetime.as_type().check_keyword(&Keyword::Static)
            {
                tokens.next().unwrap();
            }
        }

        // take package name
        let name = match tokens.next().take().unwrap().take() {
            SystemVerilogToken::Identifier(id) => id,
            _ => return Err(SystemVerilogError::Vague),
        };

        // take terminator ';'
        let t = tokens.next().take().unwrap();
        if t.as_type().check_delimiter(&Operator::Terminator) == false {
            return Err(SystemVerilogError::ExpectingOperator(Operator::Terminator));
        }

        let mut refs = RefSet::new();

        // parse until finding `endpackage`
        while let Some(t) = tokens.next() {
            if t.as_type().is_eof() == true {
                return Err(SystemVerilogError::ExpectingKeyword(Keyword::Endpackage));
            } else if t.as_type().check_keyword(&Keyword::Endpackage) {
                // exit the loop for parsing the package
                break;
            // parse other references
            } else if t.as_type().check_keyword(&Keyword::Import) {
                let i_refs = SystemVerilogSymbol::parse_import_statement(tokens)?;
                refs.extend(i_refs);
            } else if let Some(stmt) = VerilogSymbol::into_next_statement(t, tokens)? {
                // println!("{}", statement_to_string(&stmt));
                VerilogSymbol::handle_statement(stmt, None, None, &mut refs, None)?;
            }
        }

        Ok(Package {
            name: name,
            refs: refs,
            pos: pos,
        })
    }
}
