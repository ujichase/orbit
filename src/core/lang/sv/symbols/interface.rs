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
    sv::{
        error::SystemVerilogError,
        token::{identifier::Identifier, keyword::Keyword, token::SystemVerilogToken},
    },
    verilog::{
        interface::{ParamList, PortList},
        symbols::VerilogSymbol,
    },
};

use super::SystemVerilogSymbol;

#[derive(Debug, PartialEq)]
pub struct Interface {
    name: Identifier,
    params: ParamList,
    ports: PortList,
    refs: RefSet,
    pos: Position,
}

impl Interface {
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

impl Interface {
    pub fn from_tokens<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        // take interface name
        let interface_name = match tokens.next().take().unwrap().take() {
            SystemVerilogToken::Identifier(id) => id,
            _ => return Err(SystemVerilogError::Vague),
        };

        // initialize container for references to other design elements
        let mut refs = RefSet::new();

        // take all import statements
        while let Some(t) = tokens.peek() {
            if t.as_type().check_keyword(&Keyword::Import) {
                let _ = tokens.next().unwrap();
                let i_refs = SystemVerilogSymbol::parse_import_statement(tokens)?;
                refs.extend(i_refs);
            } else {
                break;
            }
        }

        // parse the declaration of the interface
        let (mut params, mut ports, d_refs) = VerilogSymbol::parse_module_declaration(tokens)?;
        refs.extend(d_refs);

        let mut refs = RefSet::new();

        // parse until finding the ending keyword
        while let Some(t) = tokens.next() {
            if t.as_type().is_eof() == true {
                return Err(SystemVerilogError::ExpectingKeyword(Keyword::Endinterface));
            } else if t.as_type().check_keyword(&Keyword::Endinterface) {
                // exit the loop for parsing this design element
                break;
            // parse other references
            } else if t.as_type().check_keyword(&Keyword::Import) {
                let i_refs = SystemVerilogSymbol::parse_import_statement(tokens)?;
                refs.extend(i_refs);
            } else if let Some(stmt) = VerilogSymbol::into_next_statement(t, tokens)? {
                // println!("{}", statement_to_string(&stmt));
                VerilogSymbol::handle_statement(
                    stmt,
                    Some(&mut params),
                    Some(&mut ports),
                    &mut refs,
                    None,
                )?;
            }
        }

        // for all ports and their datatypes, try to see if any are references to interfaces
        ports
            .iter()
            .filter_map(|p| p.as_user_defined_data_type())
            .for_each(|intf| {
                refs.insert(CompoundIdentifier::new_minimal_verilog(intf.clone()));
            });
        params
            .iter()
            .filter_map(|p| p.as_user_defined_data_type())
            .for_each(|intf| {
                refs.insert(CompoundIdentifier::new_minimal_verilog(intf.clone()));
            });

        Ok(Interface {
            name: interface_name,
            params: params,
            ports: ports,
            refs: refs,
            pos: pos,
        })
    }
}
