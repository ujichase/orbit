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

pub mod char_set {
    pub const DOUBLE_QUOTE: char = '\"';
    pub const BACKSLASH: char = '\\';
    pub const STAR: char = '*';
    pub const DASH: char = '-';
    pub const FWDSLASH: char = '/';
    pub const UNDERLINE: char = '_';
    pub const SINGLE_QUOTE: char = '\'';
    pub const DOT: char = '.';
    pub const HASH: char = '#';
    pub const COLON: char = ':';
    pub const PLUS: char = '+';

    /// Checks if `c` is a space according to VHDL-2008 LRM p225.
    /// Set: space, nbsp
    pub fn is_space(c: &char) -> bool {
        c == &'\u{0020}' || c == &'\u{00A0}'
    }

    /// Checks if `c` is a digit according to VHDL-2008 LRM p225.
    pub fn is_digit(c: &char) -> bool {
        match c {
            '0'..='9' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a graphic character according to VHDL-2008 LRM p230.
    /// - rule ::= upper_case_letter | digit | special_character | space_character
    /// | lower_case_letter | other_special_character
    pub fn is_graphic(c: &char) -> bool {
        is_lower(&c)
            || is_upper(&c)
            || is_digit(&c)
            || is_special(&c)
            || is_other_special(&c)
            || is_space(&c)
    }

    /// Checks if `c` is an upper-case letter according to VHDL-2019 LRM p257.
    /// Set: `ABCDEFGHIJKLMNOPQRSTUVWXYZÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖØÙÚÛÜÝÞ`
    pub fn is_upper(c: &char) -> bool {
        match c {
            '\u{00D7}' => false, // reject multiplication sign
            'A'..='Z' | 'À'..='Þ' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a new-line character.
    pub fn is_newline(c: &char) -> bool {
        c == &'\n'
    }

    /// Checks if `c` is a special character according to VHDL-2008 LRM p225.
    /// Set: `"#&'()*+,-./:;<=>?@[]_`|`
    pub fn is_special(c: &char) -> bool {
        match c {
            '"' | '#' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.' | '/' | ':' | ';'
            | '<' | '=' | '>' | '?' | '@' | '[' | ']' | '_' | '`' | '|' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a graphic character according to VHDL-2008 LRM p225 and
    /// is NOT a double character ".
    ///
    /// This function is exclusively used in the logic for collecting a bit string literal.
    pub fn is_graphic_and_not_double_quote(c: &char) -> bool {
        c != &DOUBLE_QUOTE && is_graphic(&c)
    }

    /// Checks if `c` is an "other special character" according to VHDL-2008 LRM p225.
    /// Set: `!$%\^{} ~¡¢£¤¥¦§ ̈©a«¬® ̄°±23 ́μ¶· ̧1o»1⁄41⁄23⁄4¿×÷-`
    pub fn is_other_special(c: &char) -> bool {
        match c {
            '!'
            | '$'
            | '%'
            | '\\'
            | '^'
            | '{'
            | '}'
            | ' '
            | '~'
            | '-'
            | '\u{00A1}'..='\u{00BF}'
            | '\u{00D7}'
            | '\u{00F7}' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a lower-case letter according to VHDL-2019 LRM p257.
    /// Set: `abcdefghijklmnopqrstuvwxyzßàáâãäåæçèéêëìíîïðñòóôõöøùúûüýþÿ`
    pub fn is_lower(c: &char) -> bool {
        match c {
            '\u{00F7}' => false, // reject division sign
            'a'..='z' | 'ß'..='ÿ' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a letter according to VHDL-2019 LRM p257.
    pub fn is_letter(c: &char) -> bool {
        is_lower(&c) || is_upper(&c)
    }

    /// Checks if `c` is a digit | letter according to VHDL-2008 LRM p230.
    pub fn is_extended_digit(c: &char) -> bool {
        is_digit(&c) || is_letter(&c)
    }

    /// Checks if `c` is a digit | letter according to VHDL-2008 LRM p229.
    pub fn is_letter_or_digit(c: &char) -> bool {
        is_digit(&c) || is_letter(&c)
    }

    /// Checks if the character is a seperator according to VHDL-2019 LRM p259.
    pub fn is_separator(c: &char) -> bool {
        // whitespace: space, nbsp
        c == &'\u{0020}' || c == &'\u{00A0}' ||
        // format-effectors: ht (\t), vt, cr (\r), lf (\n)
        c == &'\u{0009}' || c == &'\u{000B}' || c == &'\u{000D}' || c == &'\u{000A}'
    }
}

use super::super::super::lexer::{Token, TokenError};

use super::super::error::VhdlError;
use super::super::token::VhdlToken;
use crate::core::lang::lexer::Tokenize;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
struct VHDLElement(Result<Token<VhdlToken>, TokenError<VhdlError>>);

#[derive(PartialEq)]
pub struct VhdlTokenizer {
    tokens: Vec<VHDLElement>,
}

impl FromStr for VhdlTokenizer {
    type Err = VhdlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_source_code(s))
    }
}

impl VhdlTokenizer {
    /// Creates a new `VHDLTokenizer` struct.
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    /// Generates a `VHDLTokenizer` struct from source code `s`.
    ///
    /// @TODO If `skip_err` is true, it will silently omit erroneous parsing from the
    /// final vector and guarantee to be `Ok`.
    pub fn from_source_code(s: &str) -> Self {
        Self {
            tokens: Self::tokenize(s)
                .into_iter()
                .map(|f| VHDLElement(f))
                .collect(),
        }
    }

    /// Transforms the list of results into a list of tokens, silently skipping over
    /// errors.
    ///
    /// This `fn` also filters out `Comment`s. To include `Comment` tokens, see
    /// `into_tokens_all`.
    pub fn into_tokens(self) -> Vec<Token<VhdlToken>> {
        self.tokens
            .into_iter()
            .filter_map(|f| match f.0 {
                Ok(t) => match t.as_ref() {
                    VhdlToken::Comment(_) => None,
                    _ => Some(t),
                },
                Err(_) => None,
            })
            .collect()
    }

    /// Transforms the list of results into a list of tokens, silently skipping over
    /// errors.
    pub fn into_tokens_all(self) -> Vec<Token<VhdlToken>> {
        self.tokens
            .into_iter()
            .filter_map(|f| match f.0 {
                Ok(t) => Some(t),
                Err(_) => None,
            })
            .collect()
    }

    /// Transforms the list of results into a list of tokens, silently skipping over
    /// errors.
    pub fn as_tokens_all(&self) -> Vec<&Token<VhdlToken>> {
        self.tokens
            .iter()
            .filter_map(|f| match &f.0 {
                Ok(t) => Some(t),
                Err(_) => None,
            })
            .collect()
    }
}

impl std::fmt::Debug for VhdlTokenizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for tk in &self.tokens {
            write!(
                f,
                "{}\t{:?}\n",
                tk.0.as_ref().unwrap().locate(),
                tk.0.as_ref().unwrap()
            )?
        }
        Ok(())
    }
}

use crate::core::lang::lexer::TrainCar;

impl Tokenize for VhdlTokenizer {
    type TokenType = VhdlToken;
    type Err = VhdlError;

    fn tokenize(s: &str) -> Vec<Result<Token<Self::TokenType>, TokenError<Self::Err>>> {
        let mut train = TrainCar::new(s.chars());
        // store results here as we consume the characters
        let mut tokens: Vec<Result<Token<Self::TokenType>, TokenError<Self::Err>>> = Vec::new();
        // consume every character (lexical analysis)
        while let Some(c) = train.consume() {
            // skip over whitespace
            if char_set::is_separator(&c) {
                continue;
            }
            let tk_loc = train.locate().clone();
            // build a token
            tokens.push(if char_set::is_letter(&c) {
                // collect general identifier
                match Self::TokenType::consume_word(&mut train, c) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => Err(TokenError::new(e, train.locate().clone())),
                }
            } else if c == char_set::BACKSLASH {
                // collect extended identifier
                match Self::TokenType::consume_extended_identifier(&mut train) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => Err(TokenError::new(e, train.locate().clone())),
                }
            } else if c == char_set::DOUBLE_QUOTE {
                // collect string literal
                match Self::TokenType::consume_str_lit(&mut train) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => Err(TokenError::new(e, train.locate().clone())),
                }
            } else if c == char_set::SINGLE_QUOTE
                && tokens.last().is_some()
                && tokens.last().unwrap().as_ref().is_ok()
                && tokens
                    .last()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .is_delimiter()
            {
                // collect character literal
                match Self::TokenType::consume_char_lit(&mut train) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => Err(TokenError::new(e, train.locate().clone())),
                }
            } else if char_set::is_digit(&c) {
                // collect decimal literal (or bit string literal or based literal)
                match Self::TokenType::consume_numeric(&mut train, c) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => Err(TokenError::new(e, train.locate().clone())),
                }
            } else if c == char_set::DASH
                && train.peek().is_some()
                && train.peek().unwrap() == &char_set::DASH
            {
                // collect a single-line comment
                match Self::TokenType::consume_comment(&mut train) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => Err(TokenError::new(e, train.locate().clone())),
                }
            } else if c == char_set::FWDSLASH
                && train.peek().is_some()
                && train.peek().unwrap() == &char_set::STAR
            {
                // collect delimited (multi-line) comment
                match Self::TokenType::consume_delim_comment(&mut train) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => {
                        let mut tk_loc = train.locate().clone();
                        tk_loc.next_col(); // +1 col for correct alignment
                        Err(TokenError::new(e, tk_loc))
                    }
                }
            } else {
                // collect delimiter
                match Self::TokenType::collect_delimiter(&mut train, Some(c)) {
                    Ok(tk) => Ok(Token::new(tk, tk_loc)),
                    Err(e) => Err(TokenError::new(e, train.locate().clone())),
                }
            });
        }
        // push final EOF token
        let mut tk_loc = train.locate().clone();
        tk_loc.next_col();
        tokens.push(Ok(Token::new(VhdlToken::EOF, tk_loc)));
        tokens
    }
}
