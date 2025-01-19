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

use super::super::lexer::TrainCar;
use colored::ColoredString;
use colored::Colorize;
use std::fmt::Debug;
use std::fmt::Display;
use std::str::FromStr;

pub mod comment;
pub mod delimiter;

pub mod identifier;
pub mod keyword;
pub mod literal;
pub mod tokenizer;
use super::highlight::*;

use literal::{based_integer, AbstLiteral, BaseSpec, BitStrLiteral, Character};
use tokenizer::*;

pub type Identifier = identifier::Identifier;
pub type Comment = comment::Comment;
pub type Keyword = keyword::Keyword;
pub type Delimiter = delimiter::Delimiter;
pub type VhdlTokenizer = tokenizer::VhdlTokenizer;
pub type VhdlError = super::error::VhdlError;

pub trait ToColor: Display {
    fn to_color(&self) -> ColoredString;
}

#[derive(Debug, PartialEq, Clone)]
pub enum VhdlToken {
    Comment(Comment),             // (String)
    Identifier(Identifier), // (String) ...can be general or extended (case-sensitive) identifier
    AbstLiteral(AbstLiteral), // (String)
    CharLiteral(Character), // (String)
    StrLiteral(String),     // (String)
    BitStrLiteral(BitStrLiteral), // (String)
    Keyword(Keyword),
    Delimiter(Delimiter),
    EOF,
}

impl ToColor for VhdlToken {
    fn to_color(&self) -> ColoredString {
        match &self {
            Self::Comment(c) => c.to_color(),
            Self::Identifier(i) => i.to_color(),
            Self::AbstLiteral(a) => a.to_color(),
            Self::CharLiteral(c) => c.to_color(),
            Self::StrLiteral(s) => color(&format!("\"{}\"", s.to_string()), STRINGS),
            Self::BitStrLiteral(b) => b.to_color(),
            Self::Keyword(k) => k.to_color(),
            Self::Delimiter(d) => d.to_color(),
            Self::EOF => String::new().normal(),
        }
    }
}

impl Display for VhdlToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Comment(note) => note.to_string(),
                Self::Identifier(id) => id.to_string(),
                Self::AbstLiteral(a) => a.to_string(),
                Self::CharLiteral(c) => c.to_string(),
                Self::StrLiteral(s) => format!("\"{}\"", s),
                Self::BitStrLiteral(b) => b.to_string(),
                Self::Keyword(kw) => kw.to_string(),
                Self::Delimiter(d) => d.to_string(),
                Self::EOF => String::new(),
            }
        )
    }
}

impl VhdlToken {
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
            Self::Delimiter(_) => true,
            _ => false,
        }
    }

    /// Casts as a delimiter
    pub fn as_delimiter(&self) -> Option<&Delimiter> {
        match self {
            Self::Delimiter(d) => Some(d),
            _ => None,
        }
    }

    /// Attempts to match a string `s` to a valid delimiter.
    pub fn match_delimiter(s: &str) -> Result<Self, VhdlError> {
        match Delimiter::transform(s) {
            Some(d) => Ok(VhdlToken::Delimiter(d)),
            None => Err(VhdlError::Invalid(s.to_string())),
        }
    }

    /// Captures VHDL Tokens that begin with `integer` production rule:
    /// decimal literal, based_literal, and bit_string_literals.
    ///
    /// Assumes the incoming char `c0` was last char consumed as it a digit `0..=9`.
    pub fn consume_numeric(
        train: &mut TrainCar<impl Iterator<Item = char>>,
        c0: char,
    ) -> Result<VhdlToken, VhdlError> {
        let mut based_delim: Option<char> = None;
        let mut number = Self::consume_value_pattern(train, Some(c0), char_set::is_digit)?;
        // check if the next char should be included
        if let Some(mut c) = train.peek() {
            // * decimal_literal
            if c == &char_set::DOT {
                number.push(train.consume().unwrap());
                // gather more integers (must exist)
                let fraction = Self::consume_value_pattern(train, None, char_set::is_digit)?;
                if fraction.is_empty() {
                    return Err(VhdlError::Any(String::from(
                        "cannot have trailing decimal point",
                    )));
                // append to number
                } else {
                    number.push_str(&fraction);
                }
                // update c if there is another token to grab!
                c = if let Some(c_next) = train.peek() {
                    c_next
                } else {
                    return Ok(VhdlToken::AbstLiteral(AbstLiteral::Decimal(number)));
                };
            // * based_literal (can begin with '#' or ':')
            } else if c == &char_set::HASH || c == &char_set::COLON {
                // verify 2 <= number <= 16
                let base = literal::interpret_integer(&number);
                if base < 2 || base > 16 {
                    return Err(VhdlError::Any(String::from(
                        "based literal must have base of at least 2 and at most 16",
                    )));
                }
                based_delim = Some(*c);
                number.push(train.consume().unwrap());
                // gather initial extended digits
                // select the `eval` fn to evaluate digits
                let eval = based_integer::as_fn(base);
                let base_integers = Self::consume_value_pattern(train, None, eval)?;

                number.push_str(&base_integers);
                // stil expecting another token
                if let Some(c_next) = train.consume() {
                    // closing with a '#' or ':'
                    if c_next == based_delim.unwrap() {
                        number.push(c_next);
                    // is there a dot?
                    } else if c_next == char_set::DOT {
                        number.push(c_next);
                        // gather more integers (must exist)
                        let fraction = Self::consume_value_pattern(train, None, eval)?;
                        number.push_str(&fraction);
                        // make sure there is a closing character '#' or ':'
                        if let Some(c_next_next) = train.consume() {
                            // did not find the closing character '#' or ':'
                            if c_next_next != based_delim.unwrap() {
                                return Err(VhdlError::Any(String::from(
                                    "expecting closing '#' but found something else",
                                )));
                            }
                            if fraction.is_empty() {
                                return Err(VhdlError::Any(String::from(
                                    "expecting an integer after the dot",
                                )));
                            }
                            number.push(c_next_next);
                        // there is no more characters left to consume
                        } else {
                            if fraction.is_empty() {
                                return Err(VhdlError::Any(String::from(
                                    "expecting an integer after the dot",
                                )));
                            }
                            return Err(VhdlError::Any(String::from("expecting closing '#'")));
                        }
                    // an unknown character
                    } else {
                        return Err(VhdlError::Any(String::from(
                            "expecting closing '#' but got something else",
                        )));
                    }
                    // update c if there is another token to grab!
                    c = if let Some(c_next_next) = train.peek() {
                        c_next_next
                    } else {
                        return Ok(VhdlToken::AbstLiteral(AbstLiteral::Based(number)));
                    }
                // there is no more characters to consume
                } else {
                    return Err(VhdlError::Any(String::from("expecting closing '#'")));
                }
            // * bit string literal
            } else if c != &'e' && c != &'E' && char_set::is_letter(&c) {
                // gather letters
                let mut base_spec = String::from(train.consume().unwrap());
                while let Some(c_next) = train.peek() {
                    if char_set::is_letter(c_next) == true {
                        base_spec.push(train.consume().unwrap());
                    } else {
                        break;
                    }
                }
                // verify valid base specifier
                BaseSpec::from_str(&base_spec)?;
                // force double quote to be next
                if train.peek().is_none() || train.peek().unwrap() != &char_set::DOUBLE_QUOTE {
                    return Err(VhdlError::Any(String::from(
                        "expecting opening quote character for bit string literal",
                    )));
                }
                // append base_specifier
                number.push_str(&base_spec);
                // append first double quote " char
                number.push(train.consume().unwrap());
                // complete tokenizing the bit string literal
                return Ok(Self::consume_bit_str_literal(train, number)?);
            }
            // gather exponent
            if c == &'e' || c == &'E' {
                let c0 = train.consume().unwrap();
                let expon = Self::consume_exponent(train, c0)?;
                number.push_str(&expon);
            }
            return Ok(VhdlToken::AbstLiteral(match based_delim {
                Some(_) => AbstLiteral::Based(number),
                None => AbstLiteral::Decimal(number),
            }));
        } else {
            Ok(VhdlToken::AbstLiteral(AbstLiteral::Decimal(number)))
        }
    }

    /// Captures VHDL Tokens: keywords, basic identifiers, and regular bit string literals.
    ///
    /// Assumes the first `letter` char was the last char consumed before the function call.
    pub fn consume_word(
        train: &mut TrainCar<impl Iterator<Item = char>>,
        c0: char,
    ) -> Result<VhdlToken, VhdlError> {
        let mut word = Self::consume_value_pattern(train, Some(c0), char_set::is_letter_or_digit)?;
        match Keyword::match_keyword(&word) {
            Some(kw) => Ok(VhdlToken::Keyword(kw)),
            None => {
                // * bit string literal: check if the next char is a double quote
                if let Some(c) = train.peek() {
                    if c == &char_set::DOUBLE_QUOTE {
                        // verify valid base specifier
                        BaseSpec::from_str(&word)?;
                        // add the opening '"' character to the literal
                        word.push(train.consume().unwrap());
                        return Ok(Self::consume_bit_str_literal(train, word)?);
                    }
                }
                Ok(VhdlToken::Identifier(Identifier::Basic(word)))
            }
        }
    }

    /// Captures the remaining characters for a bit string literal.
    ///
    /// Assumes the integer, base_specifier, and first " char are already consumed
    /// and moved as `s0`.  Rules taken from VHDL-2019 LRM p177 due to backward-compatible additions. Note
    /// that a bit string literal is allowed to have no characters within the " ".
    /// - bit_string_literal ::= \[ integer ] base_specifier " \[ bit_value ] "
    /// - bit_value ::= graphic_character { [ underline ] graphic_character }
    pub fn consume_bit_str_literal(
        train: &mut TrainCar<impl Iterator<Item = char>>,
        s0: String,
    ) -> Result<VhdlToken, VhdlError> {
        let mut literal = s0;
        // consume bit_value (all graphic characters except the double quote " char)
        let bit_value =
            Self::consume_value_pattern(train, None, char_set::is_graphic_and_not_double_quote)?;
        // verify the next character is the closing double quote " char
        if train.peek().is_none() || train.peek().unwrap() != &char_set::DOUBLE_QUOTE {
            return Err(VhdlError::Any(String::from(
                "expecting closing double quote for bit string literal",
            )));
        }
        literal.push_str(&bit_value);
        // accept the closing " char
        literal.push(train.consume().unwrap());
        Ok(VhdlToken::BitStrLiteral(BitStrLiteral(literal)))
    }

    /// Captures an extended identifier token.
    ///
    /// Errors if the identifier is empty.
    pub fn consume_extended_identifier(
        train: &mut TrainCar<impl Iterator<Item = char>>,
    ) -> Result<VhdlToken, VhdlError> {
        let id = Self::consume_literal(train, &char_set::BACKSLASH)?;
        if id.is_empty() {
            Err(VhdlError::Any(String::from(
                "extended identifier cannot be empty",
            )))
        } else {
            Ok(VhdlToken::Identifier(Identifier::Extended(id)))
        }
    }

    /// Captures a character literal according to VHDL-2018 LRM p231.
    ///
    /// Assumes the first single quote '\'' was the last char consumed.
    pub fn consume_char_lit(
        train: &mut TrainCar<impl Iterator<Item = char>>,
    ) -> Result<VhdlToken, VhdlError> {
        let mut char_lit = String::with_capacity(1);
        if let Some(c) = train.consume() {
            // verify the character is a graphic character
            if char_set::is_graphic(&c) == false {
                return Err(VhdlError::Any(String::from("char not graphic")));
            }
            // add to the struct
            char_lit.push(c);
            // expect a closing single-quote @TODO handle attribute case name'attribute
            if let Some(c) = train.consume() {
                // return
                if c != char_set::SINGLE_QUOTE {
                    return Err(VhdlError::Any(String::from(
                        "expecting a single quote but got something else",
                    )));
                }
            } else {
                return Err(VhdlError::Any(String::from(
                    "expecting a single quote but got none",
                )));
            }
        }
        Ok(VhdlToken::CharLiteral(Character(char_lit)))
    }

    /// Captures a string literal.
    ///
    /// Assumes the first double quote '\"' was the last char consumed before entering the function.
    pub fn consume_str_lit(
        train: &mut TrainCar<impl Iterator<Item = char>>,
    ) -> Result<VhdlToken, VhdlError> {
        let value = Self::consume_literal(train, &char_set::DOUBLE_QUOTE)?;
        Ok(VhdlToken::StrLiteral(value))
    }

    /// Collects a delimited comment (all characters after a `/*` up until `*/`).
    ///
    /// Assumes the opening '/' char was the last char consumed before entering the function.
    /// Also assumes the next char is '*'.
    pub fn consume_delim_comment(
        train: &mut TrainCar<impl Iterator<Item = char>>,
    ) -> Result<VhdlToken, VhdlError> {
        // skip over opening '*'
        train.consume().expect("assumes '*' exists");
        let mut note = String::new();
        while let Some(c) = train.consume() {
            // check if we are breaking from the comment
            if c == char_set::STAR {
                if let Some(c_next) = train.peek() {
                    // break from the comment
                    if c_next == &char_set::FWDSLASH {
                        train.consume();
                        return Ok(VhdlToken::Comment(Comment::Delimited(note)));
                    }
                }
            }
            note.push(c);
        }
        Err(VhdlError::Any(String::from("missing closing delimiter */")))
    }

    /// Collects a single-line comment (all characters after a `--` up until end-of-line).
    ///
    /// Assumes the opening '-' was the last char consumed before entering the function.
    /// Also assumes the next char is '-'.
    pub fn consume_comment(
        train: &mut TrainCar<impl Iterator<Item = char>>,
    ) -> Result<VhdlToken, VhdlError> {
        // skip over second '-'
        train.consume();
        // consume characters to form the comment
        let mut note = String::new();
        while let Some(c) = train.consume() {
            // cannot be vt, cr (\r), lf (\n)
            if c == '\u{000B}' || c == '\u{000D}' || c == '\u{000A}' {
                break;
            } else {
                note.push(c);
            }
        }
        Ok(VhdlToken::Comment(Comment::Single(note)))
    }

    /// Walks through the possible interpretations for capturing a VHDL delimiter.
    ///
    /// If it successfully finds a valid VHDL delimiter, it will move the `loc` the number
    /// of characters it consumed.
    pub fn collect_delimiter(
        train: &mut TrainCar<impl Iterator<Item = char>>,
        c0: Option<char>,
    ) -> Result<VhdlToken, VhdlError> {
        // delimiter will have at most 3 characters
        let mut delim = String::with_capacity(3);
        if let Some(c) = c0 {
            delim.push(c);
        };
        // check the next character in the sequence
        while let Some(c) = train.peek() {
            match delim.len() {
                0 => match c {
                    // ambiguous characters...read another character (could be a len-2 delimiter)
                    '?' | '<' | '>' | '/' | '=' | '*' | ':' => delim.push(train.consume().unwrap()),
                    // if it was a delimiter, take the character and increment the location
                    _ => return Self::match_delimiter(&String::from(train.consume().unwrap())),
                },
                1 => match delim.chars().nth(0).unwrap() {
                    '?' => match c {
                        // move on to next round (could be a len-3 delimiter)
                        '/' | '<' | '>' => delim.push(train.consume().unwrap()),
                        _ => return Ok(Self::match_delimiter(&delim).expect("invalid token")),
                    },
                    '<' => match c {
                        // move on to next round (could be a len-3 delimiter)
                        '=' => delim.push(train.consume().unwrap()),
                        _ => return Ok(Self::match_delimiter(&delim).expect("invalid token")),
                    },
                    _ => {
                        // try with 2
                        delim.push(*c);
                        if let Ok(op) = Self::match_delimiter(&delim) {
                            train.consume();
                            return Ok(op);
                        } else {
                            // revert back to 1
                            delim.pop();
                            return Self::match_delimiter(&delim);
                        }
                    }
                },
                2 => {
                    // try with 3
                    delim.push(*c);
                    if let Ok(op) = Self::match_delimiter(&delim) {
                        train.consume();
                        return Ok(op);
                    } else {
                        // revert back to 2 (guaranteed to exist)
                        delim.pop();
                        return Ok(Self::match_delimiter(&delim).expect("invalid token"));
                    }
                }
                _ => panic!("delimiter matching exceeds 3 characters"),
            }
        }
        // try when hiting end of stream
        Self::match_delimiter(&delim)
    }

    /// Captures the generic pattern production rule by passing a fn as `eval` to compare.
    ///
    /// This function allows for an empty result to be returned as `Ok`.
    /// - A ::= A { \[ underline ] A }
    fn consume_value_pattern(
        train: &mut TrainCar<impl Iterator<Item = char>>,
        c0: Option<char>,
        eval: fn(&char) -> bool,
    ) -> Result<String, VhdlError> {
        let mut car = if let Some(c) = c0 {
            String::from(c)
        } else {
            String::new()
        };
        while let Some(c) = train.peek() {
            if eval(&c) == true {
                car.push(train.consume().unwrap());
            } else if c == &char_set::UNDERLINE {
                if car.is_empty() == true {
                    return Err(VhdlError::Any(String::from(
                        "expecting a digit before underline",
                    )));
                }
                car.push(train.consume().unwrap());
                // a digit must proceed the underline
                if let Some(c_next) = train.consume() {
                    if eval(&c_next) == false {
                        return Err(VhdlError::Any(String::from(
                            "expecting a digit to follow underline",
                        )));
                    } else {
                        car.push(c_next);
                    }
                } else {
                    return Err(VhdlError::Any(String::from("expecting a digit")));
                }
            } else {
                break;
            }
        }
        Ok(car)
    }

    /// Captures an exponent.   
    ///
    /// Assumes the previous function has already peeked and determined the next char is 'E' as `c0`.
    /// - exponent ::= E \[ + ] integer | E – integer  
    fn consume_exponent(
        train: &mut TrainCar<impl Iterator<Item = char>>,
        c0: char,
    ) -> Result<String, VhdlError> {
        // start with 'E'
        let mut expon = String::from(c0);
        // check for sign
        let sign = if let Some(c1) = train.consume() {
            if c1 != char_set::PLUS && c1 != char_set::DASH && char_set::is_digit(&c1) == false {
                return Err(VhdlError::Any(String::from("expecting +, -, or a digit")));
            } else {
                c1
            }
        } else {
            return Err(VhdlError::Any(String::from(
                "expecting +, -, or digit but got nothing",
            )));
        };
        // determine if c0 was a digit
        let c0 = if char_set::is_digit(&sign) == true {
            Some(sign)
        } else {
            // add the sign to the exponent
            expon.push(sign);
            None
        };
        let value = Self::consume_value_pattern(train, c0, char_set::is_digit)?;
        if value.is_empty() {
            Err(VhdlError::Any(String::from(
                "expecting an integer exponent value but got nothing",
            )))
        } else {
            expon.push_str(&value);
            Ok(expon)
        }
    }

    /// Walks through the stream to gather a `String` literal until finding the
    /// exiting character `br`.
    ///
    /// An escape is allowed by double placing the `br`, i.e. """hello"" world".
    /// Assumes the first token to parse in the stream is not the `br` character.
    /// Allows for zero or more characters in result and chars must be graphic.
    fn consume_literal(
        train: &mut TrainCar<impl Iterator<Item = char>>,
        br: &char,
    ) -> Result<String, VhdlError> {
        let mut result = String::new();
        while let Some(c) = train.consume() {
            // verify it is a graphic character
            if char_set::is_graphic(&c) == false {
                return Err(VhdlError::Any(String::from("invalid character in literal")));
            }
            // detect escape sequence
            if br == &c {
                match train.peek() {
                    Some(c_next) => {
                        if br == c_next {
                            train.consume(); // skip over escape character
                        } else {
                            return Ok(result);
                        }
                    }
                    None => return Ok(result),
                }
            }
            result.push(c);
        }
        Err(VhdlError::Any(String::from("expecting closing delimiter")))
    }
}

impl VhdlToken {
    /// Checks if the element is a particular keyword `kw`.
    pub fn check_keyword(&self, kw: &Keyword) -> bool {
        match self {
            VhdlToken::Keyword(r) => r == kw,
            _ => false,
        }
    }

    pub fn is_eof(&self) -> bool {
        match self {
            VhdlToken::EOF => true,
            _ => false,
        }
    }

    /// Accesses the underlying `Identifier`, if one exists.
    pub fn as_identifier(&self) -> Option<&Identifier> {
        match self {
            VhdlToken::Identifier(id) => Some(id),
            _ => None,
        }
    }

    /// Checks if the element is a particular delimiter `d`.
    pub fn check_delimiter(&self, d: &Delimiter) -> bool {
        match self {
            VhdlToken::Delimiter(r) => r == d,
            _ => false,
        }
    }

    pub fn as_comment(&self) -> Option<&Comment> {
        match self {
            VhdlToken::Comment(r) => Some(r),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::super::lexer::*;
    use super::*;

    #[test]
    fn iden_from_str() {
        let iden = "top_level";
        assert_eq!(
            Identifier::from_str(&iden).unwrap(),
            Identifier::Basic("top_level".to_owned())
        );

        let iden = "\\Top_LEVEL\\";
        assert_eq!(
            Identifier::from_str(&iden).unwrap(),
            Identifier::Extended("Top_LEVEL".to_owned())
        );

        // extra characters after closing
        let iden = "\\Top_\\LEVEL\\";
        assert_eq!(Identifier::from_str(&iden).is_err(), true);
    }

    #[test]
    fn interpret_int() {
        let contents = "16";
        assert_eq!(literal::interpret_integer(&contents), 16);

        let contents = "1_6";
        assert_eq!(literal::interpret_integer(&contents), 16);

        let contents = "50_000_000";
        assert_eq!(literal::interpret_integer(&contents), 50_000_000);
    }

    #[test]
    #[should_panic]
    fn interpret_int_with_other_chars() {
        let contents = "16a";
        literal::interpret_integer(&contents);
    }

    #[test]
    #[should_panic]
    fn interpret_int_with_no_leading_digit() {
        let contents = "";
        literal::interpret_integer(&contents);
    }

    #[test]
    fn single_quote_as_delimiter() {
        let contents = "\
foo <= std_logic_vector'('a','b','c');";
        let tokens: Vec<VhdlToken> = VhdlTokenizer::tokenize(&contents)
            .into_iter()
            .map(|f| f.unwrap().take())
            .collect();
        assert_eq!(
            tokens,
            vec![
                VhdlToken::Identifier(Identifier::Basic("foo".to_owned())),
                VhdlToken::Delimiter(Delimiter::SigAssign),
                VhdlToken::Identifier(Identifier::Basic("std_logic_vector".to_owned())),
                VhdlToken::Delimiter(Delimiter::SingleQuote),
                VhdlToken::Delimiter(Delimiter::ParenL),
                VhdlToken::CharLiteral(Character("a".to_owned())),
                VhdlToken::Delimiter(Delimiter::Comma),
                VhdlToken::CharLiteral(Character("b".to_owned())),
                VhdlToken::Delimiter(Delimiter::Comma),
                VhdlToken::CharLiteral(Character("c".to_owned())),
                VhdlToken::Delimiter(Delimiter::ParenR),
                VhdlToken::Delimiter(Delimiter::Terminator),
                VhdlToken::EOF,
            ]
        );

        let contents = "\
(clk'event = '1')";
        let tokens: Vec<VhdlToken> = VhdlTokenizer::tokenize(&contents)
            .into_iter()
            .map(|f| f.unwrap().take())
            .collect();
        assert_eq!(
            tokens,
            vec![
                VhdlToken::Delimiter(Delimiter::ParenL),
                VhdlToken::Identifier(Identifier::Basic("clk".to_owned())),
                VhdlToken::Delimiter(Delimiter::SingleQuote),
                VhdlToken::Identifier(Identifier::Basic("event".to_owned())),
                VhdlToken::Delimiter(Delimiter::Eq),
                VhdlToken::CharLiteral(Character("1".to_owned())),
                VhdlToken::Delimiter(Delimiter::ParenR),
                VhdlToken::EOF,
            ]
        );
    }

    #[test]
    fn lex_partial_bit_str() {
        let words = "b\"1010\"more text";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_word(&mut tc, c0),
            Ok(VhdlToken::BitStrLiteral(BitStrLiteral(
                "b\"1010\"".to_owned()
            )))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), "more text");
        assert_eq!(tc.locate(), &Position::place(1, 7));

        // invalid base specifier in any language standard
        let words = "z\"1010\"more text";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_word(&mut tc, c0).is_err(), true);
    }

    #[test]
    fn lex_full_bit_str() {
        let contents = "10b\"10_1001_1111\";";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap(); // already determined first digit
        assert_eq!(
            VhdlToken::consume_numeric(&mut tc, c0).unwrap(),
            VhdlToken::BitStrLiteral(BitStrLiteral("10b\"10_1001_1111\"".to_owned()))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), ";");
        assert_eq!(tc.locate(), &Position::place(1, 17));

        let contents = "12SX\"F-\";";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap(); // already determined first digit
        assert_eq!(
            VhdlToken::consume_numeric(&mut tc, c0).unwrap(),
            VhdlToken::BitStrLiteral(BitStrLiteral("12SX\"F-\"".to_owned()))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), ";");
        assert_eq!(tc.locate(), &Position::place(1, 8));
    }

    #[test]
    fn lex_numeric() {
        let contents = "32)";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap(); // already determined first digit
        assert_eq!(
            VhdlToken::consume_numeric(&mut tc, c0).unwrap(),
            VhdlToken::AbstLiteral(AbstLiteral::Decimal("32".to_owned()))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), ")");

        let contents = "32_000;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_numeric(&mut tc, c0).unwrap(),
            VhdlToken::AbstLiteral(AbstLiteral::Decimal("32_000".to_owned()))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), ";");

        let contents = "0.456";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_numeric(&mut tc, c0).unwrap(),
            VhdlToken::AbstLiteral(AbstLiteral::Decimal("0.456".to_owned()))
        );

        let contents = "6.023E+24";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_numeric(&mut tc, c0).unwrap(),
            VhdlToken::AbstLiteral(AbstLiteral::Decimal("6.023E+24".to_owned()))
        );

        let contents = "7#6.023#E+24";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_numeric(&mut tc, c0).unwrap(),
            VhdlToken::AbstLiteral(AbstLiteral::Based("7#6.023#E+24".to_owned()))
        );

        let contents = "16#F.FF#E+2";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_numeric(&mut tc, c0).unwrap(),
            VhdlToken::AbstLiteral(AbstLiteral::Based("16#F.FF#E+2".to_owned()))
        );

        let contents = "2#1.1111_1111_111#E11";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_numeric(&mut tc, c0).unwrap(),
            VhdlToken::AbstLiteral(AbstLiteral::Based("2#1.1111_1111_111#E11".to_owned()))
        );

        let contents = "016#0FF#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_numeric(&mut tc, c0).unwrap(),
            VhdlToken::AbstLiteral(AbstLiteral::Based("016#0FF#".to_owned()))
        );

        let contents = "1_6#1E.1f1# -- comment";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_numeric(&mut tc, c0).unwrap(),
            VhdlToken::AbstLiteral(AbstLiteral::Based("1_6#1E.1f1#".to_owned()))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), " -- comment");

        // '#' can be replaced by ':' if done in both occurences - VHDL-1993 LRM p180
        let contents = "016:0FF:";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_numeric(&mut tc, c0).unwrap(),
            VhdlToken::AbstLiteral(AbstLiteral::Based("016:0FF:".to_owned()))
        );

        let contents = "016:0FF#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_numeric(&mut tc, c0).is_err(), true);
    }

    #[test]
    fn based_literal_base_out_of_range() {
        let contents = "1#0123456789AaBbCcDdEeFfGg#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_numeric(&mut tc, c0).is_err(), true);
        assert_eq!(
            tc.peekable().clone().collect::<String>(),
            "#0123456789AaBbCcDdEeFfGg#"
        );

        let contents = "17#0123456789AaBbCcDdEeFfGg#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_numeric(&mut tc, c0).is_err(), true);
        assert_eq!(
            tc.peekable().clone().collect::<String>(),
            "#0123456789AaBbCcDdEeFfGg#"
        );
    }

    #[test]
    fn based_literal_digit_out_of_range() {
        let contents = "2#0123456789AaBbCcDdEeFfGg#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_numeric(&mut tc, c0).is_err(), true);
        assert_eq!(
            tc.peekable().clone().collect::<String>(),
            "3456789AaBbCcDdEeFfGg#"
        );

        let contents = "9#0123456789AaBbCcDdEeFfGg#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_numeric(&mut tc, c0).is_err(), true);
        assert_eq!(tc.peekable().clone().collect::<String>(), "AaBbCcDdEeFfGg#");

        let contents = "1_0#0123456789AaBbCcDdEeFfGg#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_numeric(&mut tc, c0).is_err(), true);
        assert_eq!(tc.peekable().clone().collect::<String>(), "aBbCcDdEeFfGg#");

        let contents = "11#0123456789AaBbCcDdEeFfGg#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_numeric(&mut tc, c0).is_err(), true);
        assert_eq!(tc.peekable().clone().collect::<String>(), "bCcDdEeFfGg#");

        let contents = "16#0123456789AaBbCcDdEeFfGg#";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_numeric(&mut tc, c0).is_err(), true);
        assert_eq!(tc.peekable().clone().collect::<String>(), "g#");
    }

    #[test]
    fn lex_single_comment() {
        let contents = "\
--here is a vhdl comment";
        let mut tc = TrainCar::new(contents.chars());
        tc.consume(); // already determined first dash
        assert_eq!(
            VhdlToken::consume_comment(&mut tc).unwrap(),
            VhdlToken::Comment(Comment::Single("here is a vhdl comment".to_owned()))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 24));

        let contents = "\
--here is a vhdl comment
entity fa is end entity;";
        let mut tc = TrainCar::new(contents.chars());
        tc.consume(); // already determined first dash
        assert_eq!(
            VhdlToken::consume_comment(&mut tc).unwrap(),
            VhdlToken::Comment(Comment::Single("here is a vhdl comment".to_owned()))
        );
        assert_eq!(
            tc.peekable().clone().collect::<String>(),
            "entity fa is end entity;"
        );
        assert_eq!(tc.locate(), &Position::place(2, 0));
    }

    #[test]
    fn lex_delim_comment() {
        let contents = "\
/* here is a vhdl 
delimited-line comment. Look at all the space! */;";
        let mut tc = TrainCar::new(contents.chars());
        tc.consume();
        assert_eq!(
            VhdlToken::consume_delim_comment(&mut tc).unwrap(),
            VhdlToken::Comment(Comment::Delimited(
                " here is a vhdl 
delimited-line comment. Look at all the space! "
                    .to_owned()
            ))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), ";");
        assert_eq!(tc.locate(), &Position::place(2, 49));

        let contents = "/* here is a vhdl comment";
        let mut tc = TrainCar::new(contents.chars());
        tc.consume();
        assert_eq!(VhdlToken::consume_delim_comment(&mut tc).is_err(), true);
    }

    #[test]
    fn lex_char_literal() {
        let contents = "1'";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::consume_char_lit(&mut tc).unwrap(),
            VhdlToken::CharLiteral(Character("1".to_owned()))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 2));

        let contents = "12'";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VhdlToken::consume_char_lit(&mut tc).is_err(), true);
    }

    #[test]
    fn lex_expon() {
        let contents = "E+24";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_exponent(&mut tc, c0).unwrap(), "E+24");
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 4));

        let contents = "e6;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_exponent(&mut tc, c0).unwrap(), "e6");
        assert_eq!(tc.peekable().clone().collect::<String>(), ";");
        assert_eq!(tc.locate(), &Position::place(1, 2));

        let contents = "e-12;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_exponent(&mut tc, c0).unwrap(), "e-12");
        assert_eq!(tc.peekable().clone().collect::<String>(), ";");

        // negative test cases
        let contents = "e-;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_exponent(&mut tc, c0).is_err(), true);

        let contents = "e+2_;";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_exponent(&mut tc, c0).is_err(), true);

        let contents = "e";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_exponent(&mut tc, c0).is_err(), true);
    }

    #[test]
    fn lex_integer() {
        // allow bit string literal to be none
        let contents = "";
        // testing using digit prod. rule "graphic"
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::consume_value_pattern(&mut tc, None, char_set::is_graphic).unwrap(),
            ""
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 0));

        let contents = "234";
        // testing using digit prod. rule "integer"
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::consume_value_pattern(&mut tc, None, char_set::is_digit).unwrap(),
            "234"
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 3));

        let contents = "1_2_345 ";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::consume_value_pattern(&mut tc, None, char_set::is_digit).unwrap(),
            "1_2_345"
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), " ");
        assert_eq!(tc.locate(), &Position::place(1, 7));

        let contents = "23__4";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::consume_value_pattern(&mut tc, None, char_set::is_digit).is_err(),
            true
        ); // double underscore

        let contents = "_24";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::consume_value_pattern(&mut tc, None, char_set::is_digit).is_err(),
            true
        ); // leading underscore

        let contents = "_23_4";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::consume_value_pattern(&mut tc, Some('1'), char_set::is_digit).is_ok(),
            true
        );

        // testing using extended_digit prod. rule "based_integer"
        let contents = "abcd_FFFF_0021";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::consume_value_pattern(&mut tc, None, char_set::is_extended_digit).unwrap(),
            "abcd_FFFF_0021"
        );

        // testing using graphic_char prod. rule "bit_value"
        let contents = "XXXX_01LH_F--1";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::consume_value_pattern(&mut tc, None, char_set::is_graphic).unwrap(),
            "XXXX_01LH_F--1"
        );
    }

    #[test]
    fn lex_identifier() {
        let words = "entity is";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_word(&mut tc, c0).unwrap(),
            VhdlToken::Keyword(Keyword::Entity)
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), " is");
        assert_eq!(tc.locate(), &Position::place(1, 6));

        let words = "std_logic_1164.all;";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_word(&mut tc, c0).unwrap(),
            VhdlToken::Identifier(Identifier::Basic("std_logic_1164".to_owned()))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), ".all;");
        assert_eq!(tc.locate(), &Position::place(1, 14));

        let words = "ready_OUT<=";
        let mut tc = TrainCar::new(words.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_word(&mut tc, c0).unwrap(),
            VhdlToken::Identifier(Identifier::Basic("ready_OUT".to_owned()))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), "<=");
        assert_eq!(tc.locate(), &Position::place(1, 9));
    }

    #[test]
    fn lex_literal() {
        let contents = "\" go Gators! \" ";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_literal(&mut tc, &c0).unwrap(),
            " go Gators! "
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), " ");
        assert_eq!(tc.locate(), &Position::place(1, 14));

        let contents = "\" go \"\"to\"\"\" ";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_literal(&mut tc, &c0).unwrap(),
            " go \"to\""
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), " ");
        assert_eq!(tc.locate(), &Position::place(1, 12));

        let contents = "\"go ";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_literal(&mut tc, &c0).is_err(), true); // no closing quote
    }

    #[test]
    fn lex_literal_2() {
        let contents = "\"Setup time is too short\"more text";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_literal(&mut tc, &c0).unwrap(),
            "Setup time is too short"
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), "more text");
        assert_eq!(tc.locate(), &Position::place(1, 25));

        let contents = "\"\"\"\"\"\"";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_literal(&mut tc, &c0).unwrap(), "\"\"");
        assert_eq!(tc.locate(), &Position::place(1, 6));

        let contents = "\" go \"\"gators\"\" from UF! \"";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(
            VhdlToken::consume_literal(&mut tc, &c0).unwrap(),
            " go \"gators\" from UF! "
        );
        assert_eq!(tc.locate(), &Position::place(1, 26));

        let contents = "\\VHDL\\";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_literal(&mut tc, &c0).unwrap(), "VHDL");

        let contents = "\\a\\\\b\\more text afterward";
        let mut tc = TrainCar::new(contents.chars());
        let c0 = tc.consume().unwrap();
        assert_eq!(VhdlToken::consume_literal(&mut tc, &c0).unwrap(), "a\\b");
        // verify the stream is left in the correct state
        assert_eq!(
            tc.peekable().clone().collect::<String>(),
            "more text afterward"
        );
    }

    #[test]
    fn lex_tokens() {
        let s = "\
entity fa is end entity;";
        let tokens: Vec<VhdlToken> = VhdlTokenizer::tokenize(s)
            .into_iter()
            .map(|f| f.unwrap().take())
            .collect();
        assert_eq!(
            tokens,
            vec![
                VhdlToken::Keyword(Keyword::Entity),
                VhdlToken::Identifier(Identifier::Basic("fa".to_owned())),
                VhdlToken::Keyword(Keyword::Is),
                VhdlToken::Keyword(Keyword::End),
                VhdlToken::Keyword(Keyword::Entity),
                VhdlToken::Delimiter(Delimiter::Terminator),
                VhdlToken::EOF,
            ]
        );
    }

    #[test]
    fn lex_comment_token() {
        let s = "\
-- here is a vhdl single-line comment!";
        let tokens: Vec<Token<VhdlToken>> = VhdlTokenizer::tokenize(s)
            .into_iter()
            .map(|f| f.unwrap())
            .collect();
        assert_eq!(
            tokens,
            vec![
                Token::new(
                    VhdlToken::Comment(Comment::Single(
                        " here is a vhdl single-line comment!".to_owned()
                    )),
                    Position::place(1, 1)
                ),
                Token::new(VhdlToken::EOF, Position::place(1, 39)),
            ]
        );
    }

    #[test]
    fn lex_comment_token_delim() {
        let s = "\
/* here is a vhdl 
    delimited-line comment. Look at all the space! */";
        let tokens: Vec<Token<VhdlToken>> = VhdlTokenizer::tokenize(s)
            .into_iter()
            .map(|f| f.unwrap())
            .collect();
        assert_eq!(
            tokens,
            vec![
                Token::new(
                    VhdlToken::Comment(Comment::Delimited(
                        " here is a vhdl 
    delimited-line comment. Look at all the space! "
                            .to_owned()
                    )),
                    Position::place(1, 1)
                ),
                Token::new(VhdlToken::EOF, Position::place(2, 54)),
            ]
        );
    }

    #[test]
    fn lex_vhdl_line() {
        let s = "\
signal magic_num : std_logic := '1';";
        let tokens: Vec<Token<VhdlToken>> = VhdlTokenizer::tokenize(s)
            .into_iter()
            .map(|f| f.unwrap())
            .collect();
        assert_eq!(
            tokens,
            vec![
                Token::new(VhdlToken::Keyword(Keyword::Signal), Position::place(1, 1)),
                Token::new(
                    VhdlToken::Identifier(Identifier::Basic("magic_num".to_owned())),
                    Position::place(1, 8)
                ),
                Token::new(
                    VhdlToken::Delimiter(Delimiter::Colon),
                    Position::place(1, 18)
                ),
                Token::new(
                    VhdlToken::Identifier(Identifier::Basic("std_logic".to_owned())),
                    Position::place(1, 20)
                ),
                Token::new(
                    VhdlToken::Delimiter(Delimiter::VarAssign),
                    Position::place(1, 30)
                ),
                Token::new(
                    VhdlToken::CharLiteral(Character("1".to_owned())),
                    Position::place(1, 33)
                ),
                Token::new(
                    VhdlToken::Delimiter(Delimiter::Terminator),
                    Position::place(1, 36)
                ),
                Token::new(VhdlToken::EOF, Position::place(1, 37)),
            ]
        );
    }

    #[test]
    fn locate_tokens() {
        let s = "\
entity fa is end entity;";
        let tokens: Vec<Position> = VhdlTokenizer::tokenize(s)
            .into_iter()
            .map(|f| f.unwrap().locate().clone())
            .collect();
        assert_eq!(
            tokens,
            vec![
                Position::place(1, 1),  // 1:1 keyword: entity
                Position::place(1, 8),  // 1:8 basic identifier: fa
                Position::place(1, 11), // 1:11 keyword: is
                Position::place(1, 14), // 1:14 keyword: end
                Position::place(1, 18), // 1:18 keyword: entity
                Position::place(1, 24), // 1:24 delimiter: ;
                Position::place(1, 25), // 1:25 eof
            ]
        );
    }

    #[test]
    fn lex_delimiter_single() {
        let contents = "&";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::collect_delimiter(&mut tc, None),
            Ok(VhdlToken::Delimiter(Delimiter::Ampersand))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 1));

        let contents = "?";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::collect_delimiter(&mut tc, None),
            Ok(VhdlToken::Delimiter(Delimiter::Question))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 1));

        let contents = "< MAX_COUNT";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::collect_delimiter(&mut tc, None),
            Ok(VhdlToken::Delimiter(Delimiter::Lt))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), " MAX_COUNT");
        assert_eq!(tc.locate(), &Position::place(1, 1));

        let contents = ");";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::collect_delimiter(&mut tc, None),
            Ok(VhdlToken::Delimiter(Delimiter::ParenR))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), ";");
        assert_eq!(tc.locate(), &Position::place(1, 1));
    }

    #[test]
    fn lex_delimiter_none() {
        let contents = "fa";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(VhdlToken::collect_delimiter(&mut tc, None).is_err(), true);
        assert_eq!(tc.peekable().clone().collect::<String>(), "a");
        assert_eq!(tc.locate(), &Position::place(1, 1));
    }

    #[test]
    fn lex_delimiter_double() {
        let contents = "<=";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::collect_delimiter(&mut tc, None),
            Ok(VhdlToken::Delimiter(Delimiter::SigAssign))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 2));

        let contents = "**WIDTH";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::collect_delimiter(&mut tc, None),
            Ok(VhdlToken::Delimiter(Delimiter::DoubleStar))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), "WIDTH");
        assert_eq!(tc.locate(), &Position::place(1, 2));
    }

    #[test]
    fn lex_delimiter_triple() {
        let contents = "<=>";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::collect_delimiter(&mut tc, None),
            Ok(VhdlToken::Delimiter(Delimiter::SigAssoc))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), "");
        assert_eq!(tc.locate(), &Position::place(1, 3));

        let contents = "?/= MAGIC_NUM";
        let mut tc = TrainCar::new(contents.chars());
        assert_eq!(
            VhdlToken::collect_delimiter(&mut tc, None),
            Ok(VhdlToken::Delimiter(Delimiter::MatchNE))
        );
        assert_eq!(tc.peekable().clone().collect::<String>(), " MAGIC_NUM");
        assert_eq!(tc.locate(), &Position::place(1, 3));
    }

    #[test]
    fn match_delimiter() {
        let word = "<=";
        assert_eq!(
            VhdlToken::match_delimiter(word),
            Ok(VhdlToken::Delimiter(Delimiter::SigAssign))
        );

        let word = "-";
        assert_eq!(
            VhdlToken::match_delimiter(word),
            Ok(VhdlToken::Delimiter(Delimiter::Dash))
        );

        let word = "<=>";
        assert_eq!(
            VhdlToken::match_delimiter(word),
            Ok(VhdlToken::Delimiter(Delimiter::SigAssoc))
        );

        let word = "^";
        assert_eq!(VhdlToken::match_delimiter(word).is_err(), true);

        let word = "entity";
        assert_eq!(VhdlToken::match_delimiter(word).is_err(), true);
    }

    #[test]
    fn match_reserved_idenifier() {
        let word = "END";
        assert_eq!(Keyword::match_keyword(word), Some(Keyword::End));

        let word = "EnTITY";
        assert_eq!(Keyword::match_keyword(word), Some(Keyword::Entity));

        let word = "entitys";
        assert_eq!(Keyword::match_keyword(word), None);

        let word = "<=";
        assert_eq!(Keyword::match_keyword(word), None);
    }

    #[test]
    fn is_separator() {
        let c = ' '; // space
        assert_eq!(char_set::is_separator(&c), true);

        let c = '\u{00A0}'; // nbsp
        assert_eq!(char_set::is_separator(&c), true);

        let c = '\t'; // horizontal tab
        assert_eq!(char_set::is_separator(&c), true);

        let c = '\n'; // new-line
        assert_eq!(char_set::is_separator(&c), true);

        let c = 'c'; // negative case: ascii char
        assert_eq!(char_set::is_separator(&c), false);
    }

    #[test]
    fn identifier_equality_and_len() {
        let id0 = Identifier::Basic("fa".to_owned());
        let id1 = Identifier::Basic("Fa".to_owned());
        assert_eq!(id1.len(), 2);
        assert_eq!(id0, id1);

        let id0 = Identifier::Basic("fa".to_owned());
        let id1 = Identifier::Basic("Full_adder".to_owned());
        assert_ne!(id0, id1);

        let id0 = Identifier::Basic("VHDL".to_owned()); // written as: VHDL
        let id1 = Identifier::Extended("VHDL".to_owned()); // written as: \VHDL\
        assert_ne!(id0, id1);

        let id0 = Identifier::Extended("vhdl".to_owned()); // written as: \vhdl\
        let id1 = Identifier::Extended("VHDL".to_owned()); // written as: \VHDL\
        assert_ne!(id0, id1);
        assert_eq!(id1.len(), 6);

        let id0 = Identifier::Extended("I\\D".to_owned()); // written as: \I\\D\
        assert_eq!(id0.len(), 6);

        let id0 = Identifier::from_str("\\I\\\\DEN\\").unwrap(); // written as: \I\\D\
        assert_eq!(id0.len(), 8);
    }

    #[test]
    fn comment_ending_pos() {
        let comment = Comment::Delimited("gators".to_string());
        assert_eq!(comment.ending_position(), Position::place(1, 10));
        let comment = Comment::Single("gators".to_string());
        assert_eq!(comment.ending_position(), Position::place(1, 8));

        let comment = Comment::Delimited("gators\n".to_string());
        assert_eq!(comment.ending_position(), Position::place(2, 2));
    }

    #[test]
    #[ignore]
    fn playground_code() {
        let s = "\
-- design file for a nor_gate
library ieee;
use ieee.std_logic_1164.all;

entity \\nor_gate\\ is --$ -- error on this line
    generic(
        N: positive
    );
    port(
        a : in std_logic_vector(N-1 downto 0);
        \\In\\ : in std_logic_vector(N-1 downto 0);
        c : out std_logic_vector(N-1 downto 0)
    );
end entity nor_gate;

architecture rtl of nor_gate is
    constant GO_ADDR_MMAP:integer:=2#001_1100.001#E14;
    constant freq_hz : unsigned := 50_000_000;
    signal   MAGIC_NUM_3 : bit_vector(3 downto 0) := 0sx\"\"
    constant MAGIC_NUM_1 : integer := 2#10101#; -- test constants against tokenizer
    constant MAGIC_NUM_2 : std_logic_vector(7 downto 0) := 0 -- 8c\"11\";
begin
    c <= a nor \\In\\;

end architecture rtl; /* long comment */";
        let vhdl = VhdlTokenizer::from_source_code(&s);
        println!("{:?}", vhdl);
        panic!("manually inspect token list")
    }
}
