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

use super::super::super::vhdl::token::Identifier as VhdlIdentifier;
use super::VerilogSymbol;
use crate::core::lang::{
    lexer::{Position, Token},
    reference::{CompoundIdentifier, RefSet},
    sv::{
        format::SystemVerilogFormat,
        symbols::SystemVerilogSymbol,
        token::{keyword::Keyword, token::SystemVerilogToken},
    },
    verilog::{
        error::VerilogError,
        interface::{self, DataType, ParamList, PortList},
        token::{identifier::Identifier, operator::Operator},
    },
    vhdl::token::VhdlTokenizer,
};

use crate::core::lang::highlight;
use crate::core::lang::highlight::ToColor;

use serde_derive::Serialize;
use std::iter::Peekable;

#[derive(Debug, PartialEq, Serialize)]
pub struct Module {
    #[serde(rename = "identifier")]
    name: Identifier,
    #[serde(rename = "generics")]
    parameters: ParamList,
    ports: PortList,
    architectures: Vec<()>,
    /// The set of names that were referenced in the entity.
    #[serde(skip_serializing)]
    refs: RefSet,
    /// The set of references that were identified as components.
    #[serde(skip_serializing)]
    deps: RefSet,
    #[serde(skip_serializing)]
    pos: Position,
    language: String,
}

impl Module {
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

impl Module {
    pub fn into_declaration(&self, fmt: &SystemVerilogFormat) -> String {
        let mut result = String::new();

        result.push_str(&format!("{} ", Keyword::Module.to_color()));
        result.push_str(&format!(
            "{}",
            highlight::color(&self.get_name().to_string(), highlight::ENTITY_NAME)
        ));
        result.push_str(&format!(
            "{}",
            interface::display_interface(&self.parameters, true, fmt)
        ));
        result.push_str(&format!(
            "{}",
            interface::display_interface(&self.ports, false, fmt)
        ));
        result.push_str(&format!("{}", Operator::Terminator.to_color()));
        result
    }

    pub fn into_instance(
        &self,
        name: &Option<VhdlIdentifier>,
        signal_prefix: &str,
        signal_suffix: &str,
        fmt: &SystemVerilogFormat,
    ) -> String {
        let mut result = String::new();
        // module name
        result.push_str(&format!(
            "{}",
            highlight::style::entity(&self.get_name().to_string())
        ));
        // parameters
        result.push_str(&format!(
            "{}",
            interface::display_connections(&self.parameters, true, "", "", fmt,)
        ));
        // leave whitespace between module name and instance if no parameters are available
        if self.parameters.is_empty() == true {
            result.push(' ');
        }

        // instance name
        let name = match &name {
            Some(iden) => iden.clone(),
            None => VhdlIdentifier::Basic(fmt.get_instance_name().to_string()),
        };
        result.push_str(&format!(
            "{}",
            highlight::style::instance(&name.to_string())
        ));

        // ports
        result.push_str(&format!(
            "{}",
            interface::display_connections(&self.ports, false, signal_prefix, signal_suffix, fmt,)
        ));
        result.push_str(&format!("{}", Operator::Terminator.to_color()));
        result
    }

    pub fn into_wires(
        &self,
        wire_prefix: &str,
        wire_suffix: &str,
        fmt: &SystemVerilogFormat,
    ) -> String {
        // compute the longest word
        let param_spacer = match fmt.is_auto_name_aligned() {
            true => Some(interface::longest_port_decl(false, &self.parameters, fmt)),
            false => None,
        };

        let port_spacer = match fmt.is_auto_name_aligned() {
            true => Some(interface::longest_port_decl(false, &self.ports, fmt)),
            false => None,
        };

        let mut result = String::new();
        self.parameters.iter().for_each(|p| {
            result.push_str(&format!(
                "{}",
                p.into_declaration(false, &param_spacer, "", "", fmt)
            ));
            result.push_str(&format!("{}\n", Operator::Terminator.to_color()));
        });
        if self.parameters.is_empty() == false {
            result.push('\n');
        }
        self.ports.iter().for_each(|p| {
            result.push_str(&format!(
                "{}",
                p.into_declaration(false, &port_spacer, wire_prefix, wire_suffix, fmt,)
            ));
            result.push_str(&format!("{}\n", Operator::Terminator.to_color()));
        });
        result
    }
}

impl Module {
    pub fn get_deps(&self) -> &RefSet {
        &self.deps
    }

    pub fn is_testbench(&self) -> bool {
        self.ports.is_empty()
    }
}

impl Module {
    /// Returns the list of compound identifiers that were parsed from entity instantiations.
    pub fn get_edge_list_entities(&self) -> Vec<CompoundIdentifier> {
        let mut list: Vec<CompoundIdentifier> = self.deps.iter().map(|f| f.clone()).collect();
        list.sort();
        list
    }

    pub fn get_edge_list(&self) -> Vec<CompoundIdentifier> {
        let mut list: Vec<CompoundIdentifier> = self.refs.iter().map(|f| f.clone()).collect();
        list.sort();
        list
    }

    /// Parses an `Module` design element from the module's identifier to
    /// the END closing statement.
    pub fn from_tokens<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
        language: &str,
    ) -> Result<Self, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        // take module name
        let mod_name = match tokens.next().take().unwrap().take() {
            SystemVerilogToken::Identifier(id) => id,
            SystemVerilogToken::Directive(cd) => Identifier::Directive(cd),
            _ => return Err(VerilogError::ModuleNameIsNotIdentifier),
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

        // parse the interface/declaration of the module
        let (mut params, mut ports, d_refs) = VerilogSymbol::parse_module_declaration(tokens)?;
        refs.extend(d_refs);

        // parse the body of the module
        let (b_refs, deps) =
            VerilogSymbol::parse_module_architecture(tokens, &mut params, &mut ports)?;
        refs.extend(b_refs);

        // DEPREC: This is now handled when parsing module architecture
        // // update declared ports from any architecture port definitions
        // body_ports
        //     .into_iter()
        //     .for_each(|p| interface::update_port_list(&mut ports, p, false));

        // DEPREC: This is now handled when parsing module architecture
        // // update declared params from any architecture param definitions
        // body_params
        //     .into_iter()
        //     .for_each(|p| interface::update_port_list(&mut params, p, false));

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

        Ok(Module {
            name: mod_name,
            parameters: params,
            ports: ports,
            refs: refs,
            deps: deps,
            architectures: Vec::new(),
            pos: pos,
            language: String::from(language),
        })
    }
}

use crate::core::lang::lexer::Tokenize;
use crate::core::lang::verilog::interface::tokens_to_string;
use crate::core::lang::vhdl::error::VhdlError;
use crate::core::lang::vhdl::symbols::entity::Entity;
use crate::core::lang::vhdl::token::delimiter::Delimiter as VhDelimiter;
use crate::core::lang::vhdl::token::identifier::Identifier as VhIdentifier;
use crate::core::lang::vhdl::token::keyword::Keyword as VhKeyword;
use crate::core::lang::vhdl::token::VhdlToken as Vvt;
use crate::core::lang::vhdl::token::VhdlToken;

impl Module {
    /// Builds an [Entity] from the structured HDL data.
    pub fn to_vhdl_entity(&self) -> Result<Entity, VhdlError> {
        // start with blank slate of VHDL tokens to build from
        let mut tokens: Vec<Token<Vvt>> = Vec::new();

        // assemble entity name
        let name = match &self.name {
            Identifier::Basic(s) => VhIdentifier::Basic(s.clone()),
            Identifier::Escaped(s) => VhIdentifier::Extended(s.clone()),
            Identifier::Directive(s) => VhIdentifier::Extended(s.clone()),
            Identifier::System(s) => VhIdentifier::Extended(s.clone()),
        };
        tokens.push(Self::vh(Vvt::Identifier(name)));

        // add IS keyword
        tokens.push(Self::vh(Vvt::Keyword(VhKeyword::Is)));

        // add generics
        if self.parameters.len() > 0 {
            // add the correct begining syntax
            tokens.push(Self::vh(Vvt::Keyword(VhKeyword::Generic)));
            tokens.push(Self::vh(Vvt::Delimiter(VhDelimiter::ParenL)));

            // add each parameter
            self.parameters.iter().for_each(|p| {
                // add the port identifier
                let name = match &p.get_name() {
                    Identifier::Basic(s) => VhIdentifier::Basic(s.clone()),
                    Identifier::Escaped(s) => VhIdentifier::Extended(s.clone()),
                    Identifier::Directive(s) => VhIdentifier::Extended(s.clone()),
                    Identifier::System(s) => VhIdentifier::Extended(s.clone()),
                };
                tokens.push(Self::vh(Vvt::Identifier(name)));
                // add the ':'
                tokens.push(Self::vh(Vvt::Delimiter(VhDelimiter::Colon)));

                // add the datatype
                tokens.push(Self::vh(Self::convert_datatype_to_vh(p.get_datatype())));

                // any ranges for that dataype?
                if let Some(ranges) = p.get_datatype().get_ranges() {
                    ranges.into_iter().for_each(|r| {
                        tokens.append(&mut Self::convert_range_to_vh(r.0, r.1));
                    });
                }

                // add the default value (if exists)
                if let Some(expr) = p.get_default().as_static_expr() {
                    tokens.append(&mut Self::convert_default_to_vh(expr));
                }

                tokens.push(Self::vh(Vvt::Delimiter(VhDelimiter::Terminator)));
            });

            // remove the last trailing ';'
            tokens.pop();

            // add the correct closing syntax
            tokens.push(Self::vh(Vvt::Delimiter(VhDelimiter::ParenR)));
            tokens.push(Self::vh(Vvt::Delimiter(VhDelimiter::Terminator)));
        }
        // add ports
        if self.ports.len() > 0 {
            // add the correct begining syntax
            tokens.push(Self::vh(Vvt::Keyword(VhKeyword::Port)));
            tokens.push(Self::vh(Vvt::Delimiter(VhDelimiter::ParenL)));

            // add each port
            self.ports.iter().for_each(|p| {
                // add the port identifier
                let name = match &p.get_name() {
                    Identifier::Basic(s) => VhIdentifier::Basic(s.clone()),
                    Identifier::Escaped(s) => VhIdentifier::Extended(s.clone()),
                    Identifier::Directive(s) => VhIdentifier::Extended(s.clone()),
                    Identifier::System(s) => VhIdentifier::Extended(s.clone()),
                };
                tokens.push(Self::vh(Vvt::Identifier(name)));
                // add the ':'
                tokens.push(Self::vh(Vvt::Delimiter(VhDelimiter::Colon)));
                // add the port direction
                let dir = if let Some(dir) = p.get_mode() {
                    match dir {
                        Keyword::Input => VhKeyword::In,
                        Keyword::Output => VhKeyword::Out,
                        Keyword::Inout => VhKeyword::Inout,
                        _ => panic!("unsupported port mode conversion"),
                    }
                } else {
                    VhKeyword::In
                };
                tokens.push(Self::vh(Vvt::Keyword(dir)));

                // add the datatype
                tokens.push(Self::vh(Self::convert_datatype_to_vh(p.get_datatype())));

                // any ranges for that dataype?
                if let Some(ranges) = p.get_datatype().get_ranges() {
                    ranges.into_iter().for_each(|r| {
                        tokens.append(&mut Self::convert_range_to_vh(r.0, r.1));
                    });
                }

                // add the default value (if exists)
                if let Some(expr) = p.get_default().as_static_expr() {
                    tokens.append(&mut Self::convert_default_to_vh(expr));
                }

                tokens.push(Self::vh(Vvt::Delimiter(VhDelimiter::Terminator)));
            });

            // remove the last trailing ';'
            tokens.pop();

            // add the correct closing syntax
            tokens.push(Self::vh(Vvt::Delimiter(VhDelimiter::ParenR)));
            tokens.push(Self::vh(Vvt::Delimiter(VhDelimiter::Terminator)));
        }

        // add proper closing before parsing
        tokens.push(Self::vh(Vvt::Keyword(VhKeyword::End)));

        // parse the assembled token list to create the module
        let mut tokens = tokens.into_iter().peekable();
        Entity::from_tokens(&mut tokens, Position::new())
    }

    /// Helps build SystemVerilog tokens from a VHDL context.
    fn vh(token: Vvt) -> Token<Vvt> {
        Token::new(token, Position::new())
    }

    /// Helps convert a SV token into its VHDL equivalent when dealing with
    /// datatypes.
    fn convert_datatype_to_vh(datatype: &DataType) -> Vvt {
        let dtype = datatype.get_type();
        let has_range = datatype.get_ranges().is_some();

        let datatype = if let Some(dtype) = dtype {
            match dtype {
                SystemVerilogToken::Keyword(k) => match k {
                    Keyword::Int | Keyword::Integer => {
                        Vvt::Identifier(VhIdentifier::Basic("integer".to_string()))
                    }
                    Keyword::Byte => Vvt::Identifier(VhIdentifier::Basic("bit_vector".to_string())),
                    Keyword::Logic => match has_range {
                        true => {
                            Vvt::Identifier(VhIdentifier::Basic("std_logic_vector".to_string()))
                        }
                        false => Vvt::Identifier(VhIdentifier::Basic("std_logic".to_string())),
                    },
                    Keyword::Bit => match has_range {
                        true => Vvt::Identifier(VhIdentifier::Basic("bit_vector".to_string())),
                        false => Vvt::Identifier(VhIdentifier::Basic("bit".to_string())),
                    },
                    Keyword::String => Vvt::Identifier(VhIdentifier::Basic("string".to_string())),
                    _ => panic!("unsupported datatype keyword conversion to vhdl"),
                },
                SystemVerilogToken::Identifier(i) => match i {
                    Identifier::Basic(s) => Vvt::Identifier(VhIdentifier::Basic(s.clone())),
                    Identifier::Escaped(s) => Vvt::Identifier(VhIdentifier::Extended(s.clone())),
                    Identifier::Directive(s) => Vvt::Identifier(VhIdentifier::Extended(s.clone())),
                    Identifier::System(s) => Vvt::Identifier(VhIdentifier::Extended(s.clone())),
                },
                _ => panic!("unsupported datatype conversion to vhdl"),
            }
        } else {
            match has_range {
                true => Vvt::Identifier(VhIdentifier::Basic("std_logic_vector".to_string())),
                false => Vvt::Identifier(VhIdentifier::Basic("std_logic".to_string())),
            }
        };
        datatype
    }

    /// Helps convert a SV range (set of tokens) into a VHDL equivalent set of tokens
    /// for specifying a range for a particular datatype.
    fn convert_range_to_vh(
        lhs: Vec<SystemVerilogToken>,
        rhs: Vec<SystemVerilogToken>,
    ) -> Vec<Token<VhdlToken>> {
        let mut tokens = Vec::new();
        // opening bracket
        tokens.push(Self::vh(Vvt::Delimiter(VhDelimiter::ParenL)));

        // left side of range
        VhdlTokenizer::tokenize(&tokens_to_string(&lhs).into_all_bland())
            .into_iter()
            .filter_map(|r| match r {
                Ok(r) => Some(r),
                Err(_) => None,
            })
            .filter(|r| r.as_type().is_eof() == false)
            .for_each(|t| {
                tokens.push(t);
            });

        // delimiter between range endpoints
        tokens.push(Self::vh(Vvt::Keyword(VhKeyword::Downto)));

        // right side of range
        VhdlTokenizer::tokenize(&tokens_to_string(&rhs).into_all_bland())
            .into_iter()
            .filter_map(|r| match r {
                Ok(r) => Some(r),
                Err(_) => None,
            })
            .filter(|r| r.as_type().is_eof() == false)
            .for_each(|t| {
                tokens.push(t);
            });

        // closing bracket
        tokens.push(Self::vh(Vvt::Delimiter(VhDelimiter::ParenR)));

        tokens
    }

    /// Helps convert a SV default value into a VHDL equivalent set of tokens.
    fn convert_default_to_vh(expr: &Vec<SystemVerilogToken>) -> Vec<Token<VhdlToken>> {
        let mut tokens = Vec::new();

        VhdlTokenizer::tokenize(&tokens_to_string(&expr).into_all_bland())
            .into_iter()
            .filter_map(|r| match r {
                Ok(r) => Some(r),
                Err(_) => None,
            })
            .filter(|r| r.as_type().is_eof() == false)
            .for_each(|t| {
                tokens.push(t);
            });

        // only introduce the '=' token if we successfully transfered the VHDL to SV
        if tokens.len() > 0 {
            tokens.insert(0, Self::vh(Vvt::Delimiter(VhDelimiter::VarAssign)));
        }

        tokens
    }
}
