//
//  Copyright (C) 2022-2024  Chase Ruskin
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

use serde_derive::Serialize;

use super::super::error::VerilogError;
use super::token::VerilogToken;
use super::tokenizer::char_set;
use crate::core::lang::lexer::TrainCar;
use std::fmt::Display;
use std::hash::Hash;
use std::hash::Hasher;
use std::str::FromStr;

#[derive(Debug, Clone, PartialOrd, Ord, Serialize)]
#[serde(untagged)]
pub enum Identifier {
    Basic(String),
    Escaped(String),
    System(String),
    /// A module can have this name after the tokenizing when parsing a module.
    Directive(String),
}

impl Eq for Identifier {}

impl Identifier {
    pub fn new() -> Self {
        Self::Basic(String::new())
    }

    // Returns the reference to the inner `String` struct.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Basic(id) => id.as_ref(),
            Self::Escaped(id) => id.as_ref(),
            Self::System(id) => id.as_ref(),
            Self::Directive(id) => id.as_ref(),
        }
    }

    /// Modifies the ending of the identifier with `ext` and writes as a String
    pub fn into_extension(&self, ext: &str) -> Identifier {
        match self {
            Self::Basic(s) => Self::Basic(s.clone() + ext),
            Self::Escaped(s) => Self::Escaped(s.clone() + ext),
            Self::System(s) => Self::System(s.clone()),
            Self::Directive(s) => Self::Directive(s.clone()),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Basic(s) => s.len(),
            Self::Escaped(s) => s.len(),
            Self::System(s) => s.len(),
            Self::Directive(s) => s.len(),
        }
    }

    /// Checks if the identifier is a system task/function or compiler directive.
    pub fn is_nonuser_name(&self) -> bool {
        match self {
            Self::System(_) | Self::Directive(_) => true,
            _ => false,
        }
    }

    /// Checks if the identifier is a system task/function name.
    pub fn is_system_name(&self) -> bool {
        match self {
            Self::System(_) => true,
            _ => false,
        }
    }
}

// TODO: test
impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Basic(id) => id.to_lowercase().hash(state),
            Self::Escaped(id) => id.hash(state),
            Self::System(id) => format!("{}", id).hash(state),
            Self::Directive(id) => format!("`{}", id).hash(state),
        }
    }
}

impl FromStr for Identifier {
    type Err = VerilogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = TrainCar::new(s.chars());
        match chars.consume() {
            // check what type of identifier it is
            Some(c) => Ok(match c {
                '\\' => Self::Escaped(VerilogToken::consume_value_pattern(
                    &mut chars,
                    None,
                    char_set::is_not_whitespace,
                )?),
                '$' => Self::System(VerilogToken::consume_value_pattern(
                    &mut chars,
                    None,
                    char_set::is_identifier_character,
                )?),
                _ => {
                    // verify the first character was a letter or underscore
                    match char_set::is_letter(&c) || c == char_set::UNDER_SCORE {
                        true => Self::Basic(VerilogToken::consume_value_pattern(
                            &mut chars,
                            Some(c),
                            char_set::is_identifier_character,
                        )?),
                        false => return Err(VerilogError::InvalidChar(c)),
                    }
                }
            }),
            None => panic!("no more characters"),
        }
    }
}

impl std::cmp::PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        // compare the two strings
        if self.is_nonuser_name() == true && other.is_nonuser_name() == true {
            self.as_str() == other.as_str()
        } else if self.is_nonuser_name() == true || other.is_nonuser_name() == true {
            false
        } else {
            self.as_str() == other.as_str()
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(id) => write!(f, "{}", id),
            Self::Escaped(id) => write!(f, "\\{}", id),
            Self::System(id) => write!(f, "{}", id),
            Self::Directive(id) => write!(f, "`{}", id),
        }
    }
}
