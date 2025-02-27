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

use cliproc::{cli, proc, stage::Memory, Arg, Cli, Help, Subcommand};

use crate::commands::helps::test;
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
use crate::util::anyerror::Fault;
use crate::util::environment::ORBIT_OUT_DIR;
use crate::util::environment::{EnvVar, Environment, ORBIT_TARGET_DIR};

use super::plan::{self, Plan};

#[derive(Debug, PartialEq)]
pub struct Test {
    target: Option<String>,
    args: Vec<String>,
    list: bool,
    dirty: bool,
    target_dir: Option<String>,
    force: bool,
    all: bool,
    plan: Option<Scheme>,
    verbose: bool,
    dut: Option<Identifier>,
    command: Option<String>,
    filesets: Option<Vec<Fileset>>,
    bench: Option<Identifier>,
}

impl Subcommand<Context> for Test {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(test::HELP))?;
        Ok(Test {
            // Flags
            list: cli.check(Arg::flag("list"))?,
            verbose: cli.check(Arg::flag("verbose"))?,
            force: cli.check(Arg::flag("force"))?,
            all: cli.check(Arg::flag("all"))?,
            dirty: cli.check(Arg::flag("no-clean"))?,
            // Options
            dut: cli.get(Arg::option("dut").value("unit"))?,
            bench: cli.get(Arg::option("tb").value("unit"))?,
            plan: cli.get(Arg::option("plan").value("format"))?,
            target: cli.get(Arg::option("target").value("name").switch('t'))?,
            target_dir: cli.get(Arg::option("target-dir"))?,
            command: cli.get(Arg::option("command").value("path"))?,
            filesets: cli.get_all(Arg::option("fileset").value("key=glob"))?,
            // Remaining args
            args: cli.remainder()?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // locate the plugin
        let target = c.select_target(&self.target, self.list == false, false)?;

        // display plugin list and exit
        if self.list == true {
            // try to get the default target
            let def_target = c.select_target(&None, true, false).unwrap_or(None);
            match target {
                // display entire contents about the particular plugin
                Some(tar) => println!("{}", tar.to_string()),
                // display quick overview of all plugins
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

        // check that user is in an IP directory
        c.jump_to_working_ip()?;

        // create the ip manifest
        let ip = Ip::load(c.get_ip_path().unwrap().clone(), true, false)?;

        // @todo: recreate the ip graph from the lockfile, then read each installation
        // see Install::install_from_lock_file

        // determine the build directory (command-line arg overrides configuration setting)
        let default_build_dir = c.get_target_dir();
        let target_dir = match &self.target_dir {
            Some(dir) => dir,
            None => &default_build_dir,
        };

        // gather the catalog and resolve any missing dependencies
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?;
        let catalog = plan::resolve_missing_deps(c, &ip, catalog, self.force)?;

        self.run(
            &ip,
            target_dir,
            target.get_name(),
            target,
            catalog,
            &c,
            &plan,
        )
    }
}

impl Test {
    fn run(
        &self,
        working_ip: &Ip,
        target_dir: &str,
        out_dir: &str,
        target: &Target,
        catalog: Catalog,
        c: &Context,
        scheme: &Scheme,
    ) -> Result<(), Fault> {
        let envs = Environment::new()
            // read config.toml for setting any env variables
            .from_config(c.get_config())?
            // read ip manifest for env variables
            .from_ip(&working_ip)?
            .add(EnvVar::new().key(ORBIT_TARGET_DIR).value(target_dir))
            .add(EnvVar::new().key(ORBIT_OUT_DIR).value(out_dir));

        // plan the target
        Plan::run(
            &working_ip,
            target_dir,
            target,
            catalog,
            self.dirty == false,
            self.force,
            false,
            self.all,
            &self.bench,
            &self.dut,
            &self.filesets,
            &scheme,
            true,
            true,
            envs,
        )?;

        let output_path = working_ip.get_root().join(target_dir).join(out_dir);

        // prepare for build
        let envs = Environment::new().from_env_file(&output_path)?;

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
