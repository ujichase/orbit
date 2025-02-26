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

use crate::core::lang::highlight::ToColor;
use colored::ColoredString;
use colored::Colorize;

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    ConcatL,
    ConcatR,
    ReplicateL,
    ReplicateR,
    Plus,
    Minus,
    Mult,
    Div,
    Pow,
    Modulus,
    Lt,
    Gt,
    Lte,
    Gte,
    LogicNeg,
    LogicAnd,
    LogicOr,
    LogicEq,
    LogicIneq,
    CaseEq,
    CaseIneq,
    BitNeg,
    BitReductAnd,
    BitReductOr,
    BitReductXor,
    BitEquivReductXnor1,
    BitEquivReductXnor2,
    ReductNand,
    ReductNor,
    LogicShiftL,
    LogicShiftR,
    ArithShiftL,
    ArithShiftR,
    Question,
    Colon,
    // not operators per say, but they are delimiters
    Comma,
    Terminator,
    ParenL,
    ParenR,
    Dot,
    BrackL,
    BrackR,
    Pound,
    BlockAssign,
    At,
    AttrL,
    AttrR,
    SingleQuote,
    DotStar,
}

impl Operator {
    /// Attempts to match the given string of characters `s` to a Verilog operator.
    pub fn transform(s: &str) -> Option<Self> {
        Some(match s {
            "{" => Self::ConcatL,
            "}" => Self::ConcatR,
            "{{" => Self::ReplicateL,
            "}}" => Self::ReplicateR,
            "+" => Self::Plus,
            "-" => Self::Minus,
            "*" => Self::Mult,
            "/" => Self::Div,
            "**" => Self::Pow,
            "%" => Self::Modulus,
            "<" => Self::Lt,
            ">" => Self::Gt,
            "<=" => Self::Lte,
            ">=" => Self::Gte,
            "!" => Self::LogicNeg,
            "&&" => Self::LogicAnd,
            "||" => Self::LogicOr,
            "==" => Self::LogicEq,
            "!=" => Self::LogicIneq,
            "===" => Self::CaseEq,
            "!==" => Self::CaseIneq,
            "~" => Self::BitNeg,
            "&" => Self::BitReductAnd,
            "|" => Self::BitReductOr,
            "^" => Self::BitReductXor,
            "^~" => Self::BitEquivReductXnor1,
            "~^" => Self::BitEquivReductXnor2,
            "~&" => Self::ReductNand,
            "~|" => Self::ReductNor,
            "<<" => Self::LogicShiftL,
            ">>" => Self::LogicShiftR,
            "<<<" => Self::ArithShiftL,
            ">>>" => Self::ArithShiftR,
            "?" => Self::Question,
            ":" => Self::Colon,
            "," => Self::Comma,
            ";" => Self::Terminator,
            "(" => Self::ParenL,
            ")" => Self::ParenR,
            "." => Self::Dot,
            "[" => Self::BrackL,
            "]" => Self::BrackR,
            "#" => Self::Pound,
            "=" => Self::BlockAssign,
            "@" => Self::At,
            "(*" => Self::AttrL,
            "*)" => Self::AttrR,
            "'" => Self::SingleQuote,
            ".*" => Self::DotStar,
            _ => return None,
        })
    }

    fn as_str(&self) -> &str {
        match self {
            Self::ConcatL => "{",
            Self::ConcatR => "}",
            Self::ReplicateL => "{{",
            Self::ReplicateR => "}}",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Mult => "*",
            Self::Div => "/",
            Self::Pow => "**",
            Self::Modulus => "%",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::Lte => "<=",
            Self::Gte => ">=",
            Self::LogicNeg => "!",
            Self::LogicAnd => "&&",
            Self::LogicOr => "||",
            Self::LogicEq => "==",
            Self::LogicIneq => "!=",
            Self::CaseEq => "===",
            Self::CaseIneq => "!==",
            Self::BitNeg => "~",
            Self::BitReductAnd => "&",
            Self::BitReductOr => "|",
            Self::BitReductXor => "^",
            Self::BitEquivReductXnor1 => "^~",
            Self::BitEquivReductXnor2 => "~^",
            Self::ReductNand => "~&",
            Self::ReductNor => "~|",
            Self::LogicShiftL => "<<",
            Self::LogicShiftR => ">>",
            Self::ArithShiftL => "<<<",
            Self::ArithShiftR => ">>>",
            Self::Question => "?",
            Self::Colon => ":",
            Self::Comma => ",",
            Self::Terminator => ";",
            Self::ParenL => "(",
            Self::ParenR => ")",
            Self::Dot => ".",
            Self::BrackL => "[",
            Self::BrackR => "]",
            Self::Pound => "#",
            Self::BlockAssign => "=",
            Self::At => "@",
            Self::AttrL => "(*",
            Self::AttrR => "*)",
            Self::SingleQuote => "'",
            Self::DotStar => ".*",
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ToColor for Operator {
    fn to_color(&self) -> ColoredString {
        self.to_string().normal()
    }
}
