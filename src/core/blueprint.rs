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

use crate::core::fileset;
use crate::util::anyerror::AnyError;
use cliproc::cli::Error;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;
use std::io::Write;
use std::{fs::File, path::PathBuf, str::FromStr};

use super::algo::IpFileNode;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Scheme {
    Tsv,
    // Json,
}

impl Default for Scheme {
    fn default() -> Self {
        Self::Tsv
    }
}

impl Display for Scheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Tsv => "tsv",
            }
        )
    }
}

impl FromStr for Scheme {
    type Err = AnyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "tsv" => Ok(Self::Tsv),
            // "json" => Ok(Self::Json),
            _ => Err(AnyError(format!("unknown file format: {}", s))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Instruction<'a, 'b> {
    Hdl(&'b IpFileNode<'a>),
    Auxiliary(String, String, String),
}

impl<'a, 'b> Instruction<'a, 'b> {
    pub fn write(&self, format: &Scheme) -> String {
        match &format {
            Scheme::Tsv => match &self {
                Self::Hdl(node) => {
                    // match on what type of file we have
                    let source_set = if fileset::is_verilog(node.get_file()) == true {
                        "VLOG"
                    } else if fileset::is_vhdl(node.get_file()) == true {
                        "VHDL"
                    } else if fileset::is_systemverilog(node.get_file()) == true {
                        "SYSV"
                    } else {
                        panic!("unknown file in source file set")
                    };
                    format!(
                        "{}\t{}\t{}",
                        source_set,
                        node.get_library(),
                        node.get_file()
                    )
                }
                Self::Auxiliary(key, lib, file) => format!("{}\t{}\t{}", key, lib, file),
            },
            // Scheme::Json => {
            //     todo!()
            // }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Blueprint<'a, 'b> {
    scheme: Scheme,
    steps: Vec<Instruction<'a, 'b>>,
}

impl<'a, 'b> Default for Blueprint<'a, 'b> {
    fn default() -> Self {
        Self {
            scheme: Scheme::default(),
            steps: Vec::default(),
        }
    }
}

impl<'a, 'b> Blueprint<'a, 'b> {
    pub fn new(scheme: Scheme) -> Self {
        Self {
            scheme: scheme,
            steps: Vec::new(),
        }
    }

    pub fn get_filename(&self) -> String {
        String::from(match self.scheme {
            Scheme::Tsv => "blueprint.tsv",
            // Scheme::Json => "blueprint.json",
        })
    }

    /// Add the next instruction `instr` to the blueprint.
    pub fn add(&mut self, instr: Instruction<'a, 'b>) {
        self.steps.push(instr);
    }

    pub fn write(&self, output_path: &PathBuf) -> Result<(PathBuf, usize), Error> {
        let blueprint_path = output_path.join(self.get_filename());
        let mut fd = File::create(&blueprint_path).expect("could not create blueprint file");
        // write the data
        let data = self.steps.iter().fold(String::new(), |mut acc, i| {
            acc.push_str(i.write(&self.scheme).as_ref());
            acc.push('\n');
            acc
        });
        fd.write_all(data.as_bytes())
            .expect("failed to write data to blueprint");
        Ok((blueprint_path, self.steps.len()))
    }
}
