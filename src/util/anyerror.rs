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

use std::error::Error;
use std::fmt::Display;

/// Quickly implement a custom/unique error message.
///
/// Can also be used to wrap an error's message.
#[derive(Debug, PartialEq)]
pub struct AnyError(pub String);

impl Error for AnyError {}

impl Display for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Fault> for AnyError {
    fn from(value: Fault) -> Self {
        AnyError(value.to_string())
    }
}

impl From<&str> for AnyError {
    fn from(value: &str) -> Self {
        AnyError(value.to_string())
    }
}

pub type Fault = Box<dyn Error>;

#[derive(Debug, thiserror::Error)]
/// Stores the (source code file, error message)
pub struct CodeFault(pub Option<String>, pub Fault);

impl From<Fault> for CodeFault {
    fn from(value: Fault) -> Self {
        Self(None, value)
    }
}

impl CodeFault {
    /// Checks if there is a source code parsing error.
    pub fn is_source_err(&self) -> bool {
        self.0.is_some()
    }

    /// References the source code file that produced an error, it exists.
    pub fn as_source_file(&self) -> Option<&String> {
        self.0.as_ref()
    }

    pub fn into_fault(self) -> Fault {
        self.1
    }
}

impl Display for CodeFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(src) => write!(f, "failed to read file {:?}: {}", src, self.1),
            None => write!(f, "{}", self.1),
        }
    }
}
