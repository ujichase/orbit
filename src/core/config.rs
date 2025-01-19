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

use crate::core::lang::vhdl::format::VhdlFormat;
use crate::core::manifest::FromFile;
use crate::core::protocol::Protocol;
use crate::core::protocol::Protocols;
use crate::core::target::{Target, Targets};
use crate::error::Error;
use crate::error::LastError;
use crate::util::anyerror::AnyError;
use crate::util::filesystem;
use crate::util::filesystem::Standardize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use serde_derive::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ConfigDocument {
    document: Document,
}

impl FromStr for ConfigDocument {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // verify all keys are valid during deserializing
        let _: Config = toml::from_str(s)?;
        Ok(Self {
            document: s.parse::<Document>().unwrap(),
        })
    }
}

const INCLUDE_KEY: &str = "include";
const GENERAL_KEY: &str = "general";
const BUILD_KEY: &str = "build";
const TEST_KEY: &str = "test";
const PUBLISH_KEY: &str = "publish";

const TOP_KEYS: [&str; 5] = [INCLUDE_KEY, GENERAL_KEY, BUILD_KEY, TEST_KEY, PUBLISH_KEY];

use crate::util::anyerror::Fault;
use toml_edit::Array;
use toml_edit::Document;
use toml_edit::Formatted;
use toml_edit::Item;
use toml_edit::Table;
use toml_edit::Value;

use super::channel::Channel;
use super::channel::Channels;
use super::lang::sv::format::SystemVerilogFormat;

impl Display for ConfigDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.document.to_string())
    }
}

impl ConfigDocument {
    fn append_list(table: &mut Table, key: &str, item: &str, on_newline: bool) -> () {
        // verify the key/entry exists (make empty array)
        if table.contains_key(key) == false {
            table.insert(key, Item::Value(Value::Array(Array::new())));
        }
        table[key].as_array_mut().unwrap().push(item);
        // before neat formatting of an item on every line
        table[key]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .enumerate()
            .for_each(|(i, f)| {
                if on_newline == true {
                    f.decor_mut().set_prefix("\n    ")
                } else {
                    if i > 0 {
                        f.decor_mut().set_prefix(" ")
                    } else {
                        f.decor_mut().set_prefix("")
                    }
                };
                if on_newline == true {
                    f.decor_mut().set_suffix("")
                } else {
                    f.decor_mut().set_suffix("")
                };
            });
        table[key]
            .as_array_mut()
            .unwrap()
            .set_trailing_comma(on_newline);
        if on_newline == true {
            table[key].as_array_mut().unwrap().set_trailing("\n")
        } else {
            table[key].as_array_mut().unwrap().set_trailing("")
        };
    }

    /// Adds a new value to the `include` entry.
    ///
    /// Automatically creates the new key if it does not exist.
    pub fn append_include(&mut self, item: &str) -> () {
        Self::append_list(&mut self.document, INCLUDE_KEY, item, true);
    }

    /// Pops the last value from the `include` entry.
    pub fn pop_include(&mut self) -> bool {
        let arr = match self.document[INCLUDE_KEY].as_array_mut() {
            Some(arr) => arr,
            None => return false,
        };
        let size = arr.len();
        match size {
            0 => false,
            _ => {
                arr.remove(size - 1);
                true
            }
        }
    }

    /// Checks if the current entry is set
    pub fn is_set(&self, table: Option<&str>, key: &str) -> bool {
        if let Some(table) = table {
            if let Some(map) = self.document.get(table) {
                map.as_table().unwrap().contains_key(key)
            } else {
                false
            }
        } else {
            self.document.contains_key(key)
        }
    }

    /// Sets a value for the given entry in the toml document.
    ///
    /// Creates parent table and/or key if does not exist.
    pub fn set(&mut self, table: &str, key: &str, value: &str) -> () {
        // create table if it does not exist
        if self.document.contains_key(table) == false {
            self.document.insert(table, Item::Table(Table::new()));
        }
        // create key if it does not exist
        let table = self
            .document
            .get_mut(table)
            .unwrap()
            .as_table_mut()
            .unwrap();
        // insert/overwrite into the table (make sure to use the correct type)
        if let Ok(v) = FromStr::from_str(value) {
            table.insert(key, Item::Value(Value::Integer(Formatted::<i64>::new(v))));
        } else if let Ok(v) = FromStr::from_str(value) {
            table.insert(key, Item::Value(Value::Float(Formatted::<f64>::new(v))));
        } else if let Ok(v) = FromStr::from_str(&value.to_lowercase()) {
            table.insert(key, Item::Value(Value::Boolean(Formatted::<bool>::new(v))));
        // last reserve/fallback to just storing the string
        } else {
            table.insert(
                key,
                Item::Value(Value::String(Formatted::<String>::new(value.to_string()))),
            );
        }
    }

    /// Removes an entry from the toml document.
    ///
    /// Errors if the entry does not exist.
    pub fn unset(&mut self, table: &str, key: &str) -> Result<(), Fault> {
        if self.document.contains_key(table) == false {
            return Err(AnyError(format!(
                "key \"{}.{}\" does not exist in configuration",
                table, key
            )))?;
        }
        // remnove the key if it does exist
        let toml_table = self
            .document
            .get_mut(table)
            .unwrap()
            .as_table_mut()
            .unwrap();
        match toml_table.contains_key(key) {
            true => {
                toml_table.remove(key);
                Ok(())
            }
            false => Err(AnyError(format!(
                "key \"{}.{}\" does not exist in configuration",
                table, key
            )))?,
        }
    }

    /// Writes the `document` to the `path`.
    ///
    /// Uses CONFIG_FILE as the filename to save to.
    pub fn write(&mut self, dest: &PathBuf) -> Result<(), Fault> {
        let contents = self.document.to_string();
        std::fs::write(&dest, contents)?;
        Ok(())
    }
}

impl FromFile for ConfigDocument {
    fn from_file(path: &PathBuf) -> Result<Self, Fault> {
        // open file
        let contents = std::fs::read_to_string(&path)?;
        // parse toml syntax
        match Self::from_str(&contents) {
            Ok(r) => Ok(r),
            // enter a blank lock file if failed (do not exit)
            Err(e) => {
                return Err(AnyError(format!(
                    "failed to parse configuration file at path {:?}: {}",
                    filesystem::into_std_str(path.clone()),
                    e
                )))?
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Locality {
    Global,
    Local,
    Regional,
    Include,
}

impl Display for Locality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Global => "global",
                Self::Local => "local",
                Self::Regional => "regional",
                Self::Include => "include",
            }
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Configs {
    inner: Vec<ConfigTriple>,
}

pub type ConfigTriple = (PathBuf, Config, Locality);

impl Configs {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn get_paths(&self) -> Vec<&PathBuf> {
        self.inner.iter().map(|f| &f.0).collect::<Vec<&PathBuf>>()
    }

    pub fn get_inner(&self) -> &Vec<(PathBuf, Config, Locality)> {
        &self.inner
    }

    pub fn load(mut self, base_config_path: PathBuf, lvl: Locality) -> Result<Self, Fault> {
        // create a set to remember what paths are already loaded
        let mut set = HashSet::new();

        let mut processed = Vec::new();

        // standardize the path
        let mut to_process = vec![(PathBuf::standardize(&base_config_path), lvl)];
        let mut i = 0;
        // process all paths
        while to_process.get(i).is_some() == true {
            {
                let (config_path, local) = to_process.get(i).unwrap();
                // load the entry file
                let cfg = match Config::from_file(&config_path) {
                    Ok(r) => {
                        // verify any config other than global does not have include set
                        if local != &Locality::Global && r.include.is_some() {
                            return Err(Error::ConfigLoadFailed(
                                filesystem::into_std_str(config_path.clone()),
                                LastError(Error::ConfigIncludeInNonglobal.to_string()),
                            ))?;
                        }
                        r
                    }
                    Err(e) => match local {
                        Locality::Include => {
                            return Err(Error::ConfigIncludeFailed(
                                filesystem::into_std_str(base_config_path.clone()),
                                filesystem::into_std_str(config_path.clone()),
                                LastError(e.to_string()),
                            ))?
                        }
                        _ => {
                            return Err(Error::ConfigLoadFailed(
                                filesystem::into_std_str(config_path.clone()),
                                LastError(e.to_string()),
                            ))?
                        }
                    },
                };
                set.insert(config_path.clone());
                processed.push((config_path.clone(), cfg, local.clone()));
            }

            let base = processed.last().unwrap().0.parent().unwrap().to_path_buf();
            // access its neighboring files (check "include" key)
            for next in processed.last().unwrap().1.get_includes() {
                // avoid processing the same files
                let std_next = filesystem::resolve_rel_path2(&base, next);
                if set.contains(&std_next) == false {
                    to_process.push((std_next, Locality::Include));
                }
            }
            i += 1;
        }
        // swap the first and last elements
        let last_index = processed.len() - 1;
        processed.swap(0, last_index);
        // reverse the list
        processed.reverse();
        // apped the previous values
        processed.append(&mut self.inner);
        Ok(Self { inner: processed })
    }

    pub fn get_targets(&self) -> HashMap<&str, &Target> {
        // iterate through all linked configs
        let mut map = HashMap::new();

        self.inner.iter().for_each(|(_path, cfg, _lvl)| {
            if let Some(tars) = &cfg.target {
                tars.iter().for_each(|p| match map.get(p.get_name()) {
                    Some(_) => (),
                    None => {
                        map.insert(p.get_name(), p);
                        ()
                    }
                });
            }
        });
        map
    }

    pub fn get_global(&self) -> &ConfigTriple {
        &self
            .inner
            .iter()
            .filter(|(_, _, l)| l == &Locality::Global)
            .next()
            .unwrap()
    }
}

impl From<Configs> for Config {
    /// Transform the multi-layered configurations into a single level.
    ///
    /// This function processes configurations with the following precedence:
    /// 1. LOCAL
    /// 2. PARENT
    /// 3. GLOBAL
    /// 4. INCLUDES (order-preserving)
    ///
    /// Once a value is set (not None), then it will not be overridden by any
    /// configuration file later in the processing order. The processing order is
    /// the precedence order.
    fn from(value: Configs) -> Self {
        let mut single = Config::new();
        // process the highest precedence first
        value.inner.into_iter().for_each(|(_, config, _)| {
            single.append(config);
        });
        single
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct General {
    #[serde(rename = "target-dir")]
    target_dir: Option<String>,
}

impl General {
    pub fn new() -> Self {
        Self { target_dir: None }
    }

    pub fn get_build_dir(&self) -> String {
        self.target_dir
            .as_ref()
            .unwrap_or(&String::from("target"))
            .clone()
    }

    /// Merges any populated data from `rhs` into attributes that do not already
    /// have data defined in `self`.
    pub fn merge(&mut self, rhs: Option<Self>) {
        if let Some(rhs) = rhs {
            // no build dir defined so give it the value from `rhs`
            if self.target_dir.is_some() == false {
                self.target_dir = rhs.target_dir
            }
        }
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Build {
    #[serde(rename = "default-target")]
    default_target: Option<String>,
}

impl Build {
    pub fn new() -> Self {
        Self {
            default_target: None,
        }
    }

    pub fn get_default_target(&self) -> Option<&String> {
        self.default_target.as_ref()
    }

    /// Merges any populated data from `rhs` into attributes that do not already
    /// have data defined in `self`.
    pub fn merge(&mut self, rhs: Option<Self>) {
        if let Some(rhs) = rhs {
            // no build dir defined so give it the value from `rhs`
            if self.default_target.is_some() == false {
                self.default_target = rhs.default_target
            }
        }
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Publish {
    #[serde(rename = "default-channel")]
    default_channel: Option<String>,
}

impl Publish {
    pub fn new() -> Self {
        Self {
            default_channel: None,
        }
    }

    pub fn get_default_channel(&self) -> Option<&String> {
        self.default_channel.as_ref()
    }

    /// Merges any populated data from `rhs` into attributes that do not already
    /// have data defined in `self`.
    pub fn merge(&mut self, rhs: Option<Self>) {
        if let Some(rhs) = rhs {
            // no build dir defined so give it the value from `rhs`
            if self.default_channel.is_some() == false {
                self.default_channel = rhs.default_channel
            }
        }
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Test {
    #[serde(rename = "default-target")]
    default_target: Option<String>,
}

impl Test {
    pub fn new() -> Self {
        Self {
            default_target: None,
        }
    }

    pub fn get_default_target(&self) -> Option<&String> {
        self.default_target.as_ref()
    }

    /// Merges any populated data from `rhs` into attributes that do not already
    /// have data defined in `self`.
    pub fn merge(&mut self, rhs: Option<Self>) {
        if let Some(rhs) = rhs {
            // no build dir defined so give it the value from `rhs`
            if self.default_target.is_some() == false {
                self.default_target = rhs.default_target
            }
        }
    }
}

pub const CONFIG_FILE: &str = "config.toml";

#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    include: Option<Vec<PathBuf>>,
    general: Option<General>,
    build: Option<Build>,
    test: Option<Test>,
    publish: Option<Publish>,
    env: Option<HashMap<String, String>>,
    target: Option<Targets>,
    protocol: Option<Protocols>,
    channel: Option<Channels>,
    #[serde(rename = "vhdl-format")]
    vhdl_format: Option<VhdlFormat>,
    #[serde(rename = "verilog-format")]
    systemverilog_format: Option<SystemVerilogFormat>,
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", toml::to_string_pretty(self).unwrap())
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            include: None,
            env: None,
            target: None,
            channel: None,
            protocol: None,
            vhdl_format: None,
            systemverilog_format: None,
            general: None,
            build: None,
            test: None,
            publish: None,
        }
    }

    pub fn has_include(&self) -> bool {
        self.include.is_some()
    }

    /// Adds `path` to the end of the list for the include attribute.
    ///
    /// This function creates some vector if no vector originally exists.
    pub fn append_include(&mut self, path: &str) {
        match &self.include.is_some() {
            true => self.include.as_mut().unwrap().push(PathBuf::from(path)),
            false => self.include = Some(vec![PathBuf::from(path)]),
        }
    }

    /// Adds the new information to the existing configuration to combine data.
    ///
    /// Note that the struct calling this function is the root/base config file. If there is
    /// already existing data in `self`, then it has precedence over any incoming data from `rhs`.
    pub fn append(&mut self, rhs: Self) {
        // combine 'includes' entry ... is this still needed?
        match &mut self.include {
            Some(v) => v.append(&mut rhs.include.unwrap_or(Vec::new())),
            None => self.include = rhs.include,
        }
        // combine '[general]' table
        match &mut self.general {
            Some(v) => v.merge(rhs.general),
            None => self.general = rhs.general,
        }
        // combine '[env]' table
        match &mut self.env {
            Some(v) => {
                let temp = rhs.env.unwrap_or(HashMap::new());
                for (key, val) in temp {
                    if v.contains_key(&key) == false {
                        v.insert(key, val);
                    }
                }
            }
            None => self.env = rhs.env,
        }
        // combine '[build]' table
        match &mut self.build {
            Some(v) => v.merge(rhs.build),
            None => self.build = rhs.build,
        }
        // combine '[test]' table
        match &mut self.test {
            Some(v) => v.merge(rhs.test),
            None => self.test = rhs.test,
        }
        // combine '[publish]' table
        match &mut self.publish {
            Some(v) => v.merge(rhs.publish),
            None => self.publish = rhs.publish,
        }
        // combine '[vhdl-format]' table
        match &mut self.vhdl_format {
            Some(v) => v.merge(rhs.vhdl_format),
            None => self.vhdl_format = rhs.vhdl_format,
        }
        // combine the '[systemverilog-format]' table
        match &mut self.systemverilog_format {
            Some(v) => v.merge(rhs.systemverilog_format),
            None => self.systemverilog_format = rhs.systemverilog_format,
        }
        // combine '[[target]]' array
        match &mut self.target {
            Some(v) => v.append(&mut rhs.target.unwrap_or(Vec::new())),
            None => self.target = rhs.target,
        }
        // combine '[[channel]]' array
        match &mut self.channel {
            Some(v) => v.append(&mut rhs.channel.unwrap_or(Vec::new())),
            None => self.channel = rhs.channel,
        }
        // combine '[[protocol]]' array
        match &mut self.protocol {
            Some(v) => v.append(&mut rhs.protocol.unwrap_or(Vec::new())),
            None => self.protocol = rhs.protocol,
        }
    }

    pub fn get_includes(&self) -> Vec<&PathBuf> {
        match &self.include {
            Some(i) => i.iter().collect(),
            None => Vec::new(),
        }
    }

    pub fn get_default_target(&self, is_build: bool) -> Option<&String> {
        match is_build {
            true => match &self.build {
                Some(m) => m.get_default_target(),
                None => None,
            },
            false => match &self.test {
                Some(m) => m.get_default_target(),
                None => None,
            },
        }
    }

    pub fn get_default_channel(&self) -> Option<&String> {
        match &self.publish {
            Some(p) => p.get_default_channel(),
            None => None,
        }
    }

    pub fn get_env(&self) -> &Option<HashMap<String, String>> {
        &self.env
    }

    pub fn get_protocols(&self) -> HashMap<&str, &Protocol> {
        let mut map = HashMap::new();

        if let Some(protos) = &self.protocol {
            protos.iter().for_each(|p| match map.get(p.get_name()) {
                Some(_) => (),
                None => {
                    map.insert(p.get_name(), p);
                    ()
                }
            });
        }
        map
    }

    pub fn get_targets(&self) -> HashMap<&str, &Target> {
        let mut map = HashMap::new();

        if let Some(tars) = &self.target {
            tars.iter().for_each(|t| match map.get(t.get_name()) {
                Some(_) => (),
                None => {
                    map.insert(t.get_name(), t);
                    ()
                }
            });
        }
        map
    }

    pub fn get_channels(&self) -> HashMap<&String, &Channel> {
        let mut map = HashMap::new();

        if let Some(chans) = &self.channel {
            chans.iter().for_each(|c| match map.get(c.get_name()) {
                Some(_) => (),
                None => {
                    map.insert(c.get_name(), c);
                    ()
                }
            });
        }
        map
    }

    pub fn get_vhdl_formatting(&self) -> Option<&VhdlFormat> {
        self.vhdl_format.as_ref()
    }

    pub fn get_sv_formatting(&self) -> Option<&SystemVerilogFormat> {
        self.systemverilog_format.as_ref()
    }

    pub fn get_general(&self) -> Option<&General> {
        self.general.as_ref()
    }
}

impl FromStr for Config {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

impl FromFile for Config {
    fn from_file(path: &PathBuf) -> Result<Self, Fault> {
        // verify the path exists
        if path.is_file() == false {
            return Err(AnyError(format!("file does not exist")))?;
        }
        // open file
        let contents = std::fs::read_to_string(&path)?;
        // parse toml syntax
        match Self::from_str(&contents) {
            Ok(mut r) => {
                // set roots for plugins and protocols
                let base = PathBuf::standardize(path).parent().unwrap().to_path_buf();
                if let Some(protos) = &mut r.protocol {
                    protos.iter_mut().for_each(|p| {
                        p.set_root(base.clone());
                    });
                }
                if let Some(tars) = &mut r.target {
                    tars.iter_mut().for_each(|t| {
                        t.set_root(base.clone());
                    });
                }
                if let Some(chans) = &mut r.channel {
                    for c in chans {
                        c.set_root(base.clone())?;
                    }
                }
                Ok(r)
            }
            // enter a blank lock file if failed (do not exit)
            Err(e) => return Err(e)?,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const C_0: &str = r#"
# This is a blank configuration file.
"#;

    const C_1: &str = r#"
# orbit configuration file

# list of configuration files
include = [
    "path/to/other/config.toml",
]
    
[[target]]
name = "quartus"
description = "Complete toolflow for Intel Quartus Prime backend program"
command = "python"
args = ["./targets/quartus.py"]
fileset.pin-plan = "*.board"
fileset.bdf-file = "*.bdf"

[env]
COURSE = "Reconfigurable Computing [EEL5721]"
quartus-path = "C:/intelFPGA_lite/19.1/quartus/bin64/"
VCD_VIEWER = "dwfv"

[[protocol]]
name = "kstp"
command = "python"
args = ["./download.py"]

[vhdl-format]
tab-size = 3
"#;

    #[test]
    fn parse_empty_config() {
        match Config::from_str(C_0) {
            Ok(r) => assert_eq!(r, Config::new()),
            Err(e) => {
                println!("{}", e);
                panic!("failed to parse")
            }
        }
    }

    #[test]
    fn parse_basic_config() {
        match Config::from_str(C_1) {
            Ok(r) => assert_ne!(r, Config::new()),
            Err(e) => {
                println!("{}", e);
                panic!("failed to parse")
            }
        }
    }

    #[test]
    fn linked_configs() {
        Configs::new()
            .load(PathBuf::from("./tests/t1/config.toml"), Locality::Global)
            .unwrap();
    }
}
