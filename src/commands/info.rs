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

use crate::commands::helps::info;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::{Ip, PartialIpSpec};
use crate::core::lang::LangUnit;
use crate::core::version;
use crate::core::visibility::Visibility;
use crate::error::{Error, Hint};
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use std::cmp::Ordering;
use std::env::current_dir;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Info {
    // TODO: narrow the displayed version list with a range?
    versions: bool,
    units: bool,
    ip: Option<PartialIpSpec>,
    all: bool,
    // TODO: view changelog?
    // TODO: view readme?
}

impl Subcommand<Context> for Info {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(info::HELP))?;
        Ok(Info {
            all: cli.check(Arg::flag("all").switch('a'))?,
            versions: cli.check(Arg::flag("versions").switch('v'))?,
            units: cli.check(Arg::flag("units").switch('u'))?,
            ip: cli.get(Arg::positional("ip"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // collect all manifests available (load catalog)
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?
            .available(&c.get_config().get_channels())?;

        let dev_ip: Option<Result<Ip, Fault>> = {
            match Context::find_ip_path(&current_dir().unwrap()) {
                Some(dir) => Some(Ip::load(dir, true, false)),
                None => None,
            }
        };

        let mut is_local_ip = false;

        // try to auto-determine the ip (check if in a working ip)
        let ip: &Ip = if let Some(spec) = &self.ip {
            // find the path to the provided ip by searching through the catalog
            if let Some(lvl) = catalog.translate_name(&spec.to_pkg_name())? {
                // return the highest available version
                if let Some(slot) = lvl.get_install(spec.get_version()) {
                    slot
                } else {
                    // try to find from downloads
                    if let Some(slot) = lvl.get_download(spec.get_version()) {
                        slot
                    } else {
                        if let Some(slot) = lvl.get_available(spec.get_version()) {
                            slot
                        } else {
                            return Err(Error::IpNotFoundInCache(spec.to_string()))?;
                        }
                    }
                }
            } else {
                return Err(Error::IpNotFoundAnywhere(
                    spec.to_string(),
                    Hint::CatalogList,
                ))?;
            }
        } else {
            if dev_ip.is_none() == true {
                return Err(Error::NoAssumedWorkingIpFound)?;
            } else {
                match &dev_ip {
                    Some(Ok(r)) => {
                        is_local_ip = true;
                        r
                    }
                    Some(Err(e)) => return Err(AnyError(format!("{}", e.to_string())))?,
                    _ => panic!("unreachable code"),
                }
            }
        };

        // load the ip's manifest
        if self.units == true {
            if ip.get_mapping().is_physical() == true {
                // force computing the primary design units if a physical ip (non-archived)
                let units = ip.collect_units(true, false)?;
                println!(
                    "{}",
                    Self::format_units_table(
                        units.into_iter().map(|(_, unit)| unit).collect(),
                        self.all,
                        is_local_ip,
                    )
                );
            } else {
                // a 'virtual' ip, so try to extract units from
                println!(
                    "info: {}",
                    "unable to display HDL units from a downloaded ip; try again after installing"
                );
            }

            return Ok(());
        }

        // display all installed versions in the cache
        if self.versions == true {
            let specified_ver = if let Some(spec) = self.ip.as_ref() {
                spec.get_version().as_specific()
            } else {
                None
            };

            return match catalog.get_possible_versions(ip.get_uuid()) {
                Some(vers) => {
                    match vers.len() {
                        0 => {
                            crate::info!("no versions in the cache")
                        }
                        _ => {
                            let mut data = String::new();
                            // further restrict versions if a particular version is set
                            vers.iter()
                                .filter(move |p| {
                                    specified_ver.is_none()
                                        || version::is_compatible(
                                            specified_ver.unwrap(),
                                            &p.get_version(),
                                        ) == true
                                })
                                .for_each(|v| {
                                    data.push_str(&format!(
                                        "{:<14}{:<9}\n",
                                        v.get_version().to_string(),
                                        v.get_state().to_string()
                                    ));
                                });
                            // pop the last \n
                            data.pop();
                            println!("{}", data);
                        }
                    }
                    Ok(())
                }
                None => Err(AnyError(format!("no ip found in catalog")))?,
            };
        }

        // print the manifest data "pretty"
        let s = toml::to_string_pretty(ip.get_man())?;
        println!("{}", s);
        Ok(())
    }
}

impl Info {
    /// Creates a string for to display the primary design units for the particular ip.
    fn format_units_table(table: Vec<LangUnit>, all: bool, is_local_ip: bool) -> String {
        let mut result = String::new();
        let mut table = table;

        table.sort_by(|a, b| match a.get_visibility().cmp(&b.get_visibility()) {
            Ordering::Equal => a.get_name().cmp(&b.get_name()),
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        });

        for unit in table {
            // skip this unit if it is not listed public and all is not provided
            if is_local_ip == false && all == false && unit.get_visibility() != &Visibility::Public
            {
                continue;
            }
            result.push_str(&format!(
                "{:<40}{:<15}{:<9}\n",
                unit.get_name().to_string(),
                unit.to_string(),
                unit.get_visibility().to_string(),
            ));
        }
        // pop the last \n
        result.pop();
        result
    }
}
