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

use crate::core::context::Context;
use crate::core::ip::Mapping;
use crate::core::pkgid::PkgPart;
use crate::util::anyerror::Fault;
use std::collections::BTreeMap;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

use crate::commands::helps::search;
use crate::core::catalog::{Catalog, IpLevel, PkgName};
use crate::core::version::AnyVersion;

#[derive(Debug, PartialEq)]
pub struct Search {
    ip: Option<PkgPart>,
    cached: bool,
    downloaded: bool,
    available: bool,
    keywords: Vec<String>,
    limit: Option<usize>,
    hard_match: bool,
}

impl Subcommand<Context> for Search {
    fn interpret(cli: &mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(search::HELP))?;
        Ok(Search {
            downloaded: cli.check(Arg::flag("download").switch('d'))?,
            cached: cli.check(Arg::flag("install").switch('i'))?,
            available: cli.check(Arg::flag("available").switch('a'))?,
            hard_match: cli.check(Arg::flag("match"))?,
            limit: cli.get(Arg::option("limit").value("n"))?,
            keywords: cli
                .get_all(Arg::option("keyword").value("term"))?
                .unwrap_or(Vec::new()),
            ip: cli.get(Arg::positional("ip"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        let mut catalog = Catalog::new();
        // collect installed IP
        catalog = catalog.installations(c.get_cache_path())?;
        // collect downloaded IP
        catalog = catalog.downloads(c.get_downloads_path())?;
        // collect available IP
        catalog = catalog.available(&c.get_config().get_channels())?;

        self.run(&catalog)
    }
}

impl Search {
    fn run(&self, catalog: &Catalog) -> Result<(), Fault> {
        // transform into a BTreeMap for alphabetical ordering
        let mut tree = BTreeMap::new();

        let mut name_match_uuids = Vec::new();

        for (key, ids) in catalog.mappings() {
            match self.hard_match {
                true => {
                    match &self.ip {
                        // names must be identical
                        Some(pkgid) => {
                            if key == pkgid {
                                name_match_uuids.extend(ids);
                            } else {
                                ()
                            }
                        }
                        // move on to the keywords
                        None => name_match_uuids.extend(ids),
                    };
                }
                false => {
                    // pass everything if there is no filters applied
                    if self.ip.is_none() && self.keywords.is_empty() {
                        name_match_uuids.extend(ids)
                    } else {
                        if let Some(pkgid) = &self.ip {
                            if key.starts_with(pkgid) == true {
                                name_match_uuids.extend(ids);
                            }
                        }
                    }
                }
            }
        }

        let mut keyword_match_uuids = Vec::new();

        for (key, iplvl) in catalog.inner() {
            if let Some(prj) = iplvl.get(true, true, &AnyVersion::Latest) {
                match self.hard_match {
                    true => {
                        let mut all_match = true;
                        for kw in &self.keywords {
                            if prj.get_man().get_ip().get_keywords().contains(kw) == false {
                                all_match = false;
                                break;
                            }
                        }
                        if all_match == true {
                            keyword_match_uuids.push(key);
                        }
                    }
                    false => {
                        for kw in &self.keywords {
                            // only one keyword must be matching
                            if prj.get_man().get_ip().get_keywords().contains(kw) == true {
                                keyword_match_uuids.push(key);
                                break;
                            }
                        }
                    }
                }
            }
        }

        catalog
            .inner()
            .into_iter()
            .filter(|(id, _)| match self.hard_match {
                true => name_match_uuids.contains(id) && keyword_match_uuids.contains(id),
                false => name_match_uuids.contains(id) || keyword_match_uuids.contains(id),
            })
            .for_each(|(key, status)| {
                let name = status
                    .get(true, true, &AnyVersion::Latest)
                    .unwrap()
                    .get_man()
                    .get_ip()
                    .get_name();
                tree.insert(PkgName::new(name, Some(key)), status);
            });

        println!(
            "{}",
            Self::fmt_table(
                tree,
                self.limit,
                self.cached,
                self.downloaded,
                self.available
            )
        );
        Ok(())
    }

    fn fmt_table(
        table: BTreeMap<PkgName, &IpLevel>,
        limit: Option<usize>,
        cached: bool,
        downloaded: bool,
        available: bool,
    ) -> String {
        //         let header = format!(
        //             "\
        // {:<28}{:<10}{:<9}
        // {3:->28}{3:->10}{3:->11}\n",
        //             "Ip", "Latest", "Status", " "
        //         );
        let header = String::new();
        let mut body = String::new();
        let mut index = 0;

        let default = !(cached || downloaded || available);

        // note: There is definitely a nicer way to handle all of this logic... but this works for now.

        for (name, status) in table {
            // use this variable to determine if a level higher in the catalog has a higher version not displayed right now
            let mut is_update_available = false;
            // return the highest version (return installation when they are equal in downloads and cache)
            let ip = {
                let ins = match default || cached {
                    true => status.get_install(&AnyVersion::Latest),
                    false => None,
                };
                let dld = status.get_download(&AnyVersion::Latest);
                let ava = status.get_available(&AnyVersion::Latest);
                // prioritize who display
                if let Some(installed_ip) = ins {
                    // check if download or available have a later version
                    if let Some(download_ip) = dld {
                        if default || cached {
                            is_update_available = download_ip.get_man().get_ip().get_version()
                                > installed_ip.get_man().get_ip().get_version();
                        }
                    }
                    // if update is not coming from download, see if it comes from available
                    if is_update_available == false {
                        if let Some(avail_ip) = ava {
                            if default || cached || downloaded && dld.is_some() {
                                is_update_available = avail_ip.get_man().get_ip().get_version()
                                    > installed_ip.get_man().get_ip().get_version();
                            }
                        }
                    }

                    // determine who to display
                    match default || cached {
                        true => Some(installed_ip),
                        false => match downloaded {
                            true => dld,
                            false => ava,
                        },
                    }
                } else if let Some(download_ip) = dld {
                    if let Some(avail_ip) = ava {
                        if default || downloaded {
                            is_update_available = avail_ip.get_man().get_ip().get_version()
                                > download_ip.get_man().get_ip().get_version();
                        }
                    }
                    match default || downloaded {
                        true => Some(download_ip),
                        false => ava,
                    }
                } else {
                    ava
                }
            };
            // ip should NOT be empty but skip if it is
            let ip = match ip {
                Some(r) => r,
                None => continue,
            };

            if let Some(cap) = limit {
                index += 1;
                // exit when next entry will go past the max results
                if index > cap {
                    break;
                }
            }

            // determine if to skip this ip based on settings
            let display_to_screen = match ip.get_mapping() {
                Mapping::Physical => default == true || cached == true,
                Mapping::Virtual(_) => default == true || downloaded == true,
                Mapping::Imaginary => default == true || available == true,
                Mapping::Relative(_) => false,
            };
            if display_to_screen == false {
                continue;
            }

            body.push_str(&format!(
                "{:<24}{:<16}{:<10}{:<25}\n",
                name.get_name().to_string(),
                ip.get_man().get_ip().get_version().to_string() + {
                    if is_update_available == true {
                        "*"
                    } else {
                        ""
                    }
                },
                match ip.get_mapping() {
                    Mapping::Physical => "install",
                    Mapping::Virtual(_) => "download",
                    Mapping::Imaginary => "available",
                    Mapping::Relative(_) => "local",
                },
                name.get_uuid().unwrap().encode()
            ));
        }
        // remove final \n from body
        body.pop();
        header + &body
    }
}
