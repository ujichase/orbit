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
        token::{
            identifier::Identifier, keyword::Keyword, operator::Operator, token::SystemVerilogToken,
        },
    },
    verilog::{error::VerilogError, interface::ParamList, symbols::VerilogSymbol},
};

use super::SystemVerilogSymbol;

#[derive(Debug, PartialEq)]
pub struct Class {
    name: Identifier,
    params: ParamList,
    refs: RefSet,
    pos: Position,
}

impl Class {
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

impl Class {
    pub fn from_tokens<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        // skip the "class" keyword if the class was starting with "virtual"
        if tokens
            .peek()
            .unwrap()
            .as_type()
            .check_keyword(&Keyword::Class)
        {
            tokens.next().unwrap();
        }

        // parse a lifetime if it exists
        if let Some(maybe_lifetime) = tokens.peek() {
            if maybe_lifetime.as_type().check_keyword(&Keyword::Automatic)
                || maybe_lifetime.as_type().check_keyword(&Keyword::Static)
            {
                tokens.next().unwrap();
            }
        }

        // take the class name
        let name = match tokens.next().take().unwrap().take() {
            SystemVerilogToken::Identifier(id) => id,
            _ => return Err(SystemVerilogError::Vague),
        };

        // initialize container for references to other design elements
        let mut refs = RefSet::new();

        let mut params = ParamList::new();

        // parse a parameter port list if it exists
        if let Some(next) = tokens.peek() {
            if next.as_type().check_delimiter(&Operator::Pound) == true {
                let _ = tokens.next().unwrap();
                let t_next = tokens.next().unwrap();
                if t_next.as_ref().check_delimiter(&Operator::ParenL) == true {
                    // parse parameter list
                    let (decl_params, param_refs) =
                        VerilogSymbol::parse_module_param_list(tokens, t_next.locate().line())?;
                    params.extend(decl_params);
                    refs.extend(param_refs);
                } else {
                    return Err(VerilogError::ExpectingOperator(Operator::ParenL));
                }
            }
        }

        // take extends for multiple class inheritances (multiple is only supported for "interface class")
        if let Some(next) = tokens.peek() {
            if next.as_type().check_keyword(&Keyword::Extends) == true {
                let _ = tokens.next().unwrap();
                let ext_class_name = match tokens.next().take().unwrap().take() {
                    SystemVerilogToken::Identifier(id) => id,
                    _ => return Err(SystemVerilogError::Vague),
                };
                // println!("extends {}", impl_class_name);
                refs.insert(CompoundIdentifier::new_minimal_verilog(ext_class_name));
                loop {
                    if let Some(peek) = tokens.peek() {
                        // take another set of extend
                        if peek.as_type().check_delimiter(&Operator::Comma) == true {
                            let _ = tokens.next().unwrap();
                            let ext_class_name = match tokens.next().take().unwrap().take() {
                                SystemVerilogToken::Identifier(id) => id,
                                _ => return Err(SystemVerilogError::Vague),
                            };
                            // println!("extends {}", impl_class_name);
                            refs.insert(CompoundIdentifier::new_minimal_verilog(ext_class_name));
                        } else if peek.as_type().check_delimiter(&Operator::ParenL) == true {
                            let beg_t = tokens.next().unwrap();
                            let stmt = VerilogSymbol::parse_until_operator(
                                tokens,
                                beg_t,
                                Operator::ParenR,
                            )?;
                            // update references that may appear in the statement
                            if let Some(s_refs) =
                                SystemVerilogSymbol::extract_refs_from_statement(&stmt)
                            {
                                refs.extend(s_refs);
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        // take implements for multiple class interfaces
        if let Some(next) = tokens.peek() {
            if next.as_type().check_keyword(&Keyword::Implements) == true {
                let _ = tokens.next().unwrap();
                let impl_class_name = match tokens.next().take().unwrap().take() {
                    SystemVerilogToken::Identifier(id) => id,
                    _ => return Err(SystemVerilogError::Vague),
                };
                // println!("implements {}", impl_class_name);
                refs.insert(CompoundIdentifier::new_minimal_verilog(impl_class_name));
                loop {
                    if let Some(peek) = tokens.peek() {
                        // take another set of implement
                        if peek.as_type().check_delimiter(&Operator::Comma) == true {
                            let _ = tokens.next().unwrap();
                            let impl_class_name = match tokens.next().take().unwrap().take() {
                                SystemVerilogToken::Identifier(id) => id,
                                _ => return Err(SystemVerilogError::Vague),
                            };
                            // println!("implements {}", impl_class_name);
                            refs.insert(CompoundIdentifier::new_minimal_verilog(impl_class_name));
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        // take the terminator
        let t = tokens.next().take().unwrap();
        if t.as_type().check_delimiter(&Operator::Terminator) == false {
            return Err(SystemVerilogError::ExpectingOperator(Operator::Terminator));
        }

        // take the class body
        let (c_refs, _c_deps) = Self::parse_class_body(tokens)?;
        refs.extend(c_refs);

        Ok(Class {
            name: name,
            params: params,
            refs: refs,
            pos: pos,
        })
    }

    fn parse_class_body<I>(tokens: &mut Peekable<I>) -> Result<(RefSet, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        let mut refs = RefSet::new();
        let mut deps = RefSet::new();

        while let Some(t) = tokens.next() {
            // expecting `endclass`
            if t.as_ref().is_eof() == true {
                return Err(VerilogError::ExpectingKeyword(Keyword::Endclass));
            // exit from the class body
            } else if t.as_ref().check_keyword(&Keyword::Endclass) == true {
                break;
            } else if let Some(stmt) = VerilogSymbol::into_next_statement(t, tokens)? {
                // println!("{}", statement_to_string(&stmt));
                VerilogSymbol::handle_statement(stmt, None, None, &mut refs, Some(&mut deps))?;
            }
        }
        Ok((refs, deps))
    }
}
