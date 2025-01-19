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

use super::super::sv::token::{keyword::Keyword, operator::Operator};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum VerilogError {
    #[error("an error has occurred.")]
    Unknown,
    #[error("missing closing sequence for block comment (*/)")]
    UnclosedBlockComment,
    #[error("invalid character {0}")]
    InvalidChar(char),
    #[error("invalid sequence {0}")]
    InvalidSequence(String),
    #[error("expecting closing delimiter {0}")]
    UnclosedLiteral(char),
    #[error("expecting +, -, or digit, but got {0}")]
    InvalidExponChar(char),
    #[error("expecting +, -, or digit, but got nothing")]
    EmptyExponChar,
    #[error("expecting digits for exponent value but got nothing")]
    EmptyExponNumber,
    #[error("expecting numeric value after base specifier")]
    EmptyBaseConstNumber,
    #[error("expecting base specifier for based constant")]
    MissingBaseSpecifier,
    #[error("invalid base specifier {0}")]
    InvalidBaseSpecifier(char),
    #[error("invalid character {0} after digit")]
    InvalidCharInNumber(char),
    #[error("missing digits after decimal point")]
    MissingNumbersAfterDecimalPoint,
    #[error("expecting keyword or identifier immediately after compiler directive `")]
    EmptyCompilerDirective,
    #[error("invalid syntax")]
    Vague,
    #[error("expecting \"{0}\" keyword")]
    ExpectingKeyword(Keyword),
    #[error("expecting \"{0}\" delimiter")]
    ExpectingOperator(Operator),
    #[error("incomplete module declaration")]
    ModDecIncomplete,
    #[error("unhandled dot operator in module declaration")]
    UnhandledDotInDecl,
    #[error("unhandled assignment operator in module declaration")]
    UnhandledAssignInDecl,
    #[error("expecting identifier for the module's name")]
    ModuleNameIsNotIdentifier,
}
