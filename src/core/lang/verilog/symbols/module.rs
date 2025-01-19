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
        interface::{self, ParamList, PortList},
        token::{identifier::Identifier, operator::Operator},
    },
};
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

        result.push_str(&format!("module "));
        result.push_str(&self.name.to_string());
        result.push_str(&interface::display_interface(&self.parameters, true, fmt));
        result.push_str(&interface::display_interface(&self.ports, false, fmt));
        result.push(';');
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
        result.push_str(&self.name.to_string());
        // parameters
        result.push_str(&interface::display_connections(
            &self.parameters,
            true,
            "",
            "",
            fmt,
        ));
        // leave whitespace between module name and instance if no parameters are available
        if self.parameters.is_empty() == true {
            result.push(' ');
        }

        // instance name
        if let Some(n) = name {
            result.push_str(&n.to_string());
        } else {
            result.push_str(&fmt.get_instance_name());
        }

        // ports
        result.push_str(&interface::display_connections(
            &self.ports,
            false,
            signal_prefix,
            signal_suffix,
            fmt,
        ));
        result.push(';');
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
            result.push_str(&&&p.into_declaration(false, &param_spacer, "", "", fmt));
            result.push_str(&Operator::Terminator.to_string());
            result.push('\n');
        });
        if self.parameters.is_empty() == false {
            result.push('\n');
        }
        self.ports.iter().for_each(|p| {
            result.push_str(&&&p.into_declaration(
                false,
                &port_spacer,
                wire_prefix,
                wire_suffix,
                fmt,
            ));
            result.push_str(&Operator::Terminator.to_string());
            result.push('\n');
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

        // println!("{}", mod_name);
        // println!("{:?}", ports);
        // println!("{:?}", refs);
        // println!("{:?}", params);
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
