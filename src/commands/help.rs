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

use crate::commands::manuals;
use crate::util::anyerror::AnyError;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Help {
    list: bool,
    topic: Option<Topic>,
}

impl Subcommand<()> for Help {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(cliproc::Help::with(HELP))?;
        Ok(Help {
            list: cli.check(Arg::flag("list"))?,
            topic: cli.get(Arg::positional("topic"))?,
        })
    }

    fn execute(self, _: &()) -> proc::Result {
        self.run()?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
enum Topic {
    New,
    Init,
    View,
    Read,
    Get,
    Tree,
    Lock,
    Build,
    Test,
    Publish,
    Search,
    Install,
    Env,
    Config,
    Remove,
}

impl Topic {
    fn list_all() -> String {
        let list = [
            "new", "init", "view", "read", "get", "tree", "lock", "test", "build", "publish",
            "search", "install", "env", "config", "remove",
        ];
        list.into_iter().fold(String::new(), |mut acc, x| {
            acc.push_str(&format!("{}\n", x));
            acc
        })
    }
}

impl std::str::FromStr for Topic {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "new" => Self::New,
            "init" => Self::Init,
            "view" => Self::View,
            "read" => Self::Read,
            "get" => Self::Get,
            "tree" => Self::Tree,
            "lock" => Self::Lock,
            "test" => Self::Test,
            "build" => Self::Build,
            "publish" => Self::Publish,
            "search" => Self::Search,
            "install" => Self::Install,
            "env" => Self::Env,
            "config" => Self::Config,
            "remove" => Self::Remove,
            _ => return Err(AnyError(format!("topic '{}' not found", s))),
        })
    }
}

impl Topic {
    /// Transforms the variant to its corresponding manual page.
    fn as_manual(&self) -> &str {
        use Topic::*;
        match &self {
            New => manuals::new::MANUAL,
            Init => manuals::init::MANUAL,
            View => manuals::view::MANUAL,
            Read => manuals::read::MANUAL,
            Get => manuals::get::MANUAL,
            Tree => manuals::tree::MANUAL,
            Lock => manuals::lock::MANUAL,
            Test => manuals::test::MANUAL,
            Build => manuals::build::MANUAL,
            Publish => manuals::publish::MANUAL,
            Search => manuals::search::MANUAL,
            Install => manuals::install::MANUAL,
            Env => manuals::env::MANUAL,
            Config => manuals::config::MANUAL,
            Remove => manuals::remove::MANUAL,
        }
    }
}

impl Help {
    fn run(&self) -> Result<(), AnyError> {
        if self.list == true {
            println!("{}", Topic::list_all());
        } else {
            let contents = match &self.topic {
                Some(t) => t.as_manual(),
                None => manuals::orbit::MANUAL,
            };
            // TODO: check for a pager program to pipe contents into?
            println!("{}", contents);
        }
        Ok(())
    }
}

const HELP: &str = "\
Read in-depth documentation on Orbit topics.

Usage:
    orbit help [<topic>]

Args:
    <topic>         a listed topic or any orbit subcommand

Use 'orbit help --list' to see all available topics.
";
