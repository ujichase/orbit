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

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct VhdlFormat {
    #[serde(rename = "highlight-syntax")]
    highlight_syntax: Option<bool>,
    #[serde(rename = "tab-size")]
    tab_size: Option<u8>,
    #[serde(rename = "type-auto-alignment")]
    type_auto_alignment: Option<bool>,
    #[serde(rename = "type-offset")]
    type_offset: Option<u8>,
    #[serde(rename = "mapping-auto-alignment")]
    mapping_auto_alignment: Option<bool>,
    #[serde(rename = "mapping-offset")]
    mapping_offset: Option<u8>,
    #[serde(rename = "indent-interface")]
    indent_interfaces: Option<bool>,
    #[serde(rename = "space-interface-parenthesis")]
    space_interface_parenthesis: Option<bool>,
    #[serde(rename = "instance-name")]
    instance_name: Option<String>,
}

impl VhdlFormat {
    pub fn new() -> Self {
        Self {
            highlight_syntax: None,
            tab_size: None,
            type_auto_alignment: None,
            type_offset: None,
            mapping_auto_alignment: None,
            mapping_offset: None,
            indent_interfaces: None,
            space_interface_parenthesis: None,
            instance_name: None,
        }
    }

    pub fn is_syntax_highlighted(&self) -> bool {
        self.highlight_syntax.unwrap_or(true)
    }

    pub fn get_tab_size(&self) -> u8 {
        self.tab_size.unwrap_or(2)
    }

    pub fn is_indented_interfaces(&self) -> bool {
        self.indent_interfaces.unwrap_or(true)
    }

    pub fn is_auto_type_aligned(&self) -> bool {
        self.type_auto_alignment.unwrap_or(true)
    }

    pub fn get_type_offset(&self) -> u8 {
        self.type_offset.unwrap_or(1)
    }

    pub fn is_auto_mapping_aligned(&self) -> bool {
        self.mapping_auto_alignment.unwrap_or(true)
    }

    pub fn get_mapping_offset(&self) -> u8 {
        self.mapping_offset.unwrap_or(1)
    }

    pub fn is_interface_parenthesis_spaced(&self) -> bool {
        self.space_interface_parenthesis.unwrap_or(true)
    }

    pub fn get_instance_name(&self) -> String {
        self.instance_name
            .as_ref()
            .unwrap_or(&String::from("ux"))
            .clone()
    }

    /// Merges any populated data from `rhs` into attributes that do not already
    /// have data defined in `self`.
    pub fn merge(&mut self, rhs: Option<Self>) -> () {
        if let Some(rhs) = rhs {
            if self.highlight_syntax.is_some() == false {
                self.highlight_syntax = rhs.highlight_syntax
            }
            if self.tab_size.is_some() == false {
                self.tab_size = rhs.tab_size
            }
            if self.type_auto_alignment.is_some() == false {
                self.type_auto_alignment = rhs.type_auto_alignment
            }
            if self.type_offset.is_some() == false {
                self.type_offset = rhs.type_offset
            }
            if self.mapping_auto_alignment.is_some() == false {
                self.mapping_auto_alignment = rhs.mapping_auto_alignment
            }
            if self.mapping_offset.is_some() == false {
                self.mapping_offset = rhs.mapping_offset
            }
            if self.indent_interfaces.is_some() == false {
                self.indent_interfaces = rhs.indent_interfaces
            }
            if self.space_interface_parenthesis.is_some() == false {
                self.space_interface_parenthesis = rhs.space_interface_parenthesis
            }
            if self.instance_name.is_some() == false {
                self.instance_name = rhs.instance_name
            }
        }
    }
}
