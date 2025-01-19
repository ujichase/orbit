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

use super::VerilogSymbol;
use crate::core::lang::{
    lexer::{Position, Token},
    reference::RefSet,
    sv::token::{keyword::Keyword, operator::Operator, token::SystemVerilogToken},
    verilog::{error::VerilogError, interface::PortList, token::identifier::Identifier},
};

use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub struct Primitive {
    name: Identifier,
    ports: PortList,
    /// The set of names that were referenced in the primitive.
    refs: RefSet,
    pos: Position,
}

impl Primitive {
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

impl Primitive {
    pub fn from_tokens<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        // take primitive name
        let name = match tokens.next().take().unwrap().take() {
            SystemVerilogToken::Identifier(id) => id,
            _ => return Err(VerilogError::Vague),
        };

        let mut refs = RefSet::new();
        let mut ports = PortList::new();

        // take the port list
        let t = tokens.next().unwrap();
        if t.as_ref().check_delimiter(&Operator::ParenL) == true {
            let (d_ports, d_refs) =
                VerilogSymbol::parse_module_port_list(tokens, t.locate().line())?;
            ports.extend(d_ports);
            refs.extend(d_refs);
        }

        // take terminator ';'
        tokens.next().take().unwrap();

        // parse until finding `endprimitive`
        while let Some(t) = tokens.next() {
            if t.as_type().is_eof() == true {
                return Err(VerilogError::ExpectingKeyword(Keyword::Endprimitive));
            } else if t.as_type().check_keyword(&Keyword::Endprimitive) {
                // exit the loop for parsing the primitive
                break;
            // parse to try to find a module name
            } else if let Some(stmt) = VerilogSymbol::into_next_statement(t, tokens)? {
                // println!("{}", statement_to_string(&stmt));
                VerilogSymbol::handle_statement(stmt, None, Some(&mut ports), &mut refs, None)?;
            }
        }

        Ok(Primitive {
            name: name,
            ports: ports,
            refs: refs,
            pos: pos,
        })
    }
}
