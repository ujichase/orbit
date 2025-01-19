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

use super::format::VhdlFormat;
use super::highlight::*;
use super::token::{identifier::Identifier, ToColor};
use colored::ColoredString;
use colored::Colorize;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde_derive::Serialize;

pub fn library_statement(lib: &Identifier) -> String {
    format!(
        "{} {}{}\n",
        Keyword::Library.to_color(),
        color(&lib.to_string(), ENTITY_NAME),
        Delimiter::Terminator.to_color()
    )
}

#[derive(Debug, PartialEq)]
enum ColorTone {
    Color(ColoredString),
    Bland(String),
}

impl Display for ColorTone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Color(c) => write!(f, "{}", c),
            Self::Bland(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ColorVec(Vec<ColorTone>);

impl Display for ColorVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.0 {
            write!(f, "{}", item)?
        }
        Ok(())
    }
}

impl ColorVec {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn push_str(&mut self, s: &str) -> () {
        self.0.push(ColorTone::Bland(String::from(s)));
    }

    fn push_color(&mut self, c: ColoredString) -> () {
        self.0.push(ColorTone::Color(c));
    }

    fn push(&mut self, ct: ColorTone) -> () {
        self.0.push(ct);
    }

    fn append(&mut self, mut cv: ColorVec) -> () {
        self.0.append(&mut cv.0);
    }

    fn push_whitespace(&mut self, count: usize) -> () {
        self.0
            .push(ColorTone::Bland(format!("{:<width$}", " ", width = count)));
    }

    fn swap(mut self, index: usize, hue: Rgb) -> Self {
        let item = self.0.get_mut(index).unwrap();
        *item = ColorTone::Color(color(&item.to_string(), hue));
        self
    }

    fn into_all_bland(self) -> String {
        self.0
            .into_iter()
            .map(|f| match f {
                ColorTone::Bland(s) => s,
                ColorTone::Color(s) => String::from_utf8_lossy(s.as_bytes()).to_string(),
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Architectures<'a>(&'a Vec<super::symbols::architecture::Architecture>);

impl<'a> Architectures<'a> {
    pub fn new(archs: &'a Vec<super::symbols::architecture::Architecture>) -> Self {
        Self(archs)
    }
}

impl<'a> std::fmt::Display for Architectures<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Architectures:\n")?;
        for arch in self.0 {
            write!(f, "    {}\n", arch.get_name())?
        }
        Ok(())
    }
}
// @note: interface_signal_declaration ::= [signal] identifier_list : [ mode ] subtype_indication [ bus ] [ := static_expression ]
// @note: identifier_list ::= identifier { , identifier }

use super::super::lexer;
use crate::core::lang::vhdl::token::{delimiter::Delimiter, keyword::Keyword, VhdlToken};
use std::fmt::Display;
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub struct IdentifierList(Vec<Identifier>);

impl IdentifierList {
    fn from_tokens<I>(tokens: &mut Peekable<I>) -> Option<Self>
    where
        I: Iterator<Item = lexer::Token<VhdlToken>>,
    {
        let mut inner = Vec::new();
        // accept first identifier
        inner.push(tokens.next()?.as_ref().as_identifier()?.clone());
        while let Some(tkn) = tokens.peek() {
            // continue on commas
            if tkn.as_ref().check_delimiter(&Delimiter::Comma) == true {
                tokens.next();
            // collect more identifiers
            } else if tkn.as_ref().as_identifier().is_some() {
                inner.push(tokens.next()?.as_ref().as_identifier()?.clone());
            // break on non-identifier or comma
            } else {
                break;
            }
        }
        Some(Self(inner))
    }
}

#[derive(Debug, PartialEq)]
struct SubtypeIndication(Vec<VhdlToken>);

impl SubtypeIndication {
    fn from_tokens<I>(tokens: &mut Peekable<I>) -> Self
    where
        I: Iterator<Item = lexer::Token<VhdlToken>>,
    {
        let mut inner = Vec::new();
        while let Some(tkn) = tokens.peek() {
            // exit case: encounter 'bus' or ':=' delimiter
            if tkn.as_ref().check_keyword(&Keyword::Bus)
                || tkn.as_ref().check_delimiter(&Delimiter::VarAssign)
            {
                break;
            } else {
                inner.push(tokens.next().unwrap().take());
            }
        }
        Self(inner)
    }
}

impl Serialize for SubtypeIndication {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&tokens_to_string(&self.0).into_all_bland())
    }
}

#[derive(Debug, PartialEq)]
struct StaticExpression(Vec<VhdlToken>);

impl StaticExpression {
    fn from_tokens<I>(tokens: &mut Peekable<I>) -> Self
    where
        I: Iterator<Item = lexer::Token<VhdlToken>>,
    {
        // take remanining tokens
        Self(tokens.map(|f| f.take()).collect())
    }

    fn to_color_vec(&self) -> ColorVec {
        let mut result = ColorVec::new();
        result.push_color(Delimiter::VarAssign.to_color());
        result.push_str(" ");
        result.append(tokens_to_string(&self.0));
        result
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Generics(pub InterfaceDeclarations);

impl Generics {
    pub fn new() -> Self {
        Self(InterfaceDeclarations(Vec::new()))
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Ports(pub InterfaceDeclarations);

impl Ports {
    pub fn new() -> Self {
        Self(InterfaceDeclarations(Vec::new()))
    }

    pub fn is_empty(&self) -> bool {
        self.0 .0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0 .0.len()
    }
}

impl std::fmt::Display for StaticExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", Delimiter::VarAssign, tokens_to_string(&self.0))
    }
}

#[derive(Debug, PartialEq)]
pub struct Mode(Option<Keyword>);

impl Serialize for Mode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Mode", 1)?;
        match &self.0 {
            Some(kw) => state.serialize_field("mode", &kw.to_string().to_lowercase()),
            None => state.serialize_field("mode", &Keyword::In.to_string().to_lowercase()),
        }?;
        state.end()
    }
}

#[derive(Debug, PartialEq)]
pub struct Expr(Option<StaticExpression>);

impl Serialize for Expr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            Some(expr) => serializer.serialize_str(&tokens_to_string(&expr.0).into_all_bland()),
            None => serializer.serialize_none(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct InterfaceDeclaration {
    #[serde(rename = "identifier")]
    identifier: Identifier,
    #[serde(rename = "mode", flatten)]
    mode: Mode,
    #[serde(rename = "type")]
    datatype: SubtypeIndication,
    #[serde(rename = "default")]
    expr: Expr,
    #[serde(skip_serializing)]
    initial_keyword: Option<Keyword>,
    #[serde(skip_serializing)]
    bus_present: bool,
}

fn tokens_to_string(tokens: &Vec<VhdlToken>) -> ColorVec {
    let mut result = ColorVec::new();
    // determine which delimiters to not add trailing spaces to
    let is_spaced_token = |d: &Delimiter| match d {
        Delimiter::ParenL
        | Delimiter::ParenR
        | Delimiter::Dot
        | Delimiter::SingleQuote
        | Delimiter::DoubleStar
        | Delimiter::Dash
        | Delimiter::Plus
        | Delimiter::Star
        | Delimiter::FwdSlash => false,
        _ => true,
    };
    // determine which delimiters to not add have whitespace preceed
    let no_preceeding_whitespace = |d: &Delimiter| match d {
        Delimiter::DoubleStar | Delimiter::Comma => true,
        _ => false,
    };
    // iterate through the tokens
    let mut iter = tokens.iter().peekable();
    while let Some(t) = iter.next() {
        // determine if to add trailing space after the token
        let trailing_space = match t {
            VhdlToken::Delimiter(d) => is_spaced_token(d),
            _ => {
                // make sure the next token is not a tight token (no-spaced)
                if let Some(m) = iter.peek() {
                    match m {
                        VhdlToken::Delimiter(d) => is_spaced_token(d),
                        _ => true,
                    }
                } else {
                    true
                }
            }
        };
        result.push_color(t.to_color());
        if trailing_space == true && iter.peek().is_some() {
            if let Some(d) = iter.peek().unwrap().as_delimiter() {
                // skip whitespace addition
                if no_preceeding_whitespace(d) == true {
                    continue;
                }
            }
            result.push_str(" ");
        }
    }
    result
}

impl InterfaceDeclaration {
    fn into_interface_string(&self, offset: usize) -> ColorVec {
        let mut result = ColorVec::new();
        // identifier
        result.push_color(self.identifier.to_color());
        // whitespace
        if offset > 0 {
            result.push_whitespace(offset);
        }
        // colon
        result.push_color(Delimiter::Colon.to_color());
        result.push_str(" ");
        // port direction
        if self.mode.0.is_none()
            && self.initial_keyword.is_some()
            && self.initial_keyword.as_ref().unwrap() == &Keyword::Signal
        {
            result.push_color(Keyword::In.to_color());
            result.push_str(" ");
        } else if self.mode.0.is_some() {
            result.push_color(self.mode.0.as_ref().unwrap().to_color());
            result.push_str(" ");
        }
        // data type
        result.append(tokens_to_string(&self.datatype.0).swap(0, DATA_TYPE));
        // optional bus keyword
        if self.bus_present == true {
            result.push_str(" ");
            result.push_color(Keyword::Bus.to_color())
        }
        // rhs initial assignment
        if self.expr.0.is_some() == true {
            result.push_str(" ");
            result.append(self.expr.0.as_ref().unwrap().to_color_vec())
        }
        return result;
    }

    /// Creates a declaration string to be copied into architecture declaration parts.
    ///
    /// Note: `offset` is used for padding after the identifier string and before ':'.
    fn into_declaration_string(
        &self,
        def_keyword: &Keyword,
        offset: usize,
        prefix: &str,
        suffix: &str,
    ) -> ColorVec {
        let mut result = ColorVec::new();
        // keyword
        result.push_color(
            self.initial_keyword
                .as_ref()
                .unwrap_or(def_keyword)
                .to_color(),
        );
        result.push_str(" ");
        // identifier prefix
        result.push_color(color(&prefix.to_string(), SIGNAL_DEC_IDENTIFIER));
        // identifier
        result.push_color(color(&self.identifier.to_string(), SIGNAL_DEC_IDENTIFIER));
        // identifier suffix
        result.push_color(color(&suffix.to_string(), SIGNAL_DEC_IDENTIFIER));
        // whitespace
        if offset > 0 {
            result.push_whitespace(offset);
        }
        result.push_color(Delimiter::Colon.to_color());
        result.push_str(" ");
        // data type
        result.append(tokens_to_string(&self.datatype.0).swap(0, DATA_TYPE));
        // optional bus keyword
        if self.bus_present == true {
            result.push_str(" ");
            result.push_color(Keyword::Bus.to_color())
        }
        // rhs initial assignment
        if self.expr.0.is_some() == true {
            result.push_str(" ");
            result.append(self.expr.0.as_ref().unwrap().to_color_vec())
        }
        result
    }

    /// Creates an instantiation line to be copied into an architecture region.
    fn into_instance_string(&self, offset: usize, prefix: &str, suffix: &str) -> ColorVec {
        let mut result = ColorVec::new();

        result.push_color(color(&self.identifier.to_string(), INSTANCE_LHS_IDENTIFIER));
        result.push_whitespace(offset);
        result.push_color(Delimiter::Arrow.to_color());
        result.push_str(" ");
        result.push_color(prefix.to_string().normal());
        result.push_color(self.identifier.to_color());
        result.push_color(suffix.to_string().normal());
        result
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct InterfaceDeclarations(Vec<InterfaceDeclaration>);

impl InterfaceDeclarations {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Determines the length of the longest identifier.
    pub fn longest_identifier(&self) -> usize {
        let longest = self
            .0
            .iter()
            .max_by(|x, y| x.identifier.len().cmp(&y.identifier.len()));
        match longest {
            Some(l) => l.identifier.len(),
            None => 0,
        }
    }

    /// Creates a set of `InterfaceDeclaration`s from VHDL Tokens.
    pub fn from_double_listed_tokens(tokens: Vec<Vec<lexer::Token<VhdlToken>>>) -> Self {
        let mut inner = Vec::new();
        for statement in tokens {
            match Self::from_tokens(&mut statement.into_iter().peekable()) {
                Some(mut interface) => inner.append(&mut interface.0),
                None => (),
            }
        }
        Self(inner)
    }

    /// Parses VHDL tokens into a series of `Interface` structs.
    pub fn from_tokens<I>(tokens: &mut Peekable<I>) -> Option<Self>
    where
        I: Iterator<Item = lexer::Token<VhdlToken>>,
    {
        // check if optional 'signal'/'constant'/'file'? keyword is present
        let token = tokens.peek()?;
        let initial_keyword = if token.as_ref().as_keyword().is_some() {
            Some(tokens.next().unwrap().take().take_keyword().unwrap())
        } else {
            None
        };
        // collect all identifiers for this type of signal
        let identifiers = IdentifierList::from_tokens(tokens)?;
        // skip past ':' delimiter
        if tokens.next()?.as_ref().check_delimiter(&Delimiter::Colon) == false {
            return None;
        }
        // check if a mode exists
        let token = tokens.peek()?;
        let mode = if let Some(kw) = token.as_type().as_keyword() {
            match kw {
                Keyword::In
                | Keyword::Out
                | Keyword::Buffer
                | Keyword::Linkage
                | Keyword::Inout => true,
                _ => false,
            }
        } else {
            false
        };
        let mode = if mode {
            Some(tokens.next().unwrap().take().take_keyword().unwrap())
        } else {
            None
        };
        // collect the datatype
        let subtype = SubtypeIndication::from_tokens(tokens);

        // check if bus keyword is present
        let token = tokens.peek();
        let bus_present = if let Some(tkn) = token {
            tkn.as_ref().check_keyword(&Keyword::Bus)
        } else {
            false
        };
        if bus_present == true {
            tokens.next();
        }

        // check if an expression exists
        let token = tokens.next();
        let expr = if let Some(tkn) = token {
            if tkn.as_ref().check_delimiter(&Delimiter::VarAssign) {
                Some(StaticExpression::from_tokens(tokens))
            } else {
                None
            }
        } else {
            None
        };

        // build signals
        let mut signals = Vec::new();
        for identifier in identifiers.0 {
            let signal = InterfaceDeclaration {
                initial_keyword: initial_keyword.clone(),
                identifier: identifier,
                mode: Mode(mode.clone()),
                datatype: SubtypeIndication(subtype.0.iter().map(|f| f.clone()).collect()),
                bus_present: bus_present,
                expr: Expr(match &expr {
                    Some(e) => Some(StaticExpression(e.0.iter().map(|f| f.clone()).collect())),
                    None => None,
                }),
            };
            signals.push(signal);
        }
        Some(Self(signals))
    }

    /// Creates the body of the component list of interface connections.
    pub fn to_interface_part_string(&self, fmt: &VhdlFormat, tab_count: usize) -> ColorVec {
        let mut result = ColorVec::new();
        if fmt.is_interface_parenthesis_spaced() == true {
            result.push_str(" ");
        }
        // auto-align by first finding longest offset needed
        let offset = match fmt.is_auto_type_aligned() {
            true => self.longest_identifier(),
            false => fmt.get_type_offset() as usize,
        };
        result.push_color(Delimiter::ParenL.to_color());
        result.push_str("\n");
        for port in &self.0 {
            if port != self.0.first().unwrap() {
                result.push_color(Delimiter::Terminator.to_color());
                result.push_str("\n");
            }
            if fmt.get_tab_size() > 0 {
                result.push_whitespace(fmt.get_tab_size() as usize * tab_count);
            }
            // compute the offset of the ':' and type of declaration
            let port_offset = match fmt.is_auto_type_aligned() {
                true => offset - port.identifier.len() + fmt.get_type_offset() as usize,
                false => offset,
            };
            result.append(port.into_interface_string(port_offset));
        }
        result.push_str("\n");
        if fmt.is_indented_interfaces() == true && fmt.get_tab_size() > 0 {
            result.push_str(&format!(
                "{:<width$}",
                " ",
                width = fmt.get_tab_size() as usize
            ));
        }
        result.push_color(Delimiter::ParenR.to_color());
        result.push_color(Delimiter::Terminator.to_color());

        result
    }

    pub fn to_declaration_part_string(
        &self,
        def_keyword: Keyword,
        fmt: &VhdlFormat,
        prefix: &str,
        suffix: &str,
    ) -> ColorVec {
        let mut result = ColorVec::new();
        // auto-align by first finding longest offset needed
        let offset = match fmt.is_auto_type_aligned() {
            true => self.longest_identifier(),
            false => fmt.get_type_offset() as usize,
        };
        for port in &self.0 {
            // compute the offset of the ':' and type of declaration
            let port_offset = match fmt.is_auto_type_aligned() {
                true => offset - port.identifier.len() + fmt.get_type_offset() as usize,
                false => offset,
            };
            result.append(port.into_declaration_string(
                &def_keyword,
                port_offset,
                &prefix,
                &suffix,
            ));
            result.push_color(Delimiter::Terminator.to_color());
            result.push_str("\n");
        }
        result
    }

    pub fn to_instantiation_part(
        &self,
        fmt: &VhdlFormat,
        tab_count: usize,
        prefix: &str,
        suffix: &str,
    ) -> ColorVec {
        // auto-align by first finding longest offset needed
        let offset = match fmt.is_auto_mapping_aligned() {
            true => self.longest_identifier(),
            false => fmt.get_mapping_offset() as usize,
        };
        let mut result = ColorVec::new();
        result.push_str(" ");
        result.push_color(Keyword::Map.to_color());
        if fmt.is_interface_parenthesis_spaced() == true {
            result.push_str(" ");
        }
        result.push_color(Delimiter::ParenL.to_color());
        result.push_str("\n");

        for port in &self.0 {
            if port != self.0.first().unwrap() {
                result.push_color(Delimiter::Comma.to_color());
                result.push_str("\n");
            }
            if fmt.get_tab_size() > 0 {
                result.push_whitespace(fmt.get_tab_size() as usize * tab_count);
            }
            // compute the offset of the '=>' and connected signal
            let port_offset = match fmt.is_auto_mapping_aligned() {
                true => offset - port.identifier.len() + fmt.get_mapping_offset() as usize,
                false => offset,
            };

            // append the actual signal
            result.append(port.into_instance_string(port_offset, &prefix, &suffix));
        }
        result.push_str("\n");
        if fmt.get_tab_size() > 0 && tab_count > 1 {
            result.push_whitespace(fmt.get_tab_size() as usize * (tab_count - 1));
        }
        result.push_color(Delimiter::ParenR.to_color());
        result
    }
}

#[cfg(test)]
mod test {
    // @todo
}
