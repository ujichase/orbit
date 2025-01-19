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

use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum Number {
    Decimal(String),
    Based(String),
    Real(String),
    Time(String),
    Unbased(String),
    OnlyBase(String),
}

impl Number {
    pub fn is_valid_time_units(s: &str) -> bool {
        match s {
            "s" | "ms" | "us" | "ns" | "ps" | "fs" => true,
            _ => false,
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Decimal(s) => s.to_string(),
                Self::Based(b) => b.to_string(),
                Self::Real(r) => r.to_string(),
                Self::Time(t) => t.to_string(),
                Self::Unbased(t) => t.to_string(),
                Self::OnlyBase(t) => t.to_string(),
            }
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum BaseSpec {
    Decimal(char),
    Hexadecimal(char),
    Octal(char),
    Binary(char),
}
