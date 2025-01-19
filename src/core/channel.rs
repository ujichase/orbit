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

use super::target::Process;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::{
    error::Error,
    util::{anyerror::Fault, environment::Environment},
};
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

pub type Channels = Vec<Channel>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    command: String,
    args: Option<Vec<String>>,
    #[serde(skip_deserializing, skip_serializing)]
    root: PathBuf,
}

impl Process for Sequence {
    fn get_root(&self) -> &PathBuf {
        &self.root
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Channel {
    name: String,
    description: Option<String>,
    /// The directory located where the channel exists.
    #[serde(rename = "root")]
    path: Option<String>,
    sync: Option<Sequence>,
    pre: Option<Sequence>,
    post: Option<Sequence>,
    /// Run command sequences from this directory and place manifest data here during launch
    #[serde(skip_serializing, skip_deserializing)]
    root: Option<PathBuf>,
}

impl Channel {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_root(&self) -> &PathBuf {
        self.root.as_ref().unwrap()
    }

    /// Displays a plugin's information in a single line for quick glance.
    pub fn quick_info(&self) -> String {
        format!(
            "{:<24}{}",
            self.name,
            self.description.as_ref().unwrap_or(&String::new())
        )
    }

    /// Resolves the root path according to its path.
    pub fn set_root(&mut self, relative_from: PathBuf) -> Result<(), Fault> {
        // apply root to channel's path
        match &self.path {
            Some(p) => {
                let p = PathBuf::from(p);
                let fp = if p.is_relative() == true {
                    relative_from.join(p)
                } else {
                    p
                };
                if fp.exists() == false {
                    return Err(Error::ChannelPathNotFound(fp))?;
                }
                if fp.is_dir() == false {
                    return Err(Error::ChannelPathNotDir(fp))?;
                }
                self.root = Some(fp);
            }
            None => {
                self.root = Some(relative_from.clone());
            }
        }
        // apply root to any of the command sequences
        if let Some(seq) = &mut self.sync {
            seq.root = relative_from.clone();
        }
        if let Some(seq) = &mut self.pre {
            seq.root = relative_from.clone();
        }
        if let Some(seq) = &mut self.post {
            seq.root = relative_from.clone();
        }
        Ok(())
    }

    pub fn run_sync(&self, env: &Environment) -> Result<(), Fault> {
        match &self.sync {
            Some(cmd) => cmd.execute(&None, &Vec::new(), false, self.get_root(), env.into_map()),
            None => Ok(()),
        }
    }

    pub fn run_pre(&self, env: &Environment) -> Result<(), Fault> {
        match &self.pre {
            Some(cmd) => cmd.execute(&None, &Vec::new(), false, self.get_root(), env.into_map()),
            None => Ok(()),
        }
    }

    pub fn run_post(&self, env: &Environment) -> Result<(), Fault> {
        match &self.post {
            Some(cmd) => cmd.execute(&None, &Vec::new(), false, self.get_root(), env.into_map()),
            None => Ok(()),
        }
    }
}

impl Channel {
    /// Creates a string to display a list of channels.
    ///
    /// The string lists the channels in alphabetical order by `alias`.
    pub fn list_channels(chans: &mut [&&Channel]) -> String {
        let mut list = String::new();
        chans.sort_by(|a, b| a.name.cmp(&b.name));
        for c in chans {
            list += &format!("{}\n", c.quick_info());
        }
        list
    }

    pub fn sync(context: &Context) -> Result<(), Fault> {
        println!("info: {}", "synchronizing channels ...");
        let channels: Vec<&Channel> = context.get_config().get_channels().into_values().collect();
        // initialize environment
        let mut env = Environment::new()
            // read config.toml for setting any env variables
            .from_config(context.get_config())?;
        if let Some(path) = context.get_ip_path() {
            // read ip manifest for env variables
            env = env.from_ip(&Ip::load(path.clone(), true)?)?;
        }
        for c in channels {
            c.run_sync(&env)?;
        }
        Ok(())
    }
}
