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

use super::super::super::lexer::TrainCar;
use super::VhdlError;
use crate::core::lang::highlight;
use crate::core::lang::highlight::ToColor;
use crate::core::lang::LangIdentifier;
use crate::core::pkgid::PkgPart;
use crate::util::strcmp;
use colored::ColoredString;
use serde_derive::Serialize;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;

use std::hash::Hasher;
use std::str::FromStr;

use crate::core::lang::vhdl::token::char_set;
use crate::core::lang::vhdl::token::VhdlToken;

#[derive(Debug, Clone, PartialOrd, Ord, Serialize)]
#[serde(untagged)]
pub enum Identifier {
    Basic(String),
    Extended(String),
}

impl std::cmp::Eq for Identifier {}

impl Identifier {
    pub fn into_lang_id(self) -> LangIdentifier {
        LangIdentifier::Vhdl(self)
    }

    /// Creates an empty basic identifier.
    pub fn new() -> Self {
        Self::Basic(String::new())
    }

    /// Creates a new basic identifier for the working library: `work`.
    pub fn new_working() -> Self {
        Self::Basic(String::from("work"))
    }

    // Returns the reference to the inner `String` struct.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Basic(name) => name.as_ref(),
            Self::Extended(name) => name.as_ref(),
        }
    }

    /// Modifies the ending of the identifier with `ext` and writes as a String
    pub fn into_extension(&self, ext: &str) -> Identifier {
        match self {
            Self::Basic(s) => Self::Basic(s.clone() + ext),
            Self::Extended(s) => Self::Extended(s.clone() + ext),
        }
    }

    /// Checks if `self` is an extended identifier or not.
    fn is_extended(&self) -> bool {
        match self {
            Self::Extended(_) => true,
            Self::Basic(_) => false,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Basic(id) => id.len(),
            Self::Extended(id) => id.len() + 2 + (id.chars().filter(|c| c == &'\\').count()),
        }
    }
}

// TODO: make sure this works as intended (collisions should occur for basic names)
impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Basic(id) => id.to_lowercase().hash(state),
            Self::Extended(id) => id.hash(state),
        }
    }
}

impl From<&PkgPart> for Identifier {
    fn from(part: &PkgPart) -> Self {
        Identifier::Basic(part.to_normal().to_string())
    }
}

impl FromStr for Identifier {
    type Err = VhdlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = TrainCar::new(s.chars());
        match chars.consume() {
            // check what type of identifier it is
            Some(c) => Ok(match c {
                '\\' => {
                    let result = Self::Extended(
                        VhdlToken::consume_literal(&mut chars, &char_set::BACKSLASH).unwrap(),
                    );
                    // gather remaining characters
                    let mut rem = String::new();
                    while let Some(c) = chars.consume() {
                        rem.push(c);
                    }
                    match rem.is_empty() {
                        true => result,
                        false => return Err(Self::Err::IdCharsAfterDelim(rem)),
                    }
                }
                _ => {
                    // verify the first character was a letter
                    match char_set::is_letter(&c) {
                        true => Self::Basic(
                            VhdlToken::consume_value_pattern(
                                &mut chars,
                                Some(c),
                                char_set::is_letter_or_digit,
                            )
                            .unwrap(),
                        ),
                        false => return Err(Self::Err::IdInvalidFirstChar(c)),
                    }
                }
            }),
            None => Err(Self::Err::IdEmpty),
        }
    }
}

impl std::cmp::PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        // instantly not equal if not they are not of same type
        if self.is_extended() != other.is_extended() {
            return false;
        };
        // compare with case sensitivity
        if self.is_extended() == true {
            self.as_str() == other.as_str()
        // compare without case sensitivity
        } else {
            strcmp::cmp_ignore_case(self.as_str(), other.as_str())
        }
    }

    fn ne(&self, other: &Self) -> bool {
        self.eq(other) == false
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(id) => write!(f, "{}", id),
            Self::Extended(id) => write!(f, "\\{}\\", id.replace('\\', r#"\\"#)),
        }
    }
}

impl ToColor for Identifier {
    fn to_color(&self) -> ColoredString {
        highlight::style::identifier(&self.to_string())
    }
}
