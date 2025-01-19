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
pub struct SystemVerilogFormat {
    #[serde(rename = "highlight-syntax")]
    highlight_syntax: Option<bool>, // TODO
    #[serde(rename = "tab-size")]
    tab_size: Option<u8>,
    #[serde(rename = "name-auto-alignment")]
    name_auto_alignment: Option<bool>,
    #[serde(rename = "name-offset")]
    name_offset: Option<u8>,
    #[serde(rename = "range-offset")]
    range_offset: Option<u8>,
    #[serde(rename = "mapping-auto-alignment")]
    mapping_auto_alignment: Option<bool>,
    #[serde(rename = "mapping-offset")]
    mapping_offset: Option<u8>,
    #[serde(rename = "instance-name")]
    instance_name: Option<String>,
}

impl SystemVerilogFormat {
    pub fn new() -> Self {
        Self {
            highlight_syntax: None,
            tab_size: None,
            name_auto_alignment: None,
            name_offset: None,
            range_offset: None,
            mapping_auto_alignment: None,
            mapping_offset: None,
            instance_name: None,
        }
    }

    pub fn is_syntax_highlighted(&self) -> bool {
        self.highlight_syntax.unwrap_or(false)
    }

    pub fn get_tab_size(&self) -> u8 {
        self.tab_size.unwrap_or(2)
    }

    pub fn is_auto_name_aligned(&self) -> bool {
        self.name_auto_alignment.unwrap_or(false)
    }

    pub fn get_name_offset(&self) -> u8 {
        self.name_offset.unwrap_or(0)
    }

    pub fn get_range_offset(&self) -> u8 {
        self.range_offset.unwrap_or(0)
    }

    pub fn is_auto_mapping_aligned(&self) -> bool {
        self.mapping_auto_alignment.unwrap_or(false)
    }

    pub fn get_mapping_offset(&self) -> u8 {
        self.mapping_offset.unwrap_or(0)
    }

    pub fn get_instance_name(&self) -> String {
        self.instance_name
            .as_ref()
            .unwrap_or(&String::from("uX"))
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
            if self.name_auto_alignment.is_some() == false {
                self.name_auto_alignment = rhs.name_auto_alignment
            }
            if self.range_offset.is_some() == false {
                self.range_offset = rhs.range_offset;
            }
            if self.name_offset.is_some() == false {
                self.name_offset = rhs.name_offset
            }
            if self.mapping_auto_alignment.is_some() == false {
                self.mapping_auto_alignment = rhs.mapping_auto_alignment
            }
            if self.mapping_offset.is_some() == false {
                self.mapping_offset = rhs.mapping_offset
            }
            if self.instance_name.is_some() == false {
                self.instance_name = rhs.instance_name
            }
        }
    }
}
