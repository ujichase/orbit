use crate::core::v2::plugin::PluginError;
use crate::util::environment::EnvVar;
use crate::util::environment::Environment;
use clif::cmd::{FromCli, Command};
use clif::Cli;
use clif::arg::{Optional, Flag};
use clif::Error as CliError;
use crate::OrbitResult;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;
use crate::core::v2::plugin::Plugin;
use crate::core::v2::plugin::Process;
use crate::util::environment;
use crate::util::environment::ORBIT_BLUEPRINT;
use crate::util::environment::ORBIT_BUILD_DIR;
use super::plan::BLUEPRINT_FILE;

use crate::core::v2::ip::Ip;

#[derive(Debug, PartialEq)]
pub struct Build {
    alias: Option<String>,
    list: bool,
    command: Option<String>,
    build_dir: Option<String>,
    args: Vec<String>,
    verbose: bool,
}

impl FromCli for Build {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Build {
            alias: cli.check_option(Optional::new("plugin").value("alias"))?,
            list: cli.check_flag(Flag::new("list"))?,
            verbose: cli.check_flag(Flag::new("verbose"))?,
            build_dir: cli.check_option(Optional::new("build-dir").value("dir"))?,
            command: cli.check_option(Optional::new("command").value("cmd"))?,
            args: cli.check_remainder()?,
        });
        command
    }
}

impl Command<Context> for Build {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // try to find plugin matching `command` name under the `alias`
        let plug = if let Some(name) = &self.alias {
            match c.get_plugins().get(name) {
                Some(p) => Some(p),
                None => return Err(PluginError::Missing(name.to_string()))?,
            }
        } else {
            None
        };
        // display plugin list and exit
        if self.list == true {
            match plug {
                Some(plg) => println!("{}", plg),
                None => println!("{}", Plugin::list_plugins(&mut c.get_plugins().values().into_iter().collect::<Vec<&Plugin>>())),
            }
            return Ok(())
        }

        // verify only 1 option is provided
        if self.command.is_some() && self.alias.is_some() {
            return Err(AnyError(format!("cannot execute both a plugin and command")))?
        }
        // verify running from an IP directory and enter IP's root directory
        c.goto_ip_path()?;

        // determine the build directory based on cli priority
        let b_dir = self.build_dir.as_ref().unwrap_or(c.get_build_dir());

        // todo: is this necessary? -> no, but maybe add a flag/option to bypass (and also allow plugins to specify if they require blueprint in settings)
        // idea: [[plugin]] require-plan = false
        // assert a blueprint file exists in the specified build directory
        if c.get_ip_path().unwrap().join(b_dir).join(BLUEPRINT_FILE).exists() == false {
            return Err(AnyError(format!("no blueprint file to build from in directory '{}'\n\nTry `orbit plan --build-dir {0}` to generate a blueprint file", b_dir)))?
        }

        Environment::new()
            // read config.toml for setting any env variables
            .from_config(c.get_config())?
            // read ip manifest for env variables
            .from_ip(&Ip::load(c.get_ip_path().unwrap().clone())?)?
            .add(EnvVar::new().key(ORBIT_BLUEPRINT).value(BLUEPRINT_FILE))
            .add(EnvVar::new().key(ORBIT_BUILD_DIR).value(b_dir))
            .initialize();

        // load from .env file from the correct build dir
        let envs = Environment::new()
            .from_env_file(&c.get_ip_path().unwrap().join(b_dir))?;

        // check if ORBIT_PLUGIN was set and no command option was set
        let plug = match plug {
            // already configured from the command-line
            Some(plg) => Some(plg),
            // was not set on the command-line
            None => {
                if let Some(plug) = envs.get(environment::ORBIT_PLUGIN) {
                    // verify there was no command option to override default plugin call
                    if self.command.is_none() { 
                        match c.get_plugins().get(plug.get_value()) {
                            Some(p) => Some(p),
                            None => return Err(PluginError::Missing(plug.get_value().to_string()))?,
                        }
                    } else { 
                        None 
                    }
                } else {
                    None
                }
            }
        };

        envs.initialize();

        if plug.is_none() && self.command.is_none() {
            return Err(AnyError(format!("pass a plugin or a Command<Context> for building")))?
        }

        self.run(plug)
    }
}

impl Build {

    fn run(&self, plug: Option<&Plugin>) -> Result<(), Box<dyn std::error::Error>> {
        // if there is a match run with the plugin then run it
        if let Some(p) = plug {
            p.execute(&self.args, self.verbose)
        } else if let Some(cmd) = &self.command {
            if self.verbose == true {
                let s = self.args.iter().fold(String::new(), |x, y| { x + "\"" + &y + "\" " });
                println!("running: {} {}", cmd, s);
            }
            let mut proc = crate::util::filesystem::invoke(cmd, &self.args, Context::enable_windows_bat_file_match())?;
            let exit_code = proc.wait()?;
            match exit_code.code() {
                Some(num) => if num != 0 { Err(AnyError(format!("exited with error code: {}", num)))? } else { Ok(()) },
                None =>  Err(AnyError(format!("terminated by signal")))?
            }
        } else {
            Ok(())
        }
    }
}

const HELP: &str = "\
Execute a backend tool/workflow.

Usage:
    orbit build [options] [--] [args]...

Options:
    --plugin <alias>    plugin to execute
    --command <cmd>     command to execute
    --list              view available plugins
    --build-dir <dir>   set the output build directory
    --verbose           display the command being executed
    -- args...          arguments to pass to the requested command

Use 'orbit help build' to learn more about the command.
";