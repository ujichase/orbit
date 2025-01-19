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

use super::super::lexer::Position;
use super::subunit::SubUnit;
use super::symbols::VhdlSymbol;
use crate::core::lang;
use crate::core::lang::reference::RefSet;
use crate::core::lang::vhdl::symbols::VHDLParser;
use crate::core::lang::vhdl::token::identifier::Identifier;
use crate::util::anyerror::CodeFault;
use crate::util::filesystem;
use crate::{core::ip::IpSpec, error::Hint};
use std::{collections::HashMap, path::PathBuf, str::FromStr};
use toml_edit::InlineTable;

pub type PrimaryUnitStore = HashMap<Identifier, PrimaryUnit>;

#[derive(PartialEq, Hash, Eq, Debug)]
pub enum PrimaryShape {
    Entity,
    Package,
    Context,
    Configuration,
}

#[derive(PartialEq, Hash, Eq, Debug)]
pub struct PrimaryUnit {
    shape: PrimaryShape,
    unit: Unit,
}

impl PrimaryUnit {
    /// References the unit's identifier.
    pub fn get_name(&self) -> &Identifier {
        &self.unit.name
    }

    pub fn get_unit(&self) -> &Unit {
        &self.unit
    }

    pub fn steal_refs(&mut self, refs: RefSet) -> () {
        let _ = &self.unit.get_symbol_mut().unwrap().steal_refs(refs);
    }

    /// Serializes the data into a toml inline table
    pub fn to_toml(&self) -> toml_edit::Value {
        let mut item = toml_edit::Value::InlineTable(InlineTable::new());
        let tbl = item.as_inline_table_mut().unwrap();
        tbl.insert(
            "identifier",
            toml_edit::value(&self.get_name().to_string())
                .into_value()
                .unwrap(),
        );
        tbl.insert(
            "type",
            toml_edit::value(&self.to_string()).into_value().unwrap(),
        );
        item
    }

    /// Deserializes the data from a toml inline table.
    pub fn from_toml(tbl: &toml_edit::InlineTable) -> Option<Self> {
        let unit = Unit {
            name: Identifier::from_str(tbl.get("identifier")?.as_str()?).unwrap(),
            symbol: None,
            source: String::new(),
        };
        let shape = match tbl.get("type")?.as_str()? {
            "entity" => PrimaryShape::Entity,
            "package" => PrimaryShape::Package,
            "context" => PrimaryShape::Context,
            "configuration" => PrimaryShape::Configuration,
            _ => return None,
        };
        Some(Self {
            shape: shape,
            unit: unit,
        })
    }
}

impl std::fmt::Display for PrimaryUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.shape {
                PrimaryShape::Entity => "entity",
                PrimaryShape::Package => "package",
                PrimaryShape::Context => "context",
                PrimaryShape::Configuration => "configuration",
            }
        )
    }
}

#[derive(Debug)]
pub struct Unit {
    name: Identifier,
    symbol: Option<VhdlSymbol>,
    /// source code file
    source: String,
}

impl Unit {
    pub fn get_symbol(&self) -> Option<&VhdlSymbol> {
        self.symbol.as_ref()
    }

    pub fn is_usable_component(&self) -> Option<()> {
        match self.get_symbol()?.as_entity()?.is_testbench() {
            true => None,
            false => Some(()),
        }
    }

    pub fn get_symbol_mut(&mut self) -> Option<&mut VhdlSymbol> {
        self.symbol.as_mut()
    }

    pub fn get_source_file(&self) -> &str {
        &self.source
    }
}

impl std::hash::Hash for Unit {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Unit {}

pub fn collect_units(files: &Vec<String>) -> Result<HashMap<Identifier, PrimaryUnit>, CodeFault> {
    let mut result: HashMap<Identifier, PrimaryUnit> = HashMap::new();
    // iterate through all source files
    for source_file in files {
        // only read the HDL files
        if crate::core::fileset::is_vhdl(&source_file) == true {
            // parse text into VHDL symbols
            // println!("parsing vhdl: {}", &source_file);
            let contents = lang::read_to_string(&source_file)?;
            let symbols = match VHDLParser::read(&contents) {
                Ok(s) => s.into_symbols(),
                Err(e) => Err(CodeFault(Some(source_file.clone()), Box::new(e)))?,
            };

            let (pri_nodes, sub_nodes): (Vec<VhdlSymbol>, Vec<VhdlSymbol>) =
                symbols.into_iter().partition(|s| s.is_primary());

            // assemble primary nodes
            let mut pri_units: HashMap<Identifier, PrimaryUnit> = pri_nodes
                .into_iter()
                .map(|sym| {
                    let name = sym.get_name().unwrap().clone();
                    let shape = match &sym {
                        VhdlSymbol::Entity(_) => Some(PrimaryShape::Entity),
                        VhdlSymbol::Package(_) => Some(PrimaryShape::Package),
                        VhdlSymbol::Configuration(_) => Some(PrimaryShape::Configuration),
                        VhdlSymbol::Context(_) => Some(PrimaryShape::Configuration),
                        VhdlSymbol::Architecture(_) => {
                            panic!("architectures cannot be here")
                        }
                        // package bodies are usually in same design file as package
                        VhdlSymbol::PackageBody(_) => {
                            panic!("package bodies cannot be here")
                        }
                    };
                    match shape {
                        Some(s) => (
                            name.clone(),
                            PrimaryUnit {
                                shape: s,
                                unit: Unit {
                                    name: name,
                                    symbol: Some(sym),
                                    source: source_file.clone(),
                                },
                            },
                        ),
                        None => panic!("must be a primary design unit"),
                    }
                })
                .collect();

            // assemble secondary nodes
            sub_nodes
                .into_iter()
                .map(|n| match n {
                    VhdlSymbol::Architecture(arch) => SubUnit::from_arch(arch),
                    VhdlSymbol::PackageBody(pkg_body) => SubUnit::from_body(pkg_body),
                    _ => panic!("primary design units cannot be here"),
                })
                .for_each(|n| {
                    if let Some(owner) = pri_units.get_mut(n.get_entity()) {
                        owner.steal_refs(n.into_refs());
                    }
                });

            for (_key, primary) in pri_units {
                if let Some(dupe) = result.insert(primary.get_name().clone(), primary) {
                    return Err(CodeFault(
                        None,
                        Box::new(HdlNamingError::DuplicateIdentifier(
                            dupe.get_name().to_string(),
                            PathBuf::from(source_file),
                            result
                                .get(dupe.get_name())
                                .unwrap()
                                .get_unit()
                                .get_symbol()
                                .unwrap()
                                .get_position()
                                .clone(),
                            PathBuf::from(dupe.get_unit().get_source_file()),
                            dupe.get_unit().get_symbol().unwrap().get_position().clone(),
                        )),
                    ))?;
                }
            }
        }
    }
    Ok(result)
}

#[derive(Debug)]
pub enum HdlNamingError {
    DuplicateIdentifier(String, PathBuf, Position, PathBuf, Position),
    DuplicateAcrossDirect(String, IpSpec, PathBuf, Position),
    DuplicateAcrossLocal(String, IpSpec, PathBuf, Position),
}

impl std::error::Error for HdlNamingError {}

impl std::fmt::Display for HdlNamingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateIdentifier(iden, path1, loc1, path2, loc2) => {
                let current_dir = std::env::current_dir().unwrap();
                let location_1 = filesystem::remove_base(&current_dir, &path1);
                let location_2 = filesystem::remove_base(&current_dir, &path2);
                write!(f, "duplicate design units identified as \"{}\"\n\nlocation 1: {}{}\nlocation 2: {}{}{}", 
                    iden,
                    filesystem::into_std_str(location_1), loc1,
                    filesystem::into_std_str(location_2), loc2,
                    Hint::ResolveDuplicateIds1)
            }
            Self::DuplicateAcrossDirect(iden, dep, path, pos) => {
                let current_dir = std::env::current_dir().unwrap();
                let location = filesystem::remove_base(&current_dir, &path);
                write!(f, "duplicate design units identified as \"{}\"\n\nlocation: {}{}\nconflicts with direct dependency: {}{}", 
                iden,
                filesystem::into_std_str(location), pos,
                dep,
                Hint::ResolveDuplicateIds1)
            }
            Self::DuplicateAcrossLocal(iden, dep, path, pos) => {
                let current_dir = std::env::current_dir().unwrap();
                let location = filesystem::remove_base(&current_dir, &path);
                write!(f, "duplicate design units identified as \"{}\"\n\nlocation: {}{}\nconflicts with local dependency: {}{}", 
                iden,
                filesystem::into_std_str(location), pos,
                dep,
                Hint::ResolveDuplicateIds1)
            }
        }
    }
}
