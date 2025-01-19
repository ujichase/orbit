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
use std::str::FromStr;

use crate::commands::helps::config;
use crate::core;
use crate::core::config::ConfigDocument;
use crate::core::config::Locality;
use crate::core::context::Context;
use crate::core::manifest::FromFile;
use crate::error::Error;
use crate::error::Hint;
use crate::error::LastError;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use crate::util::filesystem;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Entry(String, String);

impl FromStr for Entry {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // split on first '=' sign
        match s.split_once('=') {
            Some(e) => Ok(Entry(e.0.to_owned(), e.1.to_owned())),
            None => Err(AnyError(format!("missing '=' separator"))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Config {
    path: Option<PathBuf>,
    list: bool,
    push: Vec<Entry>,
    pop: Vec<String>,
    set: Vec<Entry>,
    unset: Vec<String>,
}

impl Subcommand<Context> for Config {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(config::HELP))?;
        Ok(Config {
            // Flags
            list: cli.check(Arg::flag("list"))?,
            // Options
            push: cli
                .get_all(Arg::option("push").value("key=value"))?
                .unwrap_or(Vec::new()),
            pop: cli
                .get_all(Arg::option("pop").value("key"))?
                .unwrap_or(Vec::new()),
            set: cli
                .get_all(Arg::option("set").value("key=value"))?
                .unwrap_or(Vec::new()),
            unset: cli
                .get_all(Arg::option("unset").value("key"))?
                .unwrap_or(Vec::new()),
            // Optional positionals
            path: cli.get(Arg::positional("path"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // display list
        if self.list == true {
            c.get_all_configs()
                .get_inner()
                .iter()
                .for_each(|(p, _, l)| {
                    println!("{:<61} {}", filesystem::into_std_str(p.to_path_buf()), l)
                });
            return Ok(());
        }

        // display flattened single config
        if self.path.is_none() && self.no_options_selected() {
            println!("{}", c.get_config());
            Ok(())
        // possibly work on 1 file
        } else {
            let cfg_doc_triple = match &self.path {
                // work on one file
                Some(p) => {
                    // resolve if relative and the find in list
                    let p = filesystem::resolve_rel_path2(&std::env::current_dir().unwrap(), p);
                    // try to match
                    let selected_config_triple =
                        match c.get_all_configs().get_inner().iter().find(|f| &p == &f.0) {
                            Some(r) => r,
                            None => return Err(Error::ConfigBadPath(p, Hint::ShowConfigFiles))?,
                        };
                    if self.no_options_selected() == true {
                        println!("{}", selected_config_triple.1);
                        return Ok(());
                    }
                    // work only on this one file
                    Some((
                        ConfigDocument::from_file(&selected_config_triple.0).unwrap(),
                        selected_config_triple.0.clone(),
                        selected_config_triple.2.clone(),
                    ))
                }
                None => None,
            };
            // create all configs
            // modify the settings for the cfg file
            match cfg_doc_triple {
                Some(mut cfg) => self.run(&mut cfg),
                None => {
                    let cfg_doc_triples: Vec<(ConfigDocument, PathBuf, Locality)> = c
                        .get_all_configs()
                        .get_inner()
                        .iter()
                        .filter(|(_, _, l)| l != &Locality::Include)
                        .map(|(p, _, l)| {
                            (ConfigDocument::from_file(p).unwrap(), p.clone(), l.clone())
                        })
                        .collect();
                    self.run_all(cfg_doc_triples)
                }
            }
        }
    }
}

impl Config {
    fn no_options_selected(&self) -> bool {
        self.push.is_empty() && self.pop.is_empty() && self.set.is_empty() && self.unset.is_empty()
    }

    fn run_all(&self, mut configs: Vec<(ConfigDocument, PathBuf, Locality)>) -> Result<(), Fault> {
        // check for list appending
        for entry in &self.push {
            match entry.0.as_ref() {
                "include" => {
                    let cfg = match configs
                        .iter_mut()
                        .find(|(c, _, _)| c.is_set(None, &entry.0.as_ref()))
                    {
                        Some(cfg) => cfg,
                        None => configs.last_mut().unwrap(),
                    };
                    cfg.0.append_include(&entry.1);
                }
                _ => return Err(Error::ConfigFieldNotList(entry.0.to_string()))?,
            };
        }
        // check list for popping
        for entry in &self.pop {
            match entry.as_ref() {
                "include" => {
                    let cfg = match configs
                        .iter_mut()
                        .find(|(c, _, _)| c.is_set(None, &entry.as_ref()))
                    {
                        Some(cfg) => cfg,
                        None => configs.last_mut().unwrap(),
                    };
                    cfg.0.pop_include();
                }
                _ => return Err(Error::ConfigFieldNotList(entry.to_string()))?,
            };
        }

        for entry in &self.set {
            // split by dots to get table.key (silently ignores improper parsing)
            if let Some((table, key)) = entry.0.split_once('.') {
                let cfg = match configs
                    .iter_mut()
                    .find(|(c, _, _)| c.is_set(Some(table), &key))
                {
                    Some(cfg) => cfg,
                    None => configs.last_mut().unwrap(),
                };
                cfg.0.set(table, key, &entry.1);
            } else {
                return Err(AnyError(format!(
                    "unsupported key {:?} cannot be set",
                    entry.0
                )))?;
            }
        }

        for key in &self.unset {
            // split by dots to get table.key (silently ignores improper parsing)
            if let Some((table, key)) = key.split_once('.') {
                let cfg = match configs
                    .iter_mut()
                    .find(|(c, _, _)| c.is_set(Some(table), &key))
                {
                    Some(cfg) => cfg,
                    None => configs.last_mut().unwrap(),
                };
                cfg.0.unset(table, key)?;
            } else {
                return Err(AnyError(format!("unsupported key {:?} cannot be set", key)))?;
            }
        }

        // verify all configs
        for cfg in &configs {
            // is the config file okay?
            match core::config::Config::from_str(&cfg.0.to_string()) {
                Ok(r) => {
                    if &cfg.2 != &Locality::Global && r.has_include() {
                        return Err(Error::ConfigNotSaved(LastError(
                            Error::ConfigIncludeInNonglobal.to_string(),
                        )))?;
                    }
                }
                Err(e) => {
                    return Err(Error::ConfigNotSaved(LastError(e.to_string())))?;
                }
            }
        }

        // save all configs
        for cfg in &mut configs {
            cfg.0.write(&cfg.1)?;
        }
        Ok(())
    }

    fn run(&self, config: &mut (ConfigDocument, PathBuf, Locality)) -> Result<(), Fault> {
        // display configuration and exit
        if self.no_options_selected() == true {
            println!("{}", config.0.to_string());
            return Ok(());
        }

        // check for list appending
        for entry in &self.push {
            match entry.0.as_ref() {
                "include" => config.0.append_include(&entry.1),
                _ => return Err(Error::ConfigFieldNotList(entry.0.to_string()))?,
            };
        }
        // check list for popping
        for key in &self.pop {
            match key.as_ref() {
                "include" => config.0.pop_include(),
                _ => return Err(Error::ConfigFieldNotList(key.to_string()))?,
            };
        }

        for entry in &self.set {
            // split by dots to get table.key (silently ignores improper parsing)
            if let Some((table, key)) = entry.0.split_once('.') {
                config.0.set(table, key, &entry.1)
            } else {
                return Err(AnyError(format!(
                    "unsupported key {:?} cannot be set",
                    entry.0
                )))?;
            }
        }

        for key in &self.unset {
            // split by dots to get table.key (silently ignores improper parsing)
            if let Some((table, key)) = key.split_once('.') {
                config.0.unset(table, key)?
            } else {
                return Err(AnyError(format!("unsupported key {:?} cannot be set", key)))?;
            }
        }

        // is the config file okay?
        match core::config::Config::from_str(&config.0.to_string()) {
            Ok(r) => {
                if &config.2 != &Locality::Global && r.has_include() {
                    return Err(Error::ConfigNotSaved(LastError(
                        Error::ConfigIncludeInNonglobal.to_string(),
                    )))?;
                }
            }
            Err(e) => {
                return Err(Error::ConfigNotSaved(LastError(e.to_string())))?;
            }
        }

        config.0.write(&config.1)
    }
}
