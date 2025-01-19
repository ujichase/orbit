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

use super::super::super::lexer;
use super::super::super::lexer::Token;
use super::super::super::lexer::Tokenize;
use super::super::super::lexer::TrainCar;
use super::super::error::VerilogError;
use super::token::*;
use lexer::TokenError;
use std::str::FromStr;

pub mod char_set {
    pub const DOUBLE_QUOTE: char = '\"';
    pub const STAR: char = '*';
    pub const FWD_SLASH: char = '/';
    pub const SINGLE_QUOTE: char = '\'';
    pub const PLUS: char = '+';
    pub const MINUS: char = '-';
    pub const UNDER_SCORE: char = '_';
    pub const DOLLAR_SIGN: char = '$';
    pub const GRAVE_ACCENT: char = '`';
    pub const BIG_E: char = 'E';
    pub const LIL_E: char = 'e';
    pub const ESC: char = '\\';
    pub const DOT: char = '.';

    /// Checks if `c` is a lower-case letter.
    /// Set: `abcdefghijklmnopqrstuvwxyzßàáâãäåæçèéêëìíîïðñòóôõöøùúûüýþÿ`
    pub fn is_lower(c: &char) -> bool {
        match c {
            '\u{00F7}' => false, // reject division sign
            'a'..='z' | 'ß'..='ÿ' => true,
            _ => false,
        }
    }

    /// Checks if `c` is an upper-case letter.
    /// Set: `ABCDEFGHIJKLMNOPQRSTUVWXYZÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖØÙÚÛÜÝÞ`
    pub fn is_upper(c: &char) -> bool {
        match c {
            '\u{00D7}' => false, // reject multiplication sign
            'A'..='Z' | 'À'..='Þ' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a letter.
    pub fn is_letter(c: &char) -> bool {
        is_lower(&c) || is_upper(&c)
    }

    /// Checks if `c` is a new-line character.
    pub fn is_newline(c: &char) -> bool {
        c == &'\n'
    }

    /// Checks if `c` is a digit | letter | underscore.
    pub fn is_letter_or_digit_or_underscore(c: &char) -> bool {
        is_digit(&c) || is_letter(&c) || c == &UNDER_SCORE
    }

    pub fn is_digit_or_underscore(c: &char) -> bool {
        is_digit(&c) || c == &UNDER_SCORE
    }

    /// The set of characters \[a-z]\[A-Z]\[0-9]\[_]\[$] are allowed in identifiers
    /// after the initial letter is captured.
    pub fn is_identifier_character(c: &char) -> bool {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '$' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a digit.
    pub fn is_digit(c: &char) -> bool {
        match c {
            '0'..='9' => true,
            _ => false,
        }
    }

    pub fn is_hex(c: &char) -> bool {
        match c {
            'a'..='f' | 'A'..='F' => true,
            _ => false,
        }
    }

    pub fn is_digit_or_underscore_or_signal_char(c: &char) -> bool {
        is_digit(c)
            || is_hex(c)
            || c == &UNDER_SCORE
            || c == &'x'
            || c == &'X'
            || c == &'z'
            || c == &'Z'
            || c == &'?'
    }

    // pg. 8: White space shall contain the characters for spaces, tabs, newlines, and formfeeds.
    pub fn is_whitespace(c: &char) -> bool {
        // whitespace: space, nbsp
        c == &'\u{0020}' || c == &'\u{00A0}' ||
        // format-effectors: ht (\t), vt, cr (\r), lf (\n)
        c == &'\u{0009}' || c == &'\u{000B}' || c == &'\u{000D}' || c == &'\u{000A}'
    }

    pub fn is_not_whitespace(c: &char) -> bool {
        is_whitespace(c) == false
    }
}

#[derive(Debug, PartialEq)]
struct VerilogElement(Result<lexer::Token<VerilogToken>, lexer::TokenError<VerilogError>>);

#[derive(PartialEq)]
pub struct VerilogTokenizer {
    tokens: Vec<VerilogElement>,
}

impl Tokenize for VerilogTokenizer {
    type TokenType = VerilogToken;
    type Err = VerilogError;

    fn tokenize(
        s: &str,
    ) -> Vec<Result<lexer::Token<Self::TokenType>, lexer::TokenError<Self::Err>>> {
        let mut train = TrainCar::new(s.chars());
        // store results here as we consume the characters
        let mut tokens: Vec<Result<Token<Self::TokenType>, TokenError<Self::Err>>> = Vec::new();
        // consume every character (lexical analysis)
        while let Some(c) = train.consume() {
            // skip over whitespace
            if char_set::is_whitespace(&c) == true {
                continue;
            }
            let tk_loc = train.locate().clone();
            // peek at next character
            let next = train.peek();
            // add a token to the list
            tokens.push(
                if char_set::is_letter(&c) == true || char_set::UNDER_SCORE == c {
                    // collect keyword or identifier
                    match Self::TokenType::consume_word(&mut train, c) {
                        Ok(tk) => Ok(Token::new(tk, tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                } else if char_set::ESC == c {
                    // collect identifier (escaped)
                    match Self::TokenType::consume_escaped_identifier(&mut train) {
                        Ok(tk) => Ok(Token::new(tk, tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                } else if char_set::DOUBLE_QUOTE == c {
                    // collect a string literal
                    match Self::TokenType::consume_str_literal(&mut train) {
                        Ok(tk) => Ok(Token::new(tk, tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                } else if char_set::is_digit(&c) == true
                    || char_set::SINGLE_QUOTE == c
                    || ((char_set::PLUS == c || char_set::MINUS == c)
                        && next.is_some_and(|d| char_set::is_digit(&d) == true))
                {
                    // collect a number
                    match Self::TokenType::consume_number(&mut train, c) {
                        Ok(tk) => Ok(Token::new(tk, tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                } else if char_set::FWD_SLASH == c
                    && next.is_some_and(|d| d == &char_set::FWD_SLASH)
                {
                    // collect single-line comment
                    match Self::TokenType::consume_oneline_comment(&mut train) {
                        Ok(tk) => Ok(Token::new(tk, tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                } else if char_set::FWD_SLASH == c && next.is_some_and(|f| f == &char_set::STAR) {
                    // collect block comment
                    match Self::TokenType::consume_block_comment(&mut train) {
                        Ok(tk) => Ok(Token::new(tk, tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                } else if char_set::DOLLAR_SIGN == c {
                    // collect system task/function identifier
                    match Self::TokenType::consume_word(&mut train, c) {
                        Ok(tk) => Ok(Token::new(tk, tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                    // todo!("collect system task")
                } else if char_set::GRAVE_ACCENT == c {
                    // collect compiler directive
                    match Self::TokenType::consume_compiler_directive(&mut train) {
                        Ok(tk) => Ok(Token::new(tk, tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                } else {
                    // collect operator/delimiter
                    match Self::TokenType::consume_operator(&mut train, Some(c)) {
                        Ok(tk) => Ok(Token::new(tk, tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                },
            );
        }
        // push final EOF token
        let mut tk_loc = train.locate().clone();
        tk_loc.next_col();
        tokens.push(Ok(Token::new(VerilogToken::EOF, tk_loc)));
        tokens
    }
}

impl FromStr for VerilogTokenizer {
    type Err = VerilogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_source_code(s))
    }
}

impl VerilogTokenizer {
    /// Creates a new `VerilogTokenizer` struct.
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    /// Generates a `VerilogTokenizer` struct from source code `s`.
    ///
    /// @TODO If `skip_err` is true, it will silently omit erroneous parsing from the
    /// final vector and guarantee to be `Ok`.
    pub fn from_source_code(s: &str) -> Self {
        Self {
            tokens: Self::tokenize(s)
                .into_iter()
                .map(|f| VerilogElement(f))
                .collect(),
        }
    }

    /// Transforms the list of results into a list of tokens, silently skipping over
    /// errors.
    ///
    /// This `fn` also filters out `Comment`s. To include `Comment` tokens, see
    /// `into_tokens_all`.
    pub fn into_tokens(self) -> Vec<lexer::Token<VerilogToken>> {
        self.tokens
            .into_iter()
            .filter_map(|f| match f.0 {
                Ok(t) => match t.as_ref() {
                    VerilogToken::Comment(_) => None,
                    _ => Some(t),
                },
                Err(_) => None,
            })
            .collect()
    }

    /// Transforms the list of results into a list of tokens, silently skipping over
    /// errors.
    pub fn into_tokens_all(self) -> Vec<lexer::Token<VerilogToken>> {
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
    pub fn as_tokens_all(&self) -> Vec<&lexer::Token<VerilogToken>> {
        self.tokens
            .iter()
            .filter_map(|f| match &f.0 {
                Ok(t) => Some(t),
                Err(_) => None,
            })
            .collect()
    }
}

impl std::fmt::Debug for VerilogTokenizer {
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

#[cfg(test)]
mod tests {
    use super::super::super::error::VerilogError;
    use super::*;

    #[test]
    fn ut_ex1() {
        let s = r#"// This is a comment on one line.
module toplevel(clock,reset);
    input clock;
    input reset;


    `define integer HELLO = 1;

    reg flop1;
    reg flop2;
    /*
    Block   comment!! // Wooo!!
    */
    always @(posedge reset or posedge clock) begin
        if (reset) begin
            flop1 <= 1;
            flop2 <= 0;
            $display("hello world! %d", `HELLO);
        end
        else begin
            flop1 <= flop2;
            flop2 <= flop1;
        end
    end
endmodule"#;
        let tokens: Vec<Token<VerilogToken>> = VerilogTokenizer::tokenize(s)
            .into_iter()
            .map(|f| f.unwrap())
            .collect();
        println!("{:?}", tokens);
        assert_eq!(73, tokens.len());
    }

    #[test]
    fn ut_base_const_valid() {
        let valid_cases = vec![
            "'h 837FF;",
            "'o7460;",
            "16'hz;",
            "16'sd?;",
            "-4 'sd15;",
            "4 'shf;",
            "-8 'd 6;",
            "16'b0011_0101_0001_1111;",
        ];

        for s in valid_cases {
            let tokens: Vec<Token<VerilogToken>> = VerilogTokenizer::tokenize(s)
                .into_iter()
                .map(|f| f.unwrap())
                .collect();
            println!("{:?}", tokens);
            assert_eq!(tokens.len(), 3);
        }
    }

    #[test]
    fn ut_base_const_invalid() {
        let invalid_cases = vec!["4af;", "8 'd -6;"];

        for s in invalid_cases {
            let errors: Vec<TokenError<VerilogError>> = VerilogTokenizer::tokenize(s)
                .into_iter()
                .filter_map(|f| f.err())
                .collect();
            assert_eq!(errors.len(), 1);
        }
    }

    #[test]
    fn ut_numbers_valid() {
        let valid_cases = vec!["659;", "1_000_000;"];

        for s in valid_cases {
            let tokens: Vec<Token<VerilogToken>> = VerilogTokenizer::tokenize(s)
                .into_iter()
                .map(|f| f.unwrap())
                .collect();
            assert_eq!(tokens.len(), 3);
        }
    }

    #[test]
    fn ut_real_const_valid() {
        let valid_cases = vec![
            "1.2;",
            "0.1;",
            "2394.26331;",
            "1.2E12;",
            "1.30e-2;",
            "0.1e-0;",
            "23E10;",
            "29E-2;",
            "236.123_763_e-12;",
        ];

        for s in valid_cases {
            let tokens: Vec<Token<VerilogToken>> = VerilogTokenizer::tokenize(s)
                .into_iter()
                .map(|f| f.unwrap())
                .collect();
            println!("{:?}", tokens);
            assert_eq!(tokens.len(), 3);
        }
    }

    #[test]
    fn ut_real_const_invalid() {
        let invalid_cases = vec!["9.;", "4.E3;" /* ".12;", ".2e-7;" */];

        for s in invalid_cases {
            println!("{}", s);
            let errors: Vec<TokenError<VerilogError>> = VerilogTokenizer::tokenize(s)
                .into_iter()
                .filter_map(|f| f.err())
                .collect();
            assert_eq!(errors.len(), 1);
        }
    }

    #[test]
    fn ut_identifier_valid() {
        let valid_cases = vec![
            "hello",
            "\\cpu3",
            "Module",
            "v$ar_a",
            "var23_g",
            "MY_ID",
            "_unused_port",
        ];
        for s in valid_cases {
            println!("{:?}", s);
            let tokens: Vec<Token<VerilogToken>> = VerilogTokenizer::tokenize(s)
                .into_iter()
                .map(|f| f.unwrap())
                .collect();

            assert_eq!(tokens.len(), 2);
        }
    }

    #[test]
    fn ut_identifier_invalid() {
        let invalid_cases = vec!["2var"];
        for s in invalid_cases {
            println!("{:?}", s);
            let errors: Vec<TokenError<VerilogError>> = VerilogTokenizer::tokenize(s)
                .into_iter()
                .filter_map(|f| f.err())
                .collect();
            assert_eq!(errors.len(), 1);
        }
    }

    #[test]
    fn ut_string_literal_valid() {
        let valid_cases = vec![r#""hello world!");"#, r#""\"hello world!\"");"#];

        for s in valid_cases {
            println!("{:?}", s);
            let tokens: Vec<Token<VerilogToken>> = VerilogTokenizer::tokenize(s)
                .into_iter()
                .map(|f| f.unwrap())
                .collect();

            assert_eq!(tokens.len(), 4);
        }
    }

    #[test]
    fn ut_directive_valid() {
        let valid_cases = vec!["`timescale 1ns/1ps", "`MY_MACRO <= 2;"];

        for s in valid_cases {
            println!("{:?}", s);
            let tokens: Vec<Token<VerilogToken>> = VerilogTokenizer::tokenize(s)
                .into_iter()
                .map(|f| f.unwrap())
                .collect();

            assert_eq!(tokens.len(), 5);
        }
    }

    #[test]
    fn ut_directive_invalid() {
        let invalid_cases: Vec<&str> = vec!["` define integer HELLO = 2;"];
        for s in invalid_cases {
            println!("{:?}", s);
            let errors: Vec<TokenError<VerilogError>> = VerilogTokenizer::tokenize(s)
                .into_iter()
                .filter_map(|f| f.err())
                .collect();
            assert_eq!(errors.len(), 1);
        }
    }
}
