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

use crate::core::context::Context;
use crate::core::fileset::Style;
use crate::error::Error;
use crate::util::anyerror::Fault;
use crate::util::filesystem;
use colored::Colorize;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use super::blueprint::Scheme;
use super::swap;
use super::swap::StrSwapTable;

pub type Targets = Vec<Target>;

type Filesets = HashMap<String, Style>;

/// A user-defined backend workflow for processing the files collected
/// in the generated blueprint file.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Target {
    name: String,
    description: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
    root: Option<PathBuf>,
    command: String,
    args: Option<Vec<String>>,
    fileset: Option<Filesets>,
    plans: Option<Vec<Scheme>>,
}

impl Target {
    /// Performs variable substitution on the provided arguments for the target.
    pub fn replace_vars_in_args(mut self, vtable: &StrSwapTable) -> Self {
        self.args = if let Some(args) = self.args {
            Some(
                args.into_iter()
                    .map(|arg| swap::substitute(arg, vtable))
                    .collect(),
            )
        } else {
            self.args
        };
        self
    }

    pub fn get_filesets(&self) -> Option<&Filesets> {
        self.fileset.as_ref()
    }

    pub fn coordinate_plan(&self, plan: &Option<Scheme>) -> Result<Scheme, Error> {
        match plan {
            Some(p) => {
                // verify the plan is supported by the target
                match &self.plans {
                    // plans are listed for the target
                    Some(ps) => match ps.into_iter().find(|&i| i == p).is_some() {
                        true => Ok(p.clone()),
                        false => Err(Error::BlueprintPlanNotSupported(p.clone(), ps.clone())),
                    },
                    // no plans are listed for the target
                    None => match p == &Scheme::default() {
                        true => Ok(p.clone()),
                        false => Err(Error::BlueprintPlanMustBeDefault(
                            p.clone(),
                            Scheme::default(),
                        )),
                    },
                }
            }
            None => {
                match &self.plans {
                    // choose the first plan in the list
                    Some(ps) => match ps.first() {
                        Some(item) => Ok(item.clone()),
                        None => Ok(Scheme::default()),
                    },
                    // choose the default plan
                    None => Ok(Scheme::default()),
                }
            }
        }
    }

    /// Displays a plugin's information in a single line for quick glance.
    pub fn quick_info(&self, is_default: bool) -> String {
        format!(
            "{:<30}{}",
            format!(
                "{}{}",
                self.name,
                if is_default {
                    " [default]".blue()
                } else {
                    "".blue()
                }
            ),
            self.description.as_ref().unwrap_or(&String::new()).green(),
        )
    }

    /// Creates a string to display a list of plugins.
    ///
    /// The string lists the plugins in alphabetical order by `alias`.
    pub fn list_targets(targets: &mut [&&Target], def_target: Option<&Target>) -> String {
        let mut list = String::new();
        targets.sort_by(|a, b| a.name.cmp(&b.name));
        for t in targets {
            let is_default = def_target.is_some() && def_target.unwrap().get_name() == t.get_name();
            list += &format!("{}\n", t.quick_info(is_default));
        }
        list
    }

    /// Sets the root directory from where the command should reference paths from.
    pub fn root(mut self, root: PathBuf) -> Self {
        self.root = Some(root);
        self
    }

    pub fn set_root(&mut self, root: PathBuf) {
        self.root = Some(root);
    }

    /// References the alias to call this plugin.
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let command = filesystem::resolve_rel_path(self.get_root(), &self.command);

        let args = if self.args.is_some() {
            Some(
                self.get_args()
                    .iter()
                    .map(|f| filesystem::resolve_rel_path(self.get_root(), f))
                    .collect(),
            )
        } else {
            None
        };

        let refreshed_target = Self {
            command: command,
            args: args,
            name: self.name.clone(),
            description: self.description.clone(),
            root: self.root.clone(),
            fileset: self.fileset.clone(),
            plans: self.plans.clone(),
        };

        write!(f, "{}", toml::to_string_pretty(&refreshed_target).unwrap())
    }
}

impl FromStr for Target {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

pub trait Process {
    fn get_root(&self) -> &PathBuf;

    fn get_command(&self) -> &String;

    fn get_args(&self) -> Vec<&String>;

    /// Runs the given `command` with the set `args` for the plugin.
    fn execute(
        &self,
        overloaded_command: &Option<String>,
        extra_args: &[String],
        verbose: bool,
        cwd: &PathBuf,
        envs: HashMap<&String, &String>,
    ) -> Result<(), Fault> {
        // resolve the relative paths in the command and arguments defined in original configuration
        let command = match overloaded_command {
            Some(c) => c,
            None => self.get_command(),
        };

        let root_path = self.get_root();

        let command = filesystem::resolve_rel_path(root_path, command);

        let arguments: Vec<String> = self
            .get_args()
            .iter()
            .map(|f| filesystem::resolve_rel_path(root_path, f))
            .collect();

        // append args set on the command-line to the base-line of arguments
        let args = [&arguments, extra_args].concat();
        // display the literal command being ran
        if verbose == true {
            let s = args
                .iter()
                .fold(String::new(), |x, y| x + "\"" + &y + "\" ");
            crate::info!("running: {} {}", command, s);
        }
        let mut proc = filesystem::invoke(
            cwd,
            &command,
            &args,
            Context::enable_windows_bat_file_match(),
            envs,
        )?;
        let exit_code = proc.wait()?;
        match exit_code.code() {
            Some(num) => {
                if num != 0 {
                    Err(Error::ChildProcErrorCode(num))?
                } else {
                    Ok(())
                }
            }
            None => Err(Error::ChildProcTerminated)?,
        }
    }
}

impl Process for Target {
    fn get_root(&self) -> &PathBuf {
        &self.root.as_ref().unwrap()
    }

    fn get_args(&self) -> Vec<&String> {
        match &self.args {
            Some(list) => list.iter().map(|e| e).collect(),
            None => Vec::new(),
        }
    }

    fn get_command(&self) -> &String {
        &self.command
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Plugins {
        plugin: Vec<Target>,
    }

    impl Plugins {
        pub fn new() -> Self {
            Self { plugin: Vec::new() }
        }
    }

    impl FromStr for Plugins {
        type Err = toml::de::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            toml::from_str(s)
        }
    }

    const P_1: &str = r#" 
name = "ghdl"
description = "Backend script for simulating VHDL with GHDL."  
command = "python"
args = ["./scripts/ghdl.py"]
fileset.py-model = "{{orbit.bench}}.py"
fileset.text = "*.txt"
"#;

    const P_2: &str = r#"
name = "ffi"
command = "bash"
args = ["~/scripts/download.bash"]    
"#;

    #[test]
    fn from_toml_string() {
        let plug = Target::from_str(P_1).unwrap();
        assert_eq!(
            plug,
            Target {
                name: String::from("ghdl"),
                command: String::from("python"),
                plans: None,
                args: Some(vec![String::from("./scripts/ghdl.py")]),
                description: Some(String::from(
                    "Backend script for simulating VHDL with GHDL."
                )),
                fileset: Some(HashMap::from([
                    (
                        String::from("py-model"),
                        Style::from_str("{{orbit.bench}}.py").unwrap()
                    ),
                    (String::from("text"), Style::from_str("*.txt").unwrap()),
                ])),
                root: None,
            }
        );

        let plug = Target::from_str(P_2).unwrap();
        assert_eq!(
            plug,
            Target {
                name: String::from("ffi"),
                command: String::from("bash"),
                args: Some(vec![String::from("~/scripts/download.bash")]),
                description: None,
                plans: None,
                fileset: None,
                root: None,
            }
        );
    }

    #[test]
    fn series_of_plugins() {
        let contents = format!("{0}{1}\n{0}{2}", "[[plugin]]", P_1, P_2);
        // assemble the list of protocols
        let plugs = Plugins::from_str(&contents).unwrap();
        assert_eq!(
            plugs,
            Plugins {
                plugin: vec![
                    Target::from_str(P_1).unwrap(),
                    Target::from_str(P_2).unwrap()
                ],
            }
        );
    }
}
