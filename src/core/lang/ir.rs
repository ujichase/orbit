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

// Reference for mixing languages: https://docs.amd.com/r/en-US/ug901-vivado-synthesis/Mixed-Language-Support

use serde_derive::Deserialize;

use super::vhdl::error::VhdlError;
use super::Lang;
use std::str::FromStr;

use crate::core::lang::sv::symbols::module::Module;
use crate::core::lang::vhdl::symbols::entity::Entity;

use super::lexer::Position;
use super::sv::token::token::SystemVerilogToken as Svt;
use super::vhdl::token::VhdlToken as Vvt;
use crate::core::lang::lexer::Token;

#[derive(Deserialize)]
struct Generic {
    identifier: String,
    mode: String,
    #[serde(rename = "type")]
    kind: Option<String>,
    default: Option<String>,
}

#[derive(Deserialize)]
struct Port {
    identifier: String,
    mode: String,
    #[serde(rename = "type")]
    kind: Option<String>,
    default: Option<String>,
}

/// Intermediate representation to convert HDL between the various languages.
#[derive(Deserialize)]
pub struct HdlIr {
    identifier: String,
    generics: Vec<Generic>,
    ports: Vec<Port>,
    architectures: Vec<String>,
    language: Lang,
}

impl FromStr for HdlIr {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl HdlIr {
    pub fn from_vhdl_entity(entity: &Entity) -> Self {
        Self::from_str(&serde_json::to_string(entity).unwrap()).unwrap()
    }

    pub fn from_sv_module(module: &Module) -> Self {
        Self::from_str(&serde_json::to_string(module).unwrap()).unwrap()
    }

    pub fn from_vlog_module(module: &Module) -> Self {
        Self::from_str(&serde_json::to_string(module).unwrap()).unwrap()
    }
}

use crate::core::lang::sv::token::identifier::Identifier as SvIdentifier;
// use crate::core::lang::sv::token::keyword::Keyword as SvKeyword;
use crate::core::lang::sv::token::operator::Operator as SvOp;
use crate::core::lang::verilog::error::VerilogError;

use crate::core::lang::vhdl::token::identifier::Identifier as VhIdentifier;
use crate::core::lang::vhdl::token::keyword::Keyword as VhKeyword;

impl HdlIr {
    /// Builds a [Module] from the structured HDL data.
    pub fn into_sv_module(self) -> Result<Module, VerilogError> {
        if self.language == Lang::SystemVerilog || self.language == Lang::Verilog {
            panic!("cannot use IR to convert to already native language (sv/verilog)")
        }

        // start with blank slate of SystemVerilog tokens to build from
        let mut tokens: Vec<Token<Svt>> = Vec::new();

        // NOTE: function assumes the module keyword was already processed
        //  tokens.push(Self::sv(Svt::Keyword(SvKeyword::Module)));

        // assemble module name
        tokens.push(Self::sv(Svt::Identifier(SvIdentifier::from_str(
            &self.identifier,
        )?)));

        // add generics

        // add ports

        // Add proper closing before parsing
        tokens.push(Self::sv(Svt::Operator(SvOp::Terminator)));

        // parse the assembled token list to create the module
        let mut tokens = tokens.into_iter().peekable();
        Module::from_tokens(&mut tokens, Position::new(), "systemverilog")
    }

    /// Builds an [Entity] from the structured HDL data.
    pub fn into_vhdl_entity(self) -> Result<Entity, VhdlError> {
        if self.language == Lang::Vhdl {
            panic!("cannot use IR to convert to already native language (vhdl)")
        }
        // start with blank slate of VHDL tokens to build from
        let mut tokens: Vec<Token<Vvt>> = Vec::new();

        // assemble entity name
        tokens.push(Self::vh(Vvt::Identifier(VhIdentifier::from_str(
            &self.identifier,
        )?)));

        // add IS keyword
        tokens.push(Self::vh(Vvt::Keyword(VhKeyword::Is)));

        // add generics

        // add ports

        // add proper closing before parsing
        tokens.push(Self::vh(Vvt::Keyword(VhKeyword::End)));

        // parse the assembled token list to create the module
        let mut tokens = tokens.into_iter().peekable();
        Entity::from_tokens(&mut tokens, Position::new())
    }
}

impl HdlIr {
    fn sv(token: Svt) -> Token<Svt> {
        Token::new(token, Position::new())
    }

    fn vh(token: Vvt) -> Token<Vvt> {
        Token::new(token, Position::new())
    }
}
