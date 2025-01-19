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

use crate::core::lang::vhdl::token::ToColor;
use colored::ColoredString;
use colored::Colorize;
use serde_derive::Serialize;
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Keyword {
    Abs,          // VHDL-1987 LRM - current
    Access,       // VHDL-1987 LRM - current
    After,        // VHDL-1987 LRM - current
    Alias,        // VHDL-1987 LRM - current
    All,          // VHDL-1987 LRM - current
    And,          // VHDL-1987 LRM - current
    Architecture, // VHDL-1987 LRM - current
    Array,        // VHDL-1987 LRM - current
    Assert,       // VHDL-1987 LRM - current
    Assume,
    // AssumeGuarantee "assume_guarantee" is omitted from VHDL-2019 LRM
    Attribute,     // VHDL-1987 LRM - current
    Begin,         // VHDL-1987 LRM - current
    Block,         // VHDL-1987 LRM - current
    Body,          // VHDL-1987 LRM - current
    Buffer,        // VHDL-1987 LRM - current
    Bus,           // VHDL-1987 LRM - current
    Case,          // VHDL-1987 LRM - current
    Component,     // VHDL-1987 LRM - current
    Configuration, // VHDL-1987 LRM - current
    Constant,      // VHDL-1987 LRM - current
    Context,
    Cover,
    Default,
    Disconnect, // VHDL-1987 LRM - current
    Downto,     // VHDL-1987 LRM - current
    Else,       // VHDL-1987 LRM - current
    Elsif,      // VHDL-1987 LRM - current
    End,        // VHDL-1987 LRM - current
    Entity,     // VHDL-1987 LRM - current
    Exit,       // VHDL-1987 LRM - current
    Fairness,
    File, // VHDL-1987 LRM - current
    For,  // VHDL-1987 LRM - current
    Force,
    Function, // VHDL-1987 LRM - current
    Generate, // VHDL-1987 LRM - current
    Generic,  // VHDL-1987 LRM - current
    Group,
    Guarded, // VHDL-1987 LRM - current
    If,      // VHDL-1987 LRM - current
    Impure,
    In, // VHDL-1987 LRM - current
    Inertial,
    Inout,   // VHDL-1987 LRM - current
    Is,      // VHDL-1987 LRM - current
    Label,   // VHDL-1987 LRM - current
    Library, // VHDL-1987 LRM - current
    Linkage, // VHDL-1987 LRM - current
    Literal,
    Loop,    // VHDL-1987 LRM - current
    Map,     // VHDL-1987 LRM - current
    Mod,     // VHDL-1987 LRM - current
    Nand,    // VHDL-1987 LRM - current
    New,     // VHDL-1987 LRM - current
    Next,    // VHDL-1987 LRM - current
    Nor,     // VHDL-1987 LRM - current
    Not,     // VHDL-1987 LRM - current
    Null,    // VHDL-1987 LRM - current
    Of,      // VHDL-1987 LRM - current
    On,      // VHDL-1987 LRM - current
    Open,    // VHDL-1987 LRM - current
    Or,      // VHDL-1987 LRM - current
    Others,  // VHDL-1987 LRM - current
    Out,     // VHDL-1987 LRM - current
    Package, // VHDL-1987 LRM - current
    Parameter,
    Port, // VHDL-1987 LRM - current
    Postponed,
    Private,
    Procedure, // VHDL-1987 LRM - current
    Process,   // VHDL-1987 LRM - current
    Property,
    Protected,
    Pure,
    Range,    // VHDL-1987 LRM - current
    Record,   // VHDL-1987 LRM - current
    Register, // VHDL-1987 LRM - current
    Reject,
    Release,
    Rem,    // VHDL-1987 LRM - current
    Report, // VHDL-1987 LRM - current
    Restrict,
    // RestrictGuarantee "restrict_guarantee" is omitted from VHDL-2019 LRM
    Return, // VHDL-1987 LRM - current
    Rol,
    Ror,
    Select, // VHDL-1987 LRM - current
    Sequence,
    Severity, // VHDL-1987 LRM - current
    Signal,   // VHDL-1987 LRM - current
    Shared,
    Sla,
    Sll,
    Sra,
    Srl,
    Strong,
    Subtype,   // VHDL-1987 LRM - current
    Then,      // VHDL-1987 LRM - current
    To,        // VHDL-1987 LRM - current
    Transport, // VHDL-1987 LRM - current
    Type,      // VHDL-1987 LRM - current
    Unaffected,
    Units,    // VHDL-1987 LRM - current
    Until,    // VHDL-1987 LRM - current
    Use,      // VHDL-1987 LRM - current
    Variable, // VHDL-1987 LRM - current
    View,
    Vmode,
    Vpkg,
    Vprop,
    Vunit,
    Wait,  // VHDL-1987 LRM - current
    When,  // VHDL-1987 LRM - current
    While, // VHDL-1987 LRM - current
    With,  // VHDL-1987 LRM - current
    Xnor,
    Xor, // VHDL-1987 LRM - current
}

impl Keyword {
    /// Attempts to match the given string of characters `s` to a VHDL keyword.
    ///
    /// Compares `s` against keywords using ascii lowercase comparison.
    pub fn match_keyword(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_ref() {
            "abs" => Self::Abs,
            "access" => Self::Access,
            "after" => Self::After,
            "alias" => Self::Alias,
            "all" => Self::All,
            "and" => Self::And,
            "architecture" => Self::Architecture,
            "array" => Self::Array,
            "assert" => Self::Assert,
            "assume" => Self::Assume,
            "attribute" => Self::Attribute,
            "begin" => Self::Begin,
            "block" => Self::Block,
            "body" => Self::Body,
            "buffer" => Self::Buffer,
            "bus" => Self::Bus,
            "case" => Self::Case,
            "component" => Self::Component,
            "configuration" => Self::Configuration,
            "constant" => Self::Constant,
            "context" => Self::Context,
            "cover" => Self::Cover,
            "default" => Self::Default,
            "disconnect" => Self::Disconnect,
            "downto" => Self::Downto,
            "else" => Self::Else,
            "elsif" => Self::Elsif,
            "end" => Self::End,
            "entity" => Self::Entity,
            "exit" => Self::Exit,
            "fairness" => Self::Fairness,
            "file" => Self::File,
            "for" => Self::For,
            "force" => Self::Force,
            "function" => Self::Function,
            "generate" => Self::Generate,
            "generic" => Self::Generic,
            "group" => Self::Group,
            "guarded" => Self::Guarded,
            "if" => Self::If,
            "impure" => Self::Impure,
            "in" => Self::In,
            "inertial" => Self::Inertial,
            "inout" => Self::Inout,
            "is" => Self::Is,
            "label" => Self::Label,
            "library" => Self::Library,
            "linkage" => Self::Linkage,
            "literal" => Self::Literal,
            "loop" => Self::Loop,
            "map" => Self::Map,
            "mod" => Self::Mod,
            "nand" => Self::Nand,
            "new" => Self::New,
            "next" => Self::Next,
            "nor" => Self::Nor,
            "not" => Self::Not,
            "null" => Self::Null,
            "of" => Self::Of,
            "on" => Self::On,
            "open" => Self::Open,
            "or" => Self::Or,
            "others" => Self::Others,
            "out" => Self::Out,
            "package" => Self::Package,
            "parameter" => Self::Parameter,
            "port" => Self::Port,
            "postponed" => Self::Postponed,
            "private" => Self::Private,
            "procedure" => Self::Procedure,
            "process" => Self::Process,
            "property" => Self::Property,
            "protected" => Self::Protected,
            "pure" => Self::Pure,
            "range" => Self::Range,
            "record" => Self::Record,
            "register" => Self::Register,
            "reject" => Self::Reject,
            "release" => Self::Release,
            "rem" => Self::Rem,
            "report" => Self::Report,
            "restrict" => Self::Restrict,
            "return" => Self::Return,
            "rol" => Self::Rol,
            "ror" => Self::Ror,
            "select" => Self::Select,
            "sequence" => Self::Sequence,
            "severity" => Self::Severity,
            "signal" => Self::Signal,
            "shared" => Self::Shared,
            "sla" => Self::Sla,
            "sll" => Self::Sll,
            "sra" => Self::Sra,
            "srl" => Self::Srl,
            "strong" => Self::Strong,
            "subtype" => Self::Subtype,
            "then" => Self::Then,
            "to" => Self::To,
            "transport" => Self::Transport,
            "type" => Self::Type,
            "unaffected" => Self::Unaffected,
            "units" => Self::Units,
            "until" => Self::Until,
            "use" => Self::Use,
            "variable" => Self::Variable,
            "view" => Self::View,
            "vmode" => Self::Vmode,
            "vpkg" => Self::Vpkg,
            "vprop" => Self::Vprop,
            "vunit" => Self::Vunit,
            "wait" => Self::Wait,
            "when" => Self::When,
            "while" => Self::While,
            "with" => Self::With,
            "xnor" => Self::Xnor,
            "xor" => Self::Xor,
            _ => return None,
        })
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Abs => "abs",
            Self::Access => "access",
            Self::After => "after",
            Self::Alias => "alias",
            Self::All => "all",
            Self::And => "and",
            Self::Architecture => "architecture",
            Self::Array => "array",
            Self::Assert => "assert",
            Self::Assume => "assume",
            Self::Attribute => "attribute",
            Self::Begin => "begin",
            Self::Block => "block",
            Self::Body => "body",
            Self::Buffer => "buffer",
            Self::Bus => "bus",
            Self::Case => "case",
            Self::Component => "component",
            Self::Configuration => "configuration",
            Self::Constant => "constant",
            Self::Context => "context",
            Self::Cover => "cover",
            Self::Default => "default",
            Self::Disconnect => "disconnect",
            Self::Downto => "downto",
            Self::Else => "else",
            Self::Elsif => "elsif",
            Self::End => "end",
            Self::Entity => "entity",
            Self::Exit => "exit",
            Self::Fairness => "fairness",
            Self::File => "file",
            Self::For => "for",
            Self::Force => "force",
            Self::Function => "function",
            Self::Generate => "generate",
            Self::Generic => "generic",
            Self::Group => "group",
            Self::Guarded => "guarded",
            Self::If => "if",
            Self::Impure => "impure",
            Self::In => "in",
            Self::Inertial => "inertial",
            Self::Inout => "inout",
            Self::Is => "is",
            Self::Label => "label",
            Self::Library => "library",
            Self::Linkage => "linkage",
            Self::Literal => "literal",
            Self::Loop => "loop",
            Self::Map => "map",
            Self::Mod => "mod",
            Self::Nand => "nand",
            Self::New => "new",
            Self::Next => "next",
            Self::Nor => "nor",
            Self::Not => "not",
            Self::Null => "null",
            Self::Of => "of",
            Self::On => "on",
            Self::Open => "open",
            Self::Or => "or",
            Self::Others => "others",
            Self::Out => "out",
            Self::Package => "package",
            Self::Parameter => "parameter",
            Self::Port => "port",
            Self::Postponed => "postponed",
            Self::Private => "private",
            Self::Procedure => "procedure",
            Self::Process => "process",
            Self::Property => "property",
            Self::Protected => "protected",
            Self::Pure => "pure",
            Self::Range => "range",
            Self::Record => "record",
            Self::Register => "register",
            Self::Reject => "reject",
            Self::Release => "release",
            Self::Rem => "rem",
            Self::Report => "report",
            Self::Restrict => "restrict",
            Self::Return => "return",
            Self::Rol => "rol",
            Self::Ror => "ror",
            Self::Select => "select",
            Self::Sequence => "sequence",
            Self::Severity => "severity",
            Self::Signal => "signal",
            Self::Shared => "shared",
            Self::Sla => "sla",
            Self::Sll => "sll",
            Self::Sra => "sra",
            Self::Srl => "srl",
            Self::Strong => "strong",
            Self::Subtype => "subtype",
            Self::Then => "then",
            Self::To => "to",
            Self::Transport => "transport",
            Self::Type => "type",
            Self::Unaffected => "unaffected",
            Self::Units => "units",
            Self::Until => "until",
            Self::Use => "use",
            Self::Variable => "variable",
            Self::View => "view",
            Self::Vmode => "vmode",
            Self::Vpkg => "vpkg",
            Self::Vprop => "vprop",
            Self::Vunit => "vunit",
            Self::Wait => "wait",
            Self::When => "when",
            Self::While => "while",
            Self::With => "with",
            Self::Xnor => "xnor",
            Self::Xor => "xor",
        }
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ToColor for Keyword {
    fn to_color(&self) -> ColoredString {
        self.to_string().blue()
    }
}
