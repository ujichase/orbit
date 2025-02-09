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

use super::helps::remove;
use crate::core::catalog::{CacheSlot, Catalog};
use crate::core::context::Context;
use crate::core::ip::{Ip, PartialIpSpec};
use crate::core::version::AnyVersion;
use crate::error::Error;
use crate::util::anyerror::{AnyError, Fault};
use crate::util::prompt;
use std::fs;
use std::path::PathBuf;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Remove {
    ip: PartialIpSpec,
    force: bool,
    // TODO: implement recursive removal (take away all dependent ips)
    // recurse: bool,
    verbose: bool,
    // TODO: add option to remove all versions (including archived)
    // TODO: add option to remove only from cache (skip archive)
}

impl Subcommand<Context> for Remove {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(remove::HELP))?;
        Ok(Remove {
            verbose: cli.check(Arg::flag("verbose"))?,
            force: cli.check(Arg::flag("force"))?,
            // recurse: cli.check(Arg::flag("recurse").switch('r'))?,
            ip: cli.require(Arg::positional("ip"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // collect the catalog from dev and installations
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?;

        // check for ip in development or installation
        let status = match catalog.translate_name(&self.ip.to_pkg_name())? {
            Some(st) => st,
            None => {
                return Err(AnyError(format!(
                    "ip \"{}\" does not exist in the catalog",
                    self.ip
                )))?
            }
        };

        // determine the ip version (invariant of state) that matches
        let detected_version = {
            let install_version = status.get_install(&self.ip.get_version());
            let download_version = status.get_download(&self.ip.get_version());

            if let Some(iv) = install_version {
                if let Some(dv) = download_version {
                    if iv.get_man().get_ip().get_version() > dv.get_man().get_ip().get_version() {
                        AnyVersion::Specific(
                            iv.get_man().get_ip().get_version().to_partial_version(),
                        )
                    } else {
                        AnyVersion::Specific(
                            dv.get_man().get_ip().get_version().to_partial_version(),
                        )
                    }
                } else {
                    AnyVersion::Specific(iv.get_man().get_ip().get_version().to_partial_version())
                }
            } else if let Some(dv) = download_version {
                AnyVersion::Specific(dv.get_man().get_ip().get_version().to_partial_version())
            } else {
                self.ip.get_version().clone()
            }
        };

        // check if in cache
        let cached_ip = status.get_install(&detected_version);
        let archived_ip = status.get_download(&detected_version);

        if cached_ip.is_none() && archived_ip.is_none() {
            return Err(Error::Custom(format!(
                "unable to find a version \"{1}\" for ip \"{0}\" that can be removed",
                self.ip.get_name(),
                detected_version
            )))?;
        }

        // get a complete name
        let ip_spec = match cached_ip {
            Some(c) => c.get_man().get_ip().into_ip_spec(),
            None => archived_ip.unwrap().get_man().get_ip().into_ip_spec(),
        };

        // TODO: issue a warning if the ip to be deleted is not found in a channel (this action may be
        // unrecoverable because there is no channel configured for this ip)

        // confirm with user that it is the correct ip
        if self.force == false {
            if prompt::prompt(&format!("info: removing ip {}, proceed", ip_spec))? == false {
                println!("info: {}", "removal cancelled");
                return Ok(());
            }
        }

        let selected_version = AnyVersion::Specific(ip_spec.get_version().to_partial_version());

        // grab the ip's manifest
        match status.get_install(&selected_version) {
            Some(t) => {
                Self::remove_install(t)?;
                if self.verbose == true {
                    println!("info: removed ip {} from the cache", ip_spec);
                }
                Self::remove_dynamics(c.get_cache_path(), t, self.verbose)?;
            }
            None => {
                if self.verbose == true {
                    println!("info: ip {} is already removed from the cache", self.ip);
                }
            }
        };

        // grab the ip's manifest
        match status.get_download(&selected_version) {
            Some(t) => {
                Self::remove_download(c.get_downloads_path(), t)?;
                if self.verbose == true {
                    println!("info: removed ip {} from the archive", ip_spec);
                }
            }
            None => {
                if self.verbose == true {
                    println!("info: ip {} is already removed from the archive", self.ip);
                }
            }
        };

        println!("info: removed ip {}", ip_spec);
        self.run()
    }
}

impl Remove {
    fn run(&self) -> Result<(), Fault> {
        Ok(())
    }

    /// Removes the compressed snapshot file of the ip from the archive.
    pub fn remove_download(archive_path: &PathBuf, target: &Ip) -> Result<(), Fault> {
        let ip_spec = target.get_man().get_ip().into_ip_spec();
        // delete the project from the cache (default behavior)
        fs::remove_file(
            archive_path.join(
                &target
                    .get_lock()
                    .get_self_entry(&ip_spec.get_name())
                    .unwrap()
                    .to_download_slot_key()
                    .as_ref(),
            ),
        )?;
        Ok(())
    }

    /// Removes the installed IP from its root directory. This function assumes
    /// the `target` IP exists under the installation path (cache path).
    pub fn remove_install(target: &Ip) -> Result<(), Fault> {
        // delete the project from the cache (default behavior)
        fs::remove_dir_all(target.get_root())?;
        Ok(())
    }

    pub fn remove_dynamics(cache_path: &PathBuf, target: &Ip, verbose: bool) -> Result<(), Fault> {
        let ip_spec = target.get_man().get_ip().into_ip_spec();

        let og_cache_slot = CacheSlot::new(
            target.get_uuid(),
            ip_spec.get_version(),
            target.get_checksum().as_ref().unwrap(),
        );

        // check for any "dynamics" under this target
        for dir in fs::read_dir(cache_path)? {
            // only check valid directory entries
            if let Ok(entry) = dir {
                // println!("{:?}", entry.file_name());
                let file_name = entry.file_name();

                if let Some(cache_slot) = CacheSlot::try_from_str(&file_name.to_string_lossy()) {
                    if og_cache_slot.is_child_slot(&cache_slot) == false {
                        continue;
                    }
                    // check for same UUID
                    let cached_ip = Ip::load(entry.path().to_path_buf(), false, false)?;
                    // remove the slot if it is dynamic
                    if cached_ip.is_dynamic() == true {
                        fs::remove_dir_all(entry.path())?;
                        if verbose == true {
                            println!(
                                "info: removed dynamic variant of ip {} from the cache",
                                ip_spec
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
