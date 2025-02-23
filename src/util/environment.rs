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

use crate::core::config::Config;
use crate::util::anyerror::Fault;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Read;
use std::io::Write;

use crate::core::ip::Ip;
use crate::util::filesystem::Standardize;
use std::collections::btree_set::IntoIter;
use std::collections::btree_set::Iter;
use std::path::PathBuf;

use std::collections::btree_set::BTreeSet;

#[derive(Eq)]
pub struct EnvVar {
    key: String,
    value: String,
}

impl PartialEq for EnvVar {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Ord for EnvVar {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl PartialOrd for EnvVar {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl Hash for EnvVar {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // only hash by the key name
        self.key.hash(state);
    }
}

impl EnvVar {
    pub fn with(key: &str, value: &str) -> Self {
        Self::new().key(key).value(value)
    }

    pub fn new() -> Self {
        Self {
            key: String::new(),
            value: String::new(),
        }
    }

    /// Sets the environment key.
    pub fn key(mut self, s: &str) -> Self {
        // normalize the key name upon entry
        self.key = s.to_ascii_uppercase().replace('-', "_");
        self
    }

    /// Sets the environment value.
    pub fn value(mut self, s: &str) -> Self {
        self.value = s.to_owned();
        self
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    /// Transforms the string format into a orbit variable format.
    ///
    /// The rules are that the key's '_' become '.' and all letters become lowercase.
    /// The value is left unmodified.
    pub fn to_variable(&self) -> (String, String) {
        (
            self.key.replace("_", ".").to_lowercase(),
            self.value.to_owned(),
        )
    }
}

impl std::fmt::Debug for EnvVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}=\"{}\"", self.key, self.value)
    }
}

impl std::fmt::Display for EnvVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

impl Environment {
    /// Sets environment variables from a '.env' file living at `root`.
    ///
    /// Silently skips text lines that do not have proper delimiter `=` between key and value.
    /// This function will not add any environment variables if the file does not exist.
    pub fn from_env_file(mut self, root: &std::path::PathBuf) -> Result<Self, Fault> {
        // read the .env file
        let env_file = root.join(DOT_ENV_FILE);
        if env_file.exists() == true {
            let mut file = std::fs::File::open(env_file).expect("failed to open .env file");
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .expect("failed to read contents");
            // transform into environment variables
            for line in contents.split_terminator('\n') {
                let result = line.split_once('=');
                // set env variables
                if let Some((name, value)) = result {
                    self.insert(EnvVar::new().key(name).value(value));
                }
            }
        }
        Ok(self)
    }

    pub fn into_map(&self) -> HashMap<&String, &String> {
        self.0.iter().map(|v| (&v.key, &v.value)).collect()
    }

    /// Loads environment variables from a target [Ip].
    pub fn from_ip(mut self, ip: &Ip) -> Result<Self, Fault> {
        self.insert(
            EnvVar::new()
                .key(ORBIT_IP_NAME)
                .value(&ip.get_man().get_ip().get_name().to_string()),
        );
        self.insert(
            EnvVar::new()
                .key(ORBIT_IP_UUID)
                .value(&ip.get_uuid().to_string()),
        );
        self.insert(
            EnvVar::new()
                .key(ORBIT_IP_VERSION)
                .value(&ip.get_man().get_ip().get_version().to_string()),
        );
        self.insert(
            EnvVar::new()
                .key(ORBIT_IP_LIBRARY)
                .value(&ip.get_hdl_library().to_string()),
        );
        self.insert(
            EnvVar::new()
                .key(ORBIT_MANIFEST_DIR)
                .value(PathBuf::standardize(&ip.get_root()).to_str().unwrap()),
        );
        if let Some(sum) = ip.get_checksum() {
            self.insert(
                EnvVar::new()
                    .key(ORBIT_IP_CHECKSUM)
                    .value(&sum.to_string_short()),
            );
        }
        Ok(self)
    }

    /// Loads an `Environment` struct from a `Config` document.
    ///
    /// It searches the `[env]` table and collects all env variables.
    pub fn from_config(mut self, config: &Config) -> Result<Self, Fault> {
        // read config.toml for setting any env variables
        if let Some(map) = config.get_env() {
            map.iter().for_each(|(key, val)| {
                self.insert(
                    EnvVar::new()
                        .key(&format!("{}{}", ORBIT_ENV_PREFIX, key))
                        .value(val),
                );
            });
        }
        Ok(self)
    }

    /// Sets a set of environment variables, consuming the list.
    pub fn initialize(self) -> () {
        self.into_iter()
            .for_each(|e| std::env::set_var(e.key, e.value));
    }

    pub fn read(key: &str) -> Option<String> {
        match std::env::var(key) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}

/// Stores a list of `EnvVar` at root in a file named ".env".
pub fn save_environment(env: &Environment, root: &std::path::PathBuf) -> Result<(), Fault> {
    // create the file
    let mut env_file =
        std::fs::File::create(&root.join(".env")).expect("could not create .env file");
    // prepare the data into a single string for writing
    let contents = env
        .iter()
        .fold(String::new(), |x, y| x + &y.to_string() + &"\n");
    // write the data to the file
    env_file
        .write_all(contents.as_bytes())
        .expect("failed to write data to .env file");
    Ok(())
}

pub struct Environment(BTreeSet<EnvVar>);

impl Environment {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn insert(&mut self, var: EnvVar) -> bool {
        self.0.insert(var)
    }

    pub fn add(mut self, var: EnvVar) -> Self {
        self.0.insert(var);
        self
    }

    pub fn overwrite(mut self, var: EnvVar) -> Self {
        match self.0.contains(&var) {
            true => self.0.remove(&var),
            false => true,
        };
        self.0.insert(var);
        self
    }

    pub fn iter(&self) -> Iter<'_, EnvVar> {
        self.0.iter()
    }

    pub fn into_iter(self) -> IntoIter<EnvVar> {
        self.0.into_iter()
    }

    pub fn from_vec(vec: Vec<EnvVar>) -> Self {
        let mut inner = BTreeSet::new();
        vec.into_iter().for_each(|e| {
            inner.insert(e);
            ()
        });
        Self(inner)
    }

    pub fn get(&self, key: &str) -> Option<&EnvVar> {
        self.0.get(&EnvVar::new().key(key))
    }
}

pub const DOT_ENV_FILE: &str = ".env";

pub const ORBIT_HOME: &str = "ORBIT_HOME";
pub const NO_COLOR: &str = "NO_COLOR";
pub const ORBIT_WIN_LITERAL_CMD: &str = "ORBIT_WIN_LITERAL_CMD";

pub const ORBIT_MANIFEST_DIR: &str = "ORBIT_MANIFEST_DIR";
pub const ORBIT_IP_NAME: &str = "ORBIT_IP_NAME";
pub const ORBIT_IP_UUID: &str = "ORBIT_IP_UUID";
pub const ORBIT_IP_VERSION: &str = "ORBIT_IP_VERSION";
pub const ORBIT_IP_LIBRARY: &str = "ORBIT_IP_LIBRARY";
pub const ORBIT_IP_CHECKSUM: &str = "ORBIT_IP_CHECKSUM";

pub const ORBIT_TARGET: &str = "ORBIT_TARGET";

pub const ORBIT_TOP_NAME: &str = "ORBIT_TOP_NAME";
pub const ORBIT_TOP_FILE: &str = "ORBIT_TOP_FILE";
pub const ORBIT_TOP_JSON: &str = "ORBIT_TOP_JSON";

pub const ORBIT_TB_NAME: &str = "ORBIT_TB_NAME";
pub const ORBIT_TB_FILE: &str = "ORBIT_TB_FILE";
pub const ORBIT_TB_JSON: &str = "ORBIT_TB_JSON";

pub const ORBIT_DUT_NAME: &str = "ORBIT_DUT_NAME";
pub const ORBIT_DUT_FILE: &str = "ORBIT_DUT_FILE";
pub const ORBIT_DUT_JSON: &str = "ORBIT_DUT_JSON";

pub const ORBIT_BLUEPRINT: &str = "ORBIT_BLUEPRINT";
pub const ORBIT_TARGET_DIR: &str = "ORBIT_TARGET_DIR";
pub const ORBIT_OUT_DIR: &str = "ORBIT_OUT_DIR";

pub const ORBIT_CHAN_INDEX: &str = "ORBIT_CHAN_INDEX";

pub const ORBIT_ENV_PREFIX: &str = "ORBIT_ENV_";
