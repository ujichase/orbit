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

use std::path::PathBuf;

use crate::commands::helps::env;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::util::environment;
use crate::util::environment::EnvVar;
use crate::util::environment::Environment;
use crate::util::environment::ORBIT_WIN_LITERAL_CMD;
use crate::util::filesystem::Standardize;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Env {
    keys: Vec<String>,
}

impl Subcommand<Context> for Env {
    fn interpret(cli: &mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(env::HELP))?;
        // collect all positional arguments
        let mut keys: Vec<String> = Vec::new();
        while let Some(c) = cli.get(Arg::positional("key"))? {
            keys.push(c);
        }
        let command = Ok(Env { keys: keys });
        command
    }

    fn execute(self, c: &Context) -> proc::Result {
        // assemble environment information
        let mut env = Environment::from_vec(vec![
            // @todo: context should own an `Environment` struct instead of this data transformation
            EnvVar::new()
                .key(environment::ORBIT_HOME)
                .value(PathBuf::standardize(c.get_home_path()).to_str().unwrap()),
            EnvVar::new()
                .key(environment::ORBIT_TARGET_DIR)
                .value(&c.get_target_dir()),
            EnvVar::new().key(environment::ORBIT_MANIFEST_DIR).value(
                PathBuf::standardize(c.get_ip_path().unwrap_or(&PathBuf::new()))
                    .to_str()
                    .unwrap(),
            ),
            EnvVar::new()
                .key(environment::NO_COLOR)
                .value(&std::env::var(environment::NO_COLOR).unwrap_or(String::new())),
        ])
        .from_config(c.get_config())?;

        // add platform-specific environment variables
        if cfg!(target_os = "windows") {
            env = env.add(
                EnvVar::new()
                    .key(ORBIT_WIN_LITERAL_CMD)
                    .value(&std::env::var(ORBIT_WIN_LITERAL_CMD).unwrap_or(String::new())),
            );
        }

        // check if in an ip to add those variables
        if let Some(ip_path) = c.get_ip_path() {
            // check ip
            if let Ok(ip) = Ip::load(ip_path.clone(), true, false) {
                env = env.from_ip(&ip)?;
            }
            // check the build directory
            env = env.from_env_file(&std::path::PathBuf::from(c.get_target_dir()))?;
        }

        self.run(env)
    }
}

impl Env {
    fn run(&self, env: Environment) -> Result<(), Box<dyn std::error::Error>> {
        let mut result = String::new();

        match self.keys.is_empty() {
            // print debugging output (all variables)
            true => {
                env.iter().for_each(|e| {
                    if result.is_empty() == false {
                        result.push('\n');
                    }
                    result.push_str(&format!("{:?}", e))
                });
            }
            false => {
                let mut initial = true;
                // print values only
                self.keys.iter().for_each(|k| {
                    if initial == false {
                        result.push('\n');
                    }
                    if let Some(entry) = env.get(k) {
                        result.push_str(&entry.get_value());
                    }
                    initial = false;
                });
            }
        }

        println!("{}", result);
        Ok(())
    }
}
