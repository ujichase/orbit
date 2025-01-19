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

use super::lexer::Token;
use thiserror::Error;

pub trait Parse<T> {
    type SymbolType;
    type SymbolError;

    fn parse(tokens: Vec<Token<T>>) -> Vec<Result<Symbol<Self::SymbolType>, Self::SymbolError>>;
}

#[derive(Debug, PartialEq)]
pub struct Symbol<T> {
    stype: T,
}

impl<T> Symbol<T> {
    /// Creates a new symbol.
    pub fn new(stype: T) -> Self {
        Self { stype: stype }
    }

    pub fn take(self) -> T {
        self.stype
    }

    pub fn as_ref(&self) -> &T {
        &self.stype
    }

    pub fn as_ref_mut(&mut self) -> &mut T {
        &mut self.stype
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum ParseError {
    #[error("Failed to parse file \"{0}\": {1}")]
    SourceCodeError(String, String),
}
