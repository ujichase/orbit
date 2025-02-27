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

use crate::core::lang::highlight;
use crate::core::lang::highlight::ToColor;
use colored::ColoredString;
use colored::Colorize;
use std::fmt::Display;

use crate::core::lang::{
    lexer::TrainCar,
    sv::error::SystemVerilogError,
    verilog::token::{
        identifier::Identifier,
        number::Number,
        token::{Comment, VerilogToken},
        tokenizer::char_set,
    },
};

use super::{keyword::Keyword, operator::Operator};

#[derive(Debug, PartialEq, Clone)]
pub enum SystemVerilogToken {
    Comment(Comment),
    Operator(Operator),
    Number(Number),
    Identifier(Identifier),
    Keyword(Keyword),
    Directive(String),
    StringLiteral(String),
    EOF,
}

impl ToColor for SystemVerilogToken {
    fn to_color(&self) -> ColoredString {
        match &self {
            Self::Comment(c) => c.to_color(),
            Self::Operator(o) => o.to_color(),
            Self::Number(n) => n.to_color(),
            Self::Identifier(i) => i.to_color(),
            Self::Keyword(k) => k.to_color(),
            Self::Directive(_) => highlight::style::string(&self.to_string()),
            Self::StringLiteral(_) => highlight::style::string(&self.to_string()),
            Self::EOF => String::new().normal(),
        }
    }
}

impl From<VerilogToken> for SystemVerilogToken {
    fn from(value: VerilogToken) -> Self {
        match value {
            VerilogToken::Comment(t) => Self::Comment(t),
            VerilogToken::Identifier(t) => Self::Identifier(t),
            VerilogToken::Directive(t) => Self::Directive(t),
            VerilogToken::Operator(t) => {
                Self::Operator(Operator::transform(&t.to_string()).unwrap())
            }
            VerilogToken::StringLiteral(t) => Self::StringLiteral(t),
            VerilogToken::EOF => Self::EOF,
            VerilogToken::Number(t) => Self::Number(t),
            VerilogToken::Keyword(t) => {
                Self::Keyword(Keyword::match_keyword(&t.to_string()).unwrap())
            }
        }
    }
}

impl Display for SystemVerilogToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Comment(c) => c.to_string(),
                Self::Operator(o) => o.to_string(),
                Self::Number(n) => n.to_string(),
                Self::Identifier(i) => i.to_string(),
                Self::Keyword(k) => k.to_string(),
                Self::StringLiteral(s) => format!("\"{}\"", s.to_string()),
                Self::Directive(d) => format!("`{}", d.to_string()),
                Self::EOF => String::new(),
            }
        )
    }
}

impl SystemVerilogToken {
    /// Checks if the element is a particular keyword `kw`.
    pub fn check_keyword(&self, kw: &Keyword) -> bool {
        match self {
            SystemVerilogToken::Keyword(r) => r == kw,
            _ => false,
        }
    }

    pub fn is_eof(&self) -> bool {
        match self {
            SystemVerilogToken::EOF => true,
            _ => false,
        }
    }

    pub fn is_directive(&self) -> bool {
        match self {
            SystemVerilogToken::Directive(_) => true,
            _ => false,
        }
    }

    /// Accesses the underlying `Identifier`, if one exists.
    pub fn as_identifier(&self) -> Option<&Identifier> {
        match self {
            SystemVerilogToken::Identifier(id) => match id {
                Identifier::Basic(_) | Identifier::Escaped(_) => Some(id),
                _ => None,
            },
            _ => None,
        }
    }

    /// Accesses the underlying `Number`, if one exists.
    pub fn as_number(&self) -> Option<&Number> {
        match self {
            SystemVerilogToken::Number(num) => Some(num),
            _ => None,
        }
    }

    /// Checks if the element is a particular delimiter `d`.
    pub fn check_delimiter(&self, d: &Operator) -> bool {
        match self {
            SystemVerilogToken::Operator(r) => r == d,
            _ => false,
        }
    }

    pub fn as_comment(&self) -> Option<&Comment> {
        match self {
            SystemVerilogToken::Comment(r) => Some(r),
            _ => None,
        }
    }

    pub fn is_comment(&self) -> bool {
        match self {
            SystemVerilogToken::Comment(_) => true,
            _ => false,
        }
    }

    /// Takes the identifier from the token.
    pub fn take_identifier(self) -> Option<Identifier> {
        match self {
            Self::Identifier(i) => Some(i),
            _ => None,
        }
    }

    /// Takes the keyword from the token.
    pub fn take_keyword(self) -> Option<Keyword> {
        match self {
            Self::Keyword(kw) => Some(kw),
            _ => None,
        }
    }

    /// Takes the delimter from the token.
    pub fn takes_delimiter(self) -> Option<Operator> {
        match self {
            Self::Operator(d) => Some(d),
            _ => None,
        }
    }

    /// Casts into a keyword.
    pub fn as_keyword(&self) -> Option<&Keyword> {
        match self {
            Self::Keyword(kw) => Some(kw),
            _ => None,
        }
    }

    /// Checks if the current token type `self` is a delimiter.
    pub fn is_delimiter(&self) -> bool {
        match self {
            Self::Operator(_) => true,
            _ => false,
        }
    }

    /// Casts as a delimiter
    pub fn as_delimiter(&self) -> Option<&Operator> {
        match self {
            Self::Operator(d) => Some(d),
            _ => None,
        }
    }
}

impl SystemVerilogToken {
    /// Captures SystemVerilog Tokens: keywords and basic identifiers.
    ///
    /// Assumes the first `letter` char was the last char consumed before the function call.
    pub fn consume_word(
        train: &mut TrainCar<impl Iterator<Item = char>>,
        c0: char,
    ) -> Result<Self, SystemVerilogError> {
        let word = VerilogToken::consume_value_pattern(
            train,
            Some(c0),
            char_set::is_identifier_character,
        )?;
        if c0 == char_set::UNDER_SCORE {
            Ok(Self::Identifier(Identifier::Basic(word)))
        } else if c0 == char_set::DOLLAR_SIGN {
            Ok(Self::Identifier(Identifier::System(word)))
        } else {
            match Keyword::match_keyword(&word) {
                Some(kw) => Ok(Self::Keyword(kw)),
                None => Ok(Self::Identifier(Identifier::Basic(word))),
            }
        }
    }

    /// Walks through the possible interpretations for capturing a Verilog delimiter.
    ///
    /// If it successfully finds a valid Verilog delimiter, it will move the `loc` the number
    /// of characters it consumed.
    pub fn consume_operator(
        train: &mut TrainCar<impl Iterator<Item = char>>,
        c0: Option<char>,
    ) -> Result<Self, SystemVerilogError> {
        // delimiter will have at most 3 characters
        let mut op_buf = String::with_capacity(4);
        if let Some(c) = c0 {
            op_buf.push(c);
        };
        // check the next character in the sequence
        while let Some(c) = train.peek() {
            match op_buf.len() {
                0 => match c {
                    // ambiguous characters...read another character (could be a 2 or 3 length operator)
                    '(' | ')' | '{' | '}' | '*' | '>' | '<' | '&' | '|' | '=' | '!' | '^' | '~'
                    | '+' | '-' | '?' | '%' | ':' | '/' | '.' => {
                        op_buf.push(train.consume().unwrap())
                    }
                    // if it was an operator, take the character and increment the location
                    _ => return Self::match_delimiter(&String::from(train.consume().unwrap())),
                },
                1 => match op_buf.chars().nth(0).unwrap() {
                    '!' => match c {
                        // move on to next round (is a 2 or 3 length delimiter)
                        '=' => op_buf.push(train.consume().unwrap()),
                        // stop at 1
                        _ => return Ok(Self::match_delimiter(&op_buf)?),
                    },
                    '<' => match c {
                        // move on to next round (is a 2 or 3 length delimiter)
                        '<' | '=' | '-' => op_buf.push(train.consume().unwrap()),
                        // stop at 1
                        _ => return Ok(Self::match_delimiter(&op_buf)?),
                    },
                    '>' => match c {
                        // move on to next round (is a 2 or 3 length delimiter)
                        '>' | '=' => op_buf.push(train.consume().unwrap()),
                        // stop at 1
                        _ => return Ok(Self::match_delimiter(&op_buf)?),
                    },
                    '=' => match c {
                        // move on to next round (is a 2 or 3 length delimiter)
                        '=' => op_buf.push(train.consume().unwrap()),
                        // stop at 1
                        _ => return Ok(Self::match_delimiter(&op_buf)?),
                    },
                    _ => {
                        // try with 2
                        op_buf.push(*c);
                        if let Ok(op) = Self::match_delimiter(&op_buf) {
                            train.consume();
                            return Ok(op);
                        } else {
                            // revert back to 1
                            op_buf.pop();
                            return Self::match_delimiter(&op_buf);
                        }
                    }
                },
                2 => {
                    match op_buf.chars().nth(1).unwrap() {
                        '>' => match c {
                            // move on to next round (is a 3 or 4 length delimiter)
                            '>' => op_buf.push(train.consume().unwrap()),
                            // stop at 2
                            _ => return Ok(Self::match_delimiter(&op_buf))?,
                        },
                        '<' => match c {
                            // move on to next round (is a 3 or 4 length delimiter)
                            '<' => op_buf.push(train.consume().unwrap()),
                            // stop at 2
                            _ => return Ok(Self::match_delimiter(&op_buf))?,
                        },
                        _ => {
                            // try with 3
                            op_buf.push(*c);
                            if let Ok(op) = Self::match_delimiter(&op_buf) {
                                train.consume();
                                return Ok(op);
                            } else {
                                // revert back to 2 (guaranteed to exist)
                                op_buf.pop();
                                return Ok(Self::match_delimiter(&op_buf)?);
                            }
                        }
                    }
                }
                3 => {
                    // try with 4
                    op_buf.push(*c);
                    if let Ok(op) = Self::match_delimiter(&op_buf) {
                        train.consume();
                        return Ok(op);
                    } else {
                        // revert back to 2 (guaranteed to exist)
                        op_buf.pop();
                        return Ok(Self::match_delimiter(&op_buf)?);
                    }
                }
                _ => panic!("Operator matching exceeds 3 characters"),
            }
        }
        // try when hiting end of stream
        Self::match_delimiter(&op_buf)
    }

    /// Attempts to match a string `s` to a valid delimiter.
    fn match_delimiter(s: &str) -> Result<Self, SystemVerilogError> {
        match Operator::transform(s) {
            Some(d) => Ok(Self::Operator(d)),
            None => Err(SystemVerilogError::InvalidSequence(s.to_string())),
        }
    }
}
