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

use super::plan;
use super::plan::Plan;
use crate::commands::helps::build;
use crate::core::blueprint::Scheme;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::fileset::Fileset;
use crate::core::ip::Ip;
use crate::core::lang::vhdl::token::Identifier;
use crate::core::swap::StrSwapTable;
use crate::core::target::Process;
use crate::core::target::Target;
use crate::error::Error;
use crate::error::LastError;
use crate::util::environment::EnvVar;
use crate::util::environment::Environment;
use crate::util::environment::ORBIT_OUT_DIR;
use crate::util::environment::ORBIT_TARGET_DIR;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Build {
    target: Option<String>,
    list: bool,
    force: bool,
    dirty: bool,
    all: bool,
    command: Option<String>,
    top: Option<Identifier>,
    plan: Option<Scheme>,
    target_dir: Option<String>,
    args: Vec<String>,
    verbose: bool,
    filesets: Option<Vec<Fileset>>,
}

impl Subcommand<Context> for Build {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(build::HELP))?;
        Ok(Build {
            // Flags
            list: cli.check(Arg::flag("list"))?,
            verbose: cli.check(Arg::flag("verbose"))?,
            force: cli.check(Arg::flag("force"))?,
            all: cli.check(Arg::flag("all"))?,
            dirty: cli.check(Arg::flag("no-clean"))?,
            // Options
            top: cli.get(Arg::option("top").value("unit"))?,
            plan: cli.get(Arg::option("plan").value("format"))?,
            target: cli.get(Arg::option("target").value("name").switch('t'))?,
            target_dir: cli.get(Arg::option("target-dir").value("dir"))?,
            command: cli.get(Arg::option("command").value("path"))?,
            filesets: cli.get_all(Arg::option("fileset").value("key=glob"))?,
            // Remaining args
            args: cli.remainder()?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // select the target
        let target = c.select_target(&self.target, self.list == false, true)?;
        // display target list and exit
        if self.list == true {
            // try to get the default target
            let def_target = c.select_target(&None, true, true).unwrap_or(None);
            match target {
                Some(t) => println!("{}", t.to_string()),
                None => print!(
                    "{}",
                    Target::list_targets(
                        &mut c
                            .get_config()
                            .get_targets()
                            .values()
                            .into_iter()
                            .collect::<Vec<&&Target>>(),
                        def_target,
                    )
                ),
            }
            return Ok(());
        }

        let target = target.unwrap();

        // coordinate the plan
        let plan = target.coordinate_plan(&self.plan)?;

        // verify running from an ip directory and enter ip's root directory
        c.jump_to_working_ip()?;

        let working_ip = Ip::load(c.get_ip_path().unwrap().to_path_buf(), true, false)?;

        // determine the build directory based on cli priority
        let default_target_dir = c.get_target_dir();
        let target_dir = self.target_dir.as_ref().unwrap_or(&default_target_dir);
        let out_dir = target.get_name();

        let output_path = working_ip.get_root().join(target_dir).join(out_dir);

        // gather the catalog and resolve any missing dependencies
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?;
        let catalog = plan::resolve_missing_deps(c, &working_ip, catalog, self.force)?;

        let envs = Environment::new()
            // read config.toml for setting any env variables
            .from_config(c.get_config())?
            // read ip manifest for env variables
            .from_ip(&working_ip)?
            .add(EnvVar::with(ORBIT_TARGET_DIR, target_dir))
            .add(EnvVar::with(ORBIT_OUT_DIR, out_dir));

        // plan for the provided target
        Plan::run(
            &working_ip,
            target_dir,
            target,
            catalog,
            self.dirty == false,
            self.force,
            false,
            self.all,
            &None,
            &self.top,
            &self.filesets,
            &plan,
            false,
            false,
            envs,
        )?;

        let envs = Environment::new().from_env_file(&output_path)?;

        // modify the target to update with the available
        let swap_table = StrSwapTable::new().load_environment(&envs)?;
        let target = target.clone().replace_vars_in_args(&swap_table);

        // run the command from the output path
        crate::info!("executing target {}", target.get_name().green());
        match target.execute(
            &self.command,
            &self.args,
            self.verbose,
            &output_path,
            envs.into_map(),
        ) {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::TargetProcFailed(LastError(e.to_string())))?,
        }
    }
}
