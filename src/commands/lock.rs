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

use super::plan::{self, Plan};
use crate::commands::helps::lock;
use crate::core::algo;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::lockfile::LockEntry;
use crate::core::swap::StrSwapTable;
use crate::util::anyerror::Fault;
use crate::util::environment::Environment;
use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};
use colored::Colorize;

#[derive(Debug, PartialEq)]
pub struct Lock {
    force: bool,
}

impl Subcommand<Context> for Lock {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(lock::HELP))?;
        let command = Ok(Lock {
            // flags
            force: cli.check(Arg::flag("force"))?,
        });
        command
    }

    fn execute(self, c: &Context) -> proc::Result {
        // check that user is in an IP directory
        c.jump_to_working_ip()?;

        let force_apply_new_uuid = self.force;

        // store the working ip struct
        let working_ip = Ip::load(c.get_ip_path().unwrap().clone(), true, force_apply_new_uuid)?;

        // assemble the catalog
        let mut catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?;

        // TODO: recreate the ip graph from the lockfile, then read each installation
        // see Install::install_from_lock_file

        // this code is only ran if the lock file matches the manifest and we aren't force to recompute
        if working_ip.can_use_lock(&catalog) == true && self.force == false {
            let le: LockEntry = LockEntry::from((&working_ip, true));
            let lf = working_ip.get_lock();

            let env = Environment::new()
                // read config.toml for setting any env variables
                .from_config(c.get_config())?;
            let vtable = StrSwapTable::new().load_environment(&env)?;

            plan::download_missing_deps(
                vtable,
                &lf,
                &le,
                &catalog,
                &c.get_config().get_protocols(),
            )?;
            // recollect the downloaded items to update the catalog for installations
            catalog = catalog.downloads(c.get_downloads_path())?;

            plan::install_missing_deps(&lf, &le, &catalog)?;
            // recollect the installations to update the catalog for dependency graphing
            catalog = catalog.installations(c.get_cache_path())?;
        }

        Self::run(&working_ip, &catalog, self.force)
    }
}

impl Lock {
    /// Performs the backend logic for creating a blueprint file (planning a design).
    pub fn run(working_ip: &Ip, catalog: &Catalog, force: bool) -> Result<(), Fault> {
        // build entire ip graph and resolve with dynamic symbol transformation
        let ip_graph = match algo::compute_final_ip_graph(&working_ip, &catalog) {
            Ok(g) => g,
            Err(e) => return Err(e)?,
        };

        // only write lockfile and exit if flag is raised
        Plan::write_lockfile(&working_ip, &ip_graph, force, true, &catalog)?;
        Ok(())
    }

    /// Writes a lockfile for a newly created ip (one that either was made with `new` or `init`).
    pub fn write_new_lockfile(local_ip: &Ip, warn: bool) -> Result<(), Fault> {
        // build entire ip graph and resolve with dynamic symbol transformation
        let catalog = Catalog::new();
        let ip_graph = match algo::compute_final_ip_graph(&local_ip, &catalog) {
            Ok(g) => g,
            Err(e) => match warn {
                true => {
                    println!("{}: {}", "warning".yellow(), e.1);
                    algo::minimal_graph_map(local_ip)
                }
                false => return Err(e)?,
            },
        };
        Plan::write_lockfile(&local_ip, &ip_graph, true, false, &catalog)?;
        Ok(())
    }
}
