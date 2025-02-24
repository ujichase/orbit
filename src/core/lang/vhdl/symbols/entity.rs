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

use std::iter::Peekable;

use serde_derive::Serialize;

use crate::core::lang::{
    reference::RefSet,
    vhdl::{error::VhdlError, format::VhdlFormat},
};

use super::{
    architecture::Architecture, color, Architectures, Delimiter, Generics, Identifier,
    InterfaceDeclarations, Keyword, Ports, Position, ToColor, Token, VhdlSymbol, VhdlToken,
    ENTITY_NAME,
};

#[derive(Debug, PartialEq, Serialize)]
pub struct Entity {
    #[serde(rename = "identifier")]
    name: Identifier,
    generics: Generics,
    ports: Ports,
    architectures: Vec<Architecture>,
    /// The set of names that were referenced in the entity.
    #[serde(skip_serializing)]
    refs: RefSet,
    /// The set of references that were identified as components.
    #[serde(skip_serializing)]
    deps: RefSet,
    #[serde(skip_serializing)]
    pos: Position,
    language: String,
}

impl Entity {
    /// Returns a new blank `Entity` struct.
    pub fn new() -> Self {
        Self {
            name: Identifier::new(),
            ports: Ports::new(),
            generics: Generics::new(),
            architectures: Vec::new(),
            refs: RefSet::new(),
            deps: RefSet::new(),
            pos: Position::new(),
            language: String::from("vhdl"),
        }
    }

    /// Creates a basic entity from a `name`. Assumes no other information is
    /// available.
    pub fn black_box(name: Identifier) -> Self {
        Self {
            name: name,
            ports: Ports::new(),
            generics: Generics::new(),
            architectures: Vec::new(),
            refs: RefSet::new(),
            deps: RefSet::new(),
            pos: Position::new(),
            language: String::from("vhdl"),
        }
    }

    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    /// Checks if the current `Entity` is a testbench.
    ///
    /// This is determined by checking if the ports list is empty.
    pub fn is_testbench(&self) -> bool {
        self.ports.is_empty()
    }

    /// Accesses the entity's identifier.
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    /// Accesses the entity's generics.
    pub fn get_generics(&self) -> &Generics {
        &self.generics
    }

    /// Accesses the entity's ports.
    pub fn get_ports(&self) -> &Ports {
        &self.ports
    }

    /// References the references for the entity.
    pub fn get_refs(&self) -> &RefSet {
        &self.refs
    }

    // Generates VHDL component code from the entity.
    pub fn into_component(&self, fmt: &VhdlFormat) -> String {
        let mut result = format!("{} ", Keyword::Component.to_color());
        result.push_str(&format!(
            "{}",
            color(&self.get_name().to_string(), ENTITY_NAME)
        ));

        let interface_depth = match fmt.is_indented_interfaces() {
            true => 2,
            false => 1,
        };

        if self.generics.0.len() > 0 {
            result.push('\n');
            if fmt.is_indented_interfaces() == true && fmt.get_tab_size() > 0 {
                result.push_str(&format!(
                    "{:<width$}",
                    " ",
                    width = fmt.get_tab_size() as usize
                ));
            }
            result.push_str(&format!("{}", Keyword::Generic.to_color()));
            result.push_str(
                &self
                    .generics
                    .0
                    .to_interface_part_string(&fmt, interface_depth)
                    .to_string(),
            );
        }
        if self.ports.0.len() > 0 {
            result.push('\n');
            if fmt.is_indented_interfaces() == true && fmt.get_tab_size() > 0 {
                result.push_str(&format!(
                    "{:<width$}",
                    " ",
                    width = fmt.get_tab_size() as usize
                ));
            }
            result.push_str(&format!("{}", Keyword::Port.to_color()));
            result.push_str(
                &self
                    .ports
                    .0
                    .to_interface_part_string(&fmt, interface_depth)
                    .to_string(),
            );
        }
        result.push_str(&format!(
            "\n{} {}{}\n",
            Keyword::End.to_color(),
            Keyword::Component.to_color(),
            Delimiter::Terminator.to_color()
        ));
        result
    }

    /// Generates VHDL signal declaration code from the entity data.
    pub fn into_signals(&self, fmt: &VhdlFormat, prefix: &str, suffix: &str) -> String {
        self.ports
            .0
            .to_declaration_part_string(Keyword::Signal, &fmt, &prefix, &suffix)
            .to_string()
    }

    /// Generates VHDL constant declaration code from the entity data.
    pub fn into_constants(&self, fmt: &VhdlFormat, prefix: &str, suffix: &str) -> String {
        self.generics
            .0
            .to_declaration_part_string(Keyword::Constant, &fmt, &prefix, &suffix)
            .to_string()
    }

    /// Generates VHDL instantiation code from the entity data.
    pub fn into_instance(
        &self,
        inst: &Option<Identifier>,
        library: &Option<Identifier>,
        fmt: &VhdlFormat,
        signal_prefix: &str,
        signal_suffix: &str,
        const_prefix: &str,
        const_suffix: &str,
    ) -> String {
        let prefix = match library {
            Some(lib) => format!(
                "{} {}{}",
                Keyword::Entity.to_color(),
                color(&lib.to_string(), ENTITY_NAME),
                Delimiter::Dot.to_color()
            ),
            None => String::new(),
        };

        let name = match &inst {
            Some(iden) => iden.clone(),
            None => Identifier::Basic(fmt.get_instance_name().to_string()),
        };

        let mapping_depth = match fmt.is_indented_interfaces() {
            true => 2,
            false => 1,
        };

        let mut result = String::new();

        result.push_str(&format!("{}", name.to_color()));
        if fmt.get_type_offset() > 0 {
            result.push_str(&format!(
                "{:<width$}",
                " ",
                width = fmt.get_type_offset() as usize
            ));
        }
        result.push_str(&format!(
            "{} {}{}",
            Delimiter::Colon.to_color(),
            prefix,
            color(&self.get_name().to_string(), ENTITY_NAME)
        ));
        if self.generics.0.len() > 0 {
            result.push('\n');
            if fmt.is_indented_interfaces() == true && fmt.get_tab_size() > 0 {
                result.push_str(&format!(
                    "{:<width$}",
                    " ",
                    width = fmt.get_tab_size() as usize
                ));
            }
            result.push_str(&(format!("{}", Keyword::Generic.to_color())));
            result.push_str(
                &self
                    .generics
                    .0
                    .to_instantiation_part(&fmt, mapping_depth, &const_prefix, &const_suffix)
                    .to_string(),
            )
        }
        if self.ports.0.len() > 0 {
            // add extra spacing
            result.push('\n');
            if fmt.is_indented_interfaces() == true && fmt.get_tab_size() > 0 {
                result.push_str(&format!(
                    "{:<width$}",
                    " ",
                    width = fmt.get_tab_size() as usize
                ));
            }
            result.push_str(&format!("{}", Keyword::Port.to_color()));
            result.push_str(
                &self
                    .ports
                    .0
                    .to_instantiation_part(&fmt, mapping_depth, &signal_prefix, &signal_suffix)
                    .to_string(),
            )
        }
        result.push_str(&Delimiter::Terminator.to_string());
        result
    }

    /// Generates list of available architectures.
    ///
    /// Note: This fn must be ran after linking entities and architectures in the
    /// current ip.
    pub fn get_architectures(&self) -> Architectures {
        Architectures::new(&self.architectures)
    }

    pub fn link_architecture(&mut self, arch: Architecture) -> () {
        self.architectures.push(arch);
    }

    /// Parses an `Entity` primary design unit from the entity's identifier to
    /// the END closing statement.
    pub fn from_tokens<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, VhdlError>
    where
        I: Iterator<Item = Token<VhdlToken>>,
    {
        // take entity name
        let entity_name = tokens.next().take().unwrap().take();
        let (generics, ports, entity_refs, entity_deps) =
            VhdlSymbol::parse_entity_declaration(tokens)?;

        let generics = generics
            .into_iter()
            .map(|f| f.0)
            .collect::<Vec<Vec<Token<VhdlToken>>>>();

        let ports = ports
            .into_iter()
            .map(|f| f.0)
            .collect::<Vec<Vec<Token<VhdlToken>>>>();

        Ok(Entity {
            name: match entity_name {
                VhdlToken::Identifier(id) => id,
                // expecting identifier
                _ => return Err(VhdlError::Vague),
            },
            architectures: Vec::new(),
            generics: Generics(InterfaceDeclarations::from_double_listed_tokens(generics)),
            ports: Ports(InterfaceDeclarations::from_double_listed_tokens(ports)),
            refs: entity_refs,
            deps: entity_deps,
            pos: pos,
            language: String::from("vhdl"),
        })
    }

    /// Accesses the references as mutable for the entity.
    pub fn get_refs_mut(&mut self) -> &mut RefSet {
        &mut self.refs
    }
}

use crate::core::lang::vhdl::symbols::tokens_to_string;

use crate::core::lang::lexer::Tokenize;
use crate::core::lang::sv::token::token::SystemVerilogToken as Svt;
use crate::core::lang::sv::token::token::SystemVerilogToken;
use crate::core::lang::sv::token::tokenizer::SystemVerilogTokenizer;

use crate::core::lang::sv::symbols::module::Module;
use crate::core::lang::sv::token::identifier::Identifier as SvIdentifier;
use crate::core::lang::sv::token::keyword::Keyword as SvKeyword;
use crate::core::lang::sv::token::operator::Operator as SvOp;
use crate::core::lang::verilog::error::VerilogError;

impl Entity {
    /// Builds a [Module] from the structured HDL data.
    pub fn to_sv_module(&self) -> Result<Module, VerilogError> {
        // start with blank slate of SystemVerilog tokens to build from
        let mut tokens: Vec<Token<Svt>> = Vec::new();

        // assemble module name
        let name = match &self.name {
            Identifier::Basic(s) => SvIdentifier::Basic(s.clone()),
            Identifier::Extended(s) => SvIdentifier::Escaped(s.clone()),
        };
        tokens.push(Self::sv(Svt::Identifier(name)));

        // add generics
        if self.generics.0.len() > 0 {
            tokens.push(Self::sv(Svt::Operator(SvOp::Pound)));
            tokens.push(Self::sv(Svt::Operator(SvOp::ParenL)));

            self.generics.0 .0.iter().for_each(|g| {
                tokens.push(Self::sv(Svt::Keyword(SvKeyword::Parameter)));

                // datatype
                tokens.push(Self::sv(Self::convert_datatype_to_sv(
                    g.get_type().get_type(),
                )));

                // ignore any ranges used for a VHDL string datatype since SV does not have explicit range for it
                if tokens
                    .last()
                    .unwrap()
                    .as_type()
                    .check_keyword(&SvKeyword::String)
                    == false
                {
                    // any ranges for that dataype?
                    if let Some(ranges) = g.get_type().get_ranges() {
                        ranges.into_iter().for_each(|r| {
                            tokens.append(&mut Self::convert_range_to_sv(r.0, r.1));
                        });
                    }
                }

                // name of the port
                let name = match &g.get_name() {
                    Identifier::Basic(s) => SvIdentifier::Basic(s.clone()),
                    Identifier::Extended(s) => SvIdentifier::Escaped(s.clone()),
                };
                tokens.push(Self::sv(Svt::Identifier(name)));

                // default value (if exists)
                if let Some(expr) = g.get_default().as_static_expr() {
                    tokens.append(&mut Self::convert_default_to_sv(expr));
                }

                // closing delimiter
                tokens.push(Self::sv(Svt::Operator(SvOp::Comma)));
            });
            // remove the final trailing comma
            tokens.pop();

            tokens.push(Self::sv(Svt::Operator(SvOp::ParenR)));
        }

        // add ports
        if self.ports.0.len() > 0 {
            tokens.push(Self::sv(Svt::Operator(SvOp::ParenL)));

            // iterate through all ports
            self.ports.0 .0.iter().for_each(|p| {
                // port direction
                let dir = match p.get_mode().as_keyword() {
                    Some(Keyword::Out) => SvKeyword::Output,
                    Some(Keyword::Inout) => SvKeyword::Inout,
                    Some(Keyword::In) => SvKeyword::Input,
                    None => SvKeyword::Input,
                    _ => panic!(),
                };
                tokens.push(Self::sv(Svt::Keyword(dir)));

                // datatype
                tokens.push(Self::sv(Self::convert_datatype_to_sv(
                    p.get_type().get_type(),
                )));

                // ignore any ranges used for a VHDL string datatype since SV does not have explicit range for it
                if tokens
                    .last()
                    .unwrap()
                    .as_type()
                    .check_keyword(&SvKeyword::String)
                    == false
                {
                    // any ranges for that dataype?
                    if let Some(ranges) = p.get_type().get_ranges() {
                        ranges.into_iter().for_each(|r| {
                            tokens.append(&mut Self::convert_range_to_sv(r.0, r.1));
                        });
                    }
                }

                // name of the port
                let name = match &p.get_name() {
                    Identifier::Basic(s) => SvIdentifier::Basic(s.clone()),
                    Identifier::Extended(s) => SvIdentifier::Escaped(s.clone()),
                };
                tokens.push(Self::sv(Svt::Identifier(name)));

                // default value (if exists)
                if let Some(expr) = p.get_default().as_static_expr() {
                    tokens.append(&mut Self::convert_default_to_sv(expr));
                }

                // closing delimiter
                tokens.push(Self::sv(Svt::Operator(SvOp::Comma)));
            });
            // remove the final comma
            tokens.pop();

            tokens.push(Self::sv(Svt::Operator(SvOp::ParenR)));
        }

        // Add proper closing before parsing
        tokens.push(Self::sv(Svt::Operator(SvOp::Terminator)));

        // parse the assembled token list to create the module
        let mut tokens = tokens.into_iter().peekable();
        Module::from_tokens(&mut tokens, Position::new(), "systemverilog")
    }

    /// Helps build SystemVerilog tokens from a VHDL context.
    fn sv(token: Svt) -> Token<Svt> {
        Token::new(token, Position::new())
    }

    /// Helps convert a VHDL token into its SystemVerilog equivalent when dealing with
    /// datatypes.
    fn convert_datatype_to_sv(token: &VhdlToken) -> SystemVerilogToken {
        match token {
            VhdlToken::Identifier(i) => match i {
                Identifier::Basic(s) => match s.to_lowercase().as_str() {
                    "bit" | "boolean" => Svt::Keyword(SvKeyword::Bit),
                    "std_logic" | "std_ulogic" | "logic" | "rlogic" | "ulogic" => {
                        Svt::Keyword(SvKeyword::Logic)
                    }
                    "std_logic_vector" | "std_ulogic_vector" | "logics" | "rlogics" | "ulogics" => {
                        Svt::Keyword(SvKeyword::Logic)
                    }
                    "character" | "char" => Svt::Keyword(SvKeyword::Byte),
                    "integer" | "natural" | "positive" => Svt::Keyword(SvKeyword::Integer),
                    "string" | "str" => Svt::Keyword(SvKeyword::String),
                    _ => Svt::Identifier(SvIdentifier::Basic(s.clone())),
                },
                Identifier::Extended(s) => Svt::Identifier(SvIdentifier::Escaped(s.clone())),
            },
            _ => panic!(),
        }
    }

    /// Helps convert a VHDL range (set of tokens) into a SystemVerilog equivalent set of tokens
    /// for specifying a range for a particular datatype.
    fn convert_range_to_sv(
        lhs: Vec<VhdlToken>,
        rhs: Vec<VhdlToken>,
    ) -> Vec<Token<SystemVerilogToken>> {
        let mut tokens = Vec::new();
        // opening bracket
        tokens.push(Self::sv(Svt::Operator(SvOp::BrackL)));

        // left side of range
        SystemVerilogTokenizer::tokenize(&tokens_to_string(&lhs).into_all_bland())
            .into_iter()
            .filter_map(|r| match r {
                Ok(r) => Some(r),
                Err(_) => None,
            })
            .filter(|r| r.as_type().is_eof() == false)
            .for_each(|t| {
                tokens.push(t);
            });

        // delimiter between range endpoints
        tokens.push(Self::sv(Svt::Operator(SvOp::Colon)));

        // right side of range
        SystemVerilogTokenizer::tokenize(&tokens_to_string(&rhs).into_all_bland())
            .into_iter()
            .filter_map(|r| match r {
                Ok(r) => Some(r),
                Err(_) => None,
            })
            .filter(|r| r.as_type().is_eof() == false)
            .for_each(|t| {
                tokens.push(t);
            });
        // closing bracket
        tokens.push(Self::sv(Svt::Operator(SvOp::BrackR)));

        tokens
    }

    /// Helps convert a VHDL default value into a SystemVerilog equivalent set of tokens.
    fn convert_default_to_sv(expr: &Vec<VhdlToken>) -> Vec<Token<SystemVerilogToken>> {
        let mut tokens = Vec::new();

        SystemVerilogTokenizer::tokenize(&tokens_to_string(&expr).into_all_bland())
            .into_iter()
            .filter_map(|r| match r {
                Ok(r) => Some(r),
                Err(_) => None,
            })
            .filter(|r| r.as_type().is_eof() == false)
            .for_each(|t| {
                tokens.push(t);
            });

        // only introduce the '=' token if we successfully transfered the VHDL to SV
        if tokens.len() > 0 {
            tokens.insert(0, Self::sv(Svt::Operator(SvOp::BlockAssign)));
        }

        tokens
    }
}
