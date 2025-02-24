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

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum VhdlError {
    #[error("{0}")]
    Any(String),
    #[error("invalid character {0}")]
    Invalid(String),
    #[error("missing and empty {0}")]
    MissingAndEmpty(char),
    #[error("expecting closing {0} but got {1}")]
    MissingClosingAndGot(char, char),
    #[error("invalid syntax")]
    Vague,
    #[error("empty identifier")]
    IdEmpty,
    #[error("characters \'{0}\' found following closing extended backslash")]
    IdCharsAfterDelim(String),
    #[error("first character must be a letter but found \'{0}\'")]
    IdInvalidFirstChar(char),
}
