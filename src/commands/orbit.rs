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

use crate::commands::helps::orbit;
use crate::core::channel::Channel;
use crate::core::config;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;
use crate::util::environment;
use crate::util::prompt;
use crate::util::sha256::Sha256Hash;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Command, Subcommand};

use std::env;

pub type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, PartialEq)]
pub enum ColorMode {
    Always,
    Auto,
    Never,
}

impl Default for ColorMode {
    fn default() -> Self {
        Self::Auto
    }
}

impl ColorMode {
    pub fn sync(&self) {
        match self {
            Self::Always => colored::control::set_override(true),
            Self::Never => colored::control::set_override(false),
            Self::Auto => (),
        }
    }
}

impl FromStr for ColorMode {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            _ => Err(AnyError(format!(
                "value must be 'auto', 'always', or 'never'"
            ))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Orbit {
    upgrade: bool,
    version: bool,
    license: bool,
    force: bool,
    sync: bool,
    cmode: ColorMode,
    command: Option<OrbitSubcommand>,
}

impl Command for Orbit {
    fn interpret(cli: &mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(cliproc::Help::with(orbit::HELP))?;
        Ok(Orbit {
            upgrade: cli.check(Arg::flag("upgrade"))?,
            version: cli.check(Arg::flag("version"))?,
            license: cli.check(Arg::flag("license"))?,
            sync: cli.check(Arg::flag("sync"))?,
            force: cli.check(Arg::flag("force"))?,
            cmode: cli
                .get(Arg::option("color").value("when"))?
                .unwrap_or_default(),
            command: cli.nest(Arg::subcommand("command"))?,
        })
    }

    fn execute(self) -> proc::Result {
        // synchronize the coloring mode
        self.cmode.sync();
        // prioritize license information
        if self.license == true {
            println!("{}", DISCLAIMER);
            Ok(())
        // prioritize version information
        } else if self.version == true {
            let real_version = option_env!("GIT_DESC_VERSION").unwrap_or(VERSION);
            let disp_version = match real_version.len() {
                0 => format!("{} (build)", VERSION),
                _ => format!("{}", real_version),
            };
            println!("orbit {}", disp_version);
            Ok(())
        // prioritize upgrade information
        } else if self.upgrade == true {
            println!("info: checking for latest orbit binary...");
            let info = self.upgrade()?;
            println!("info: {}", info);
            Ok(())
        // run the specified command
        } else if let Some(sub) = self.command {
            // set up the context (ignores the context passed in)
            let context = Context::new()
                .home(environment::ORBIT_HOME)?
                .cache()?
                .archive()?
                .current_ip_dir(environment::ORBIT_MANIFEST_DIR)? // must come before .settings() call
                .settings(config::CONFIG_FILE)?
                .build_dir(environment::ORBIT_TARGET_DIR)?;
            // update channels
            if self.sync == true {
                Channel::sync(&context)?;
            }
            // pass the context to the given command
            sub.execute(&context)
        // if no command is given then print default help
        } else if self.sync == true {
            // update channels
            let context = Context::new()
                .home(environment::ORBIT_HOME)?
                .cache()?
                .archive()?
                .current_ip_dir(environment::ORBIT_MANIFEST_DIR)? // must come before .settings() call
                .settings(config::CONFIG_FILE)?
                .build_dir(environment::ORBIT_TARGET_DIR)?;
            Channel::sync(&context)?;
            Ok(())
        } else {
            Ok(println!("{}", orbit::HELP))
        }
    }
}

use crate::commands::build::Build;
use crate::commands::config::Config;
use crate::commands::env::Env;
use crate::commands::get::Get;
use crate::commands::help::Help;
use crate::commands::init::Init;
use crate::commands::install::Install;
use crate::commands::lock::Lock;
use crate::commands::new::New;
use crate::commands::publish::Publish;
use crate::commands::read::Read;
use crate::commands::remove::Remove;
use crate::commands::search::Search;
use crate::commands::test::Test;
use crate::commands::tree::Tree;
use crate::commands::view::View;

#[derive(Debug, PartialEq)]
enum OrbitSubcommand {
    Help(Help),
    New(New),
    Search(Search),
    Lock(Lock),
    Build(Build),
    Test(Test),
    Publish(Publish),
    Install(Install),
    Tree(Tree),
    Get(Get),
    Init(Init),
    View(View),
    Env(Env),
    Config(Config),
    Uninstall(Remove),
    Read(Read),
}

impl Subcommand<Context> for OrbitSubcommand {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        match cli
            .select(&[
                "help", "new", "search", "lock", "build", "test", "t", "publish", "install", "get",
                "init", "tree", "view", "b", "env", "config", "remove", "read",
            ])?
            .as_ref()
        {
            "get" => Ok(OrbitSubcommand::Get(Get::interpret(cli)?)),
            "help" => Ok(OrbitSubcommand::Help(Help::interpret(cli)?)),
            "new" => Ok(OrbitSubcommand::New(New::interpret(cli)?)),
            "search" => Ok(OrbitSubcommand::Search(Search::interpret(cli)?)),
            "lock" => Ok(OrbitSubcommand::Lock(Lock::interpret(cli)?)),
            "b" | "build" => Ok(OrbitSubcommand::Build(Build::interpret(cli)?)),
            "t" | "test" => Ok(OrbitSubcommand::Test(Test::interpret(cli)?)),
            "init" => Ok(OrbitSubcommand::Init(Init::interpret(cli)?)),
            "publish" => Ok(OrbitSubcommand::Publish(Publish::interpret(cli)?)),
            "install" => Ok(OrbitSubcommand::Install(Install::interpret(cli)?)),
            "tree" => Ok(OrbitSubcommand::Tree(Tree::interpret(cli)?)),
            "view" => Ok(OrbitSubcommand::View(View::interpret(cli)?)),
            "env" => Ok(OrbitSubcommand::Env(Env::interpret(cli)?)),
            "config" => Ok(OrbitSubcommand::Config(Config::interpret(cli)?)),
            "remove" => Ok(OrbitSubcommand::Uninstall(Remove::interpret(cli)?)),
            "read" => Ok(OrbitSubcommand::Read(Read::interpret(cli)?)),
            _ => panic!("an unimplemented command was passed through!"),
        }
    }

    fn execute(self, context: &Context) -> proc::Result {
        match self {
            OrbitSubcommand::Get(sub) => sub.execute(context),
            OrbitSubcommand::Search(sub) => sub.execute(context),
            OrbitSubcommand::Lock(sub) => sub.execute(context),
            OrbitSubcommand::Build(sub) => sub.execute(context),
            OrbitSubcommand::Install(sub) => sub.execute(context),
            OrbitSubcommand::Help(sub) => sub.execute(&()),
            OrbitSubcommand::New(sub) => sub.execute(context),
            OrbitSubcommand::Test(sub) => sub.execute(context),
            OrbitSubcommand::Publish(sub) => sub.execute(context),
            OrbitSubcommand::Tree(sub) => sub.execute(context),
            OrbitSubcommand::Init(sub) => sub.execute(context),
            OrbitSubcommand::View(sub) => sub.execute(context),
            OrbitSubcommand::Env(sub) => sub.execute(context),
            OrbitSubcommand::Config(sub) => sub.execute(context),
            OrbitSubcommand::Uninstall(sub) => sub.execute(context),
            OrbitSubcommand::Read(sub) => sub.execute(context),
        }
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

const DISCLAIMER: &str = r#"Copyright (C) 2022 - 2025  Chase Ruskin

This program is free software, covered by the GNU General Public License. There 
is NO warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE."#;

// TODO: check for additional data such as the commit being used

use crate::core::version::Version;
use crate::util::anyerror::Fault;
use crate::util::filesystem::get_exe_path;
use crate::util::sha256;
use curl::easy::{Easy, List};
use std::env::consts;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use tempfile;
use zip;
use zip::ZipArchive;

use serde_json::Value;

pub const RESPONSE_OKAY: u32 = 200;

impl Orbit {
    /// Returns current machine's target as `<arch>-<os>`.
    fn target_triple() -> String {
        format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
    }

    /// Runs a process to check for an updated version of Orbit on GitHub to install.
    ///
    /// Steps it follows:  
    /// 1. Removes any old version existing in executables' current folder
    /// 2. Gets website data from GitHub releases page to check for latest version
    /// 3. If new version, download checksum file and search for a compatible platform
    /// 4. Download compatible platform zip file and verify checksum matches
    /// 5. Unzip the file and replace the Orbit executable in-place.
    /// 6. Rename the old executable as `orbit-<version>`.
    fn upgrade(&self) -> Result<String, Fault> {
        // check for stale versions at the current executable's path
        let exe_path = get_exe_path()?;
        let mut current_exe_dir = exe_path.clone();
        current_exe_dir.pop();
        // find any old versions existing in executable's current folder
        let paths = fs::read_dir(&current_exe_dir)?;
        for path in paths {
            if path
                .as_ref()
                .unwrap()
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("orbit-")
            {
                // remove stale binaries
                fs::remove_file(path.as_ref().unwrap().path())?;
            }
        }

        // check the connection to grab latest html data
        let api_url: &str = "https://api.github.com/repos/chaseruskin/orbit/releases/latest";

        let mut dst = Vec::new();
        {
            let mut easy = Easy::new();
            easy.url(api_url).unwrap();
            easy.follow_location(false).unwrap();
            // create headers
            let mut list = List::new();
            list.append("User-Agent: Orbit").unwrap();
            easy.http_headers(list).unwrap();
            {
                let mut transfer = easy.transfer();
                transfer
                    .write_function(|data| {
                        dst.extend_from_slice(data);
                        Ok(data.len())
                    })
                    .unwrap();

                transfer.perform()?;
            }
            let rc = easy.response_code()?;
            if rc != RESPONSE_OKAY {
                return Err(Box::new(UpgradeError::FailedConnection(
                    api_url.to_owned(),
                    rc,
                )));
            }
        }
        let body: String = String::from_utf8(dst)?;

        // create body into string to find the latest version
        let version = {
            let json_word: Value = serde_json::from_str(body.as_ref())?;
            json_word["name"].as_str().unwrap().to_string()
        };

        // our current version is guaranteed to be valid
        let current = Version::from_str(VERSION).unwrap();
        // the latest version
        let latest = Version::from_str(&version).expect("invalid version released");
        if latest > current {
            // await user input
            if self.force == false {
                if prompt::prompt(&format!(
                    "info: a new version is available ({}), would you like to upgrade",
                    latest
                ))? == false
                {
                    return Ok(String::from("upgrade cancelled"));
                }
            }
        } else {
            return Ok(format!(
                "the latest version is already installed ({})",
                &latest
            ));
        }

        let base_url: &str = "https://github.com/chaseruskin/orbit/releases";

        // download the list of checksums
        println!("info: downloading update...");
        let sum_url = format!(
            "{0}/download/{1}/orbit-{1}-checksums.txt",
            &base_url, &latest
        );

        let mut dst = Vec::new();
        {
            let mut easy = Easy::new();
            easy.url(&sum_url).unwrap();
            easy.follow_location(true).unwrap();
            {
                let mut transfer = easy.transfer();
                transfer
                    .write_function(|data| {
                        dst.extend_from_slice(data);
                        Ok(data.len())
                    })
                    .unwrap();

                transfer.perform()?;
            }
            let rc = easy.response_code()?;
            if rc != RESPONSE_OKAY {
                return Err(Box::new(UpgradeError::FailedConnection(sum_url, rc)));
            }
        }
        let checksums: String = String::from_utf8(dst)?;

        // store user's target
        let target = Orbit::target_triple();

        let pkg = format!("orbit-{}-{}.zip", &latest, &target);
        // search the checksums to check if the desired pkg is available for download
        let cert = checksums.split_terminator('\n').find_map(|p| {
            let (cert, key) = p.split_once(' ').expect("bad checksum file format");
            if key == pkg {
                Some(Sha256Hash::from_str(cert).expect("bad checksum format"))
            } else {
                None
            }
        });
        // verify there was a certificate/checksum for the requested pkg
        let cert = match cert {
            Some(c) => c,
            None => return Err(Box::new(UpgradeError::UnsupportedTarget(target)))?,
        };

        // download the zip pkg file
        let pkg_url = format!("{}/download/{}/{}", &base_url, &latest, &pkg);
        // let res = reqwest::get(&pkg_url).await?;
        // if res.status() != 200 {
        //     return Err(Box::new(UpgradeError::FailedDownload(pkg_url.to_string(), res.status())))?
        // }
        // let body_bytes = res.bytes().await?;

        let mut body_bytes = Vec::new();
        {
            let mut easy = Easy::new();
            easy.url(&pkg_url).unwrap();
            easy.follow_location(true).unwrap();
            {
                let mut transfer = easy.transfer();
                transfer
                    .write_function(|data| {
                        body_bytes.extend_from_slice(data);
                        Ok(data.len())
                    })
                    .unwrap();

                transfer.perform()?;
            }
            let rc = easy.response_code()?;
            if rc != RESPONSE_OKAY {
                return Err(Box::new(UpgradeError::FailedConnection(pkg_url, rc)));
            }
        }

        // compute the checksum on the downloaded zip file
        let sum = sha256::compute_sha256(&body_bytes);
        // verify the checksums match
        match sum == cert {
            true => println!("info: verified download"),
            false => return Err(Box::new(UpgradeError::BadChecksum(sum, cert)))?,
        };

        // unzip the bytes and put file in temporary file
        println!("info: installing update...");
        let mut temp_file = tempfile::tempfile()?;
        temp_file.write_all(&body_bytes)?;
        let mut zip_archive = ZipArchive::new(temp_file)?;

        // decompress zip file to a temporary directory
        let temp_dir = tempfile::tempdir()?;
        zip_archive.extract(&temp_dir)?;

        let exe_ext = if consts::EXE_EXTENSION.is_empty() == true {
            ""
        } else {
            ".exe"
        };

        // verify the path to the new executable exists before renaming current binary
        let temp_exe_path = temp_dir.path().join(&format!(
            "orbit-{}-{}/bin/orbit{}",
            &latest, &target, &exe_ext
        ));
        if Path::exists(&temp_exe_path) == false {
            return Err(Box::new(UpgradeError::MissingExe))?;
        }

        // rename the current binary with its version to become a 'stale binary'
        let stale_exe_path = current_exe_dir.join(&format!("orbit-{}", VERSION));
        fs::rename(&exe_path, &stale_exe_path)?;

        // copy the executable from the temporary directory to the original location
        fs::copy(&temp_exe_path, &exe_path)?;

        Ok(String::from(format!(
            "successfully upgraded orbit to version {}",
            &latest
        )))
    }
}

#[derive(Debug, PartialEq)]
pub enum UpgradeError {
    UnsupportedTarget(String),
    FailedConnection(String, u32),
    FailedDownload(String, u32),
    NoReleasesFound,
    BadChecksum(Sha256Hash, Sha256Hash),
    MissingExe,
}

impl std::error::Error for UpgradeError {}

impl std::fmt::Display for UpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::MissingExe => write!(f, "failed to find the binary in the downloaded package"),
            Self::BadChecksum(computed, ideal) => write!(
                f,
                "checksums did not match, please try again\n\ncomputed: {}\nexpected: {}",
                computed, ideal
            ),
            Self::FailedConnection(url, status) => {
                write!(f, "connection failed\n\nurl: {}\nstatus: {}", url, status)
            }
            Self::FailedDownload(url, status) => {
                write!(f, "download failed\n\nurl: {}\nstatus: {}", url, status)
            }
            Self::UnsupportedTarget(t) => write!(
                f,
                "no pre-compiled binaries exist for the current target {}",
                t
            ),
            Self::NoReleasesFound => write!(f, "no releases were found"),
        }
    }
}
