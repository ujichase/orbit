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

use super::super::super::lexer::Position;
use super::super::token::char_set;
use crate::core::lang::highlight;
use crate::core::lang::highlight::ToColor;
use colored::ColoredString;
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum Comment {
    Single(String),
    Delimited(String),
}

impl Comment {
    fn as_str(&self) -> &str {
        match self {
            Self::Single(note) => note.as_ref(),
            Self::Delimited(note) => note.as_ref(),
        }
    }

    /// Computes the ending position the cursor ends up in.
    pub fn ending_position(&self) -> Position {
        // begin with counting the opening delimiters (-- or /*)
        let mut pos = Position::place(1, 2);
        let mut chars = self.as_str().chars();
        while let Some(c) = chars.next() {
            if char_set::is_newline(&c) == true {
                pos.next_line();
            } else {
                pos.next_col();
            }
        }
        match self {
            Self::Single(_) => (),
            // increment to handle the closing delimiters */
            Self::Delimited(_) => {
                pos.next_col();
                pos.next_col();
            }
        }
        pos
    }
}

impl Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(c) => write!(f, "--{}", c),
            Self::Delimited(c) => write!(f, "/*{}*/", c),
        }
    }
}

impl ToColor for Comment {
    fn to_color(&self) -> ColoredString {
        highlight::style::comment(&self.to_string())
    }
}
