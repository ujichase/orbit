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

use crate::core::catalog::Catalog;
use crate::core::catalog::DownloadSlot;
use crate::core::ip::Ip;
use crate::core::ip::IpSpec;
use crate::core::ip::PartialIpSpec;
use crate::core::iparchive::IpArchive;
use crate::core::lockfile::LockEntry;
use crate::core::lockfile::LockFile;
use crate::core::manifest;
use crate::core::manifest::IP_MANIFEST_FILE;
use crate::core::protocol::Protocol;
use crate::core::source::Source;
use crate::core::swap::StrSwapTable;
use crate::core::target::Process;
use crate::error::Error;
use crate::error::Hint;
use crate::error::LastError;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use crate::util::filesystem::Standardize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub type ProtocolMap<'a> = HashMap<&'a str, &'a Protocol>;

#[derive(Debug, PartialEq)]
pub struct Download {
    all: bool,
    missing: bool,
    list: bool,
    queue_dir: Option<PathBuf>,
    verbose: bool,
    force: bool,
    // IDEA: add <url> argument to download? with --protocol <alias> option?
}

impl Download {
    /// Generates a list of dependencies required to be downloaded from the internet.
    ///
    /// Enabling `missing_only` will only push sources for ip not already installed.
    pub fn compile_download_list<'a>(
        le: &LockEntry,
        lf: &'a LockFile,
        catalog: &Catalog,
        missing_only: bool,
    ) -> Vec<(IpSpec, Source)> {
        let mut vtable = StrSwapTable::new();
        lf.inner()
            .iter()
            .filter(|p| p.get_source().is_some() == true)
            .filter(|p| {
                p.matches_target(&le, &catalog) == false
                    && (missing_only == false
                        || catalog.is_downloaded_slot(&p.to_download_slot_key()) == false)
            })
            .map(|f| {
                let spec = f.to_ip_spec();
                vtable.add("orbit.ip.name", spec.get_name().as_ref());
                vtable.add("orbit.ip.version", &spec.get_version().to_string());
                let processed_src = f.get_source().unwrap().clone().replace_vars_in_url(&vtable);
                (spec, processed_src)
            })
            .collect()
    }

    /// Calls a protocol for the given package and then places the download into
    /// the downloads folder.
    pub fn download(
        vtable: &mut StrSwapTable,
        spec: Option<&PartialIpSpec>,
        src: &Source,
        queue: Option<&PathBuf>,
        download_dir: &PathBuf,
        protocols: &HashMap<&str, &Protocol>,
        verbose: bool,
        _force: bool,
    ) -> Result<(IpSpec, Vec<u8>), Fault> {
        // use the user-provided queue directory or simply use a temporary directory
        let queue = match queue {
            Some(q) => {
                std::fs::create_dir_all(q)?;
                q.clone()
            }
            None => TempDir::into_path(TempDir::new()?),
        };

        // access the protocol
        if let Some(proto) = src.get_protocol() {
            match protocols.get(proto.as_str()) {
                Some(&entry) => {
                    if let Some(ip_spec) = spec {
                        if verbose == true {
                            println!(
                                "info: downloading ip {} over \"{}\" protocol ...",
                                ip_spec, &proto
                            );
                        }
                        // update variable table for this lock entry
                        vtable.add("orbit.ip.name", ip_spec.get_name().as_ref());
                        vtable.add("orbit.ip.version", &ip_spec.get_version().to_string());
                    } else {
                        if verbose == true {
                            crate::info!("downloading ip over \"{}\" protocol ...", &proto);
                        }
                    }

                    // perform string swap on source url
                    let processed_src = src
                        .clone()
                        .replace_vars_in_url(&vtable)
                        .replace_vars_in_tag(&vtable);

                    let std_queue = PathBuf::standardize(&queue);
                    vtable.add("orbit.queue", std_queue.to_str().unwrap());
                    vtable.add("orbit.ip.source.url", processed_src.get_url());
                    vtable.add("orbit.ip.source.protocol", entry.get_name());
                    vtable.add(
                        "orbit.ip.source.tag",
                        src.get_tag().as_ref().unwrap_or(&String::new()),
                    );
                    // allow the user to handle placing the code in the queue
                    let entry: Protocol = entry.clone().replace_vars_in_args(&vtable);
                    if let Err(err) = entry.execute(&None, &[], verbose, &std_queue, HashMap::new())
                    {
                        fs::remove_dir_all(queue)?;
                        return Err(Error::ProtocolProcFailed(LastError(err.to_string())))?;
                    }
                }
                None => {
                    // potential to use --force here to avoid this error and try with default but not currently implemented that way
                    fs::remove_dir_all(queue)?;
                    return Err(Error::ProtocolNotFound(proto.to_string()))?;
                }
            }
        }
        // try to use default protocol
        if src.is_default() == true {
            if let Some(ip_spec) = spec {
                vtable.add("orbit.ip.name", ip_spec.get_name().as_ref());
                vtable.add("orbit.ip.version", &ip_spec.get_version().to_string());
                if verbose == true {
                    crate::info!("downloading ip {} ...", ip_spec);
                }
            } else {
                if verbose == true {
                    crate::info!("downloading ip ...");
                }
            }

            // perform string swap on source url
            let processed_src = src
                .clone()
                .replace_vars_in_url(&vtable)
                .replace_vars_in_tag(&vtable);

            if let Err(err) = Protocol::single_download(processed_src.get_url(), &queue) {
                fs::remove_dir_all(queue)?;
                return Err(err);
            }
        }
        // move the IP to the downloads folder
        match Self::move_to_download_dir(&queue, download_dir, spec, verbose) {
            Ok((name, bytes)) => {
                // clean up temporary directory
                fs::remove_dir_all(queue)?;
                Ok((name, bytes))
            }
            Err(e) => {
                fs::remove_dir_all(queue)?;
                Err(e)
            }
        }
    }

    pub fn move_to_download_dir(
        queue: &PathBuf,
        downloads: &PathBuf,
        spec: Option<&PartialIpSpec>,
        verbose: bool,
    ) -> Result<(IpSpec, Vec<u8>), Fault> {
        // code is in the queue now, move it to the downloads/ folder

        let entries = manifest::find_file(&queue, IP_MANIFEST_FILE, false)?;

        let mut matching_ips = Vec::new();

        // find the IP (make sure there is only 1!)
        for entry in entries {
            // println!("{:?}", entry);
            // check if this is our IP
            match Ip::load(entry.parent().unwrap().to_path_buf(), true, false) {
                Ok(temp) => {
                    // println!("{}", temp.get_man().get_ip().into_ip_spec());
                    let manifest_version =
                        temp.get_man().get_ip().get_version().to_partial_version();

                    // move to downloads only if we match the name
                    if let Some(prov) = spec {
                        let mut is_match = true;
                        // make sure the uuid's match (if available)
                        if let Some(uuid) = prov.as_uuid() {
                            if uuid != temp.get_man().get_ip().get_uuid() {
                                is_match = false;
                            }
                        }
                        // make sure the names match
                        if prov.get_name() != temp.get_man().get_ip().get_name() {
                            is_match = false;
                        }
                        // make sure the version falls under right domain
                        if manifest_version.in_domain(
                            prov.get_version()
                                .as_specific()
                                .unwrap_or(&manifest_version),
                        ) == false
                        {
                            is_match = false;
                        }

                        if is_match == true {
                            matching_ips.push(temp);
                        }
                    } else {
                        // we take all manifests if no name was given
                        matching_ips.push(temp);
                    };
                }
                Err(_) => {}
            }
        }

        match matching_ips.len() {
            0 => {
                // could not find the IP
                Err(AnyError(format!(
                    "failed to find a manifest for ip \"{}\"",
                    spec.unwrap()
                )))?
            }
            1 => {
                let temp = matching_ips.get(0).unwrap();
                let manifest_name = temp.get_man().get_ip().get_name();
                let found_ip_spec = temp.get_man().get_ip().into_ip_spec();
                if verbose == true {
                    crate::info!("found ip {}", found_ip_spec);
                }
                // verify the ip is okay
                Ip::load(temp.get_root().to_path_buf(), false, false)?;
                // zip the project to the downloads directory
                let download_slot_name = DownloadSlot::new(
                    manifest_name,
                    temp.get_uuid(),
                    temp.get_man().get_ip().get_version(),
                );
                let full_download_path = downloads.join(&download_slot_name.as_ref());
                let bytes = IpArchive::write(&temp, &full_download_path)?;
                return Ok((found_ip_spec, bytes));
            }
            _ => {
                let mut candidate_list = String::new();
                matching_ips.iter().for_each(|i| {
                    candidate_list
                        .push_str(&format!("\n    {:?}", i.get_man().get_ip().into_ip_spec()));
                });
                Err(Box::new(Error::DownloadFoundManyIps(
                    matching_ips.len(),
                    candidate_list,
                    Hint::SpecifyIpSpecForDownload,
                )))
            }
        }
    }

    pub fn download_all(
        downloads: &Vec<(IpSpec, Source)>,
        proto_map: &HashMap<&str, &Protocol>,
        vtable: StrSwapTable,
        verbose: bool,
        queue: Option<&PathBuf>,
        download_dir: &PathBuf,
        force: bool,
    ) -> Result<(), Fault> {
        match downloads.len() {
            0 => {
                crate::info!("no missing downloads");
                return Ok(());
            }
            1 => {
                crate::info!("downloading 1 ip ...")
            }
            _ => {
                crate::info!("downloading {} ips ...", downloads.len())
            }
        }
        let mut vtable = vtable;
        let mut results = downloads.iter().filter_map(|e| {
            match Self::download(
                &mut vtable,
                Some(&e.0.to_partial_ip_spec()),
                &e.1,
                queue,
                &download_dir,
                &proto_map,
                verbose,
                force,
            ) {
                Ok(_) => None,
                Err(e) => Some(e),
            }
        });
        if let Some(n) = results.next() {
            return Err(n);
        }

        Ok(())
    }
}

// impl Download {
// DEPRECATED: This function may be outdated- was used when `plan` used to be a
// dedicated subcommand.

// fn execute(self, c: &Context) -> Result<(), Fault> {
// // @idea: display lock entries as JSON? or use different env var for ORBIT_DOWNLOAD_LIST and ORBIT_VERSION_LIST

// // cannot happen
// if self.all == true && self.missing == true {
//     panic!("cannot display all and missing lock entries");
// }

// if let Some(dir) = &self.queue_dir {
//     if dir.exists() == true {
//         panic!("queue directory must be a non-existent directory");
//     }
// }

// let proto_map: ProtocolMap = c.get_config().get_protocols();

// // load the catalog (ignore errors because we are only downloading)
// let catalog = match self.force {
//     true => Catalog::new().set_downloads_path(c.get_downloads_path())?,
//     false => Catalog::new().downloads(c.get_downloads_path())?,
// };

// // verify running from an IP directory and enter IP's root directory
// c.jump_to_working_ip()?;

// let ip = Ip::load(c.get_ip_path().unwrap().clone(), true)?;

// // verify a lockfile exists
// if ip.get_lock().is_empty() == true {
//     panic!("cannot download due to missing lockfile")
// }

// let env = Environment::new()
//     // read config.toml for setting any env variables
//     .from_config(c.get_config())?
//     // read ip manifest for env variables
//     .from_ip(&Ip::load(c.get_ip_path().unwrap().clone(), true)?)?;

// let vtable = StrSwapTable::new().load_environment(&env)?;
// env.initialize();

// // default behavior is report only missing installations
// let missing_only = self.force == false || self.missing == true;

// // default behavior is to print out to console
// let to_stdout = self.list == true;

// // determine whether to filter out or keep the dev dependencies from the lock file
// let lf = ip.get_lock().keep_dev_dep_entries(&ip, self.all);

// let downloads =
//     Self::compile_download_list(&LockEntry::from((&ip, true)), &lf, &catalog, missing_only);
// // print to console
// if to_stdout == true {
//     downloads.iter().for_each(|(_, src)| println!("{}", src));
// // execute the command
// } else {
//     Self::download_all(
//         &downloads,
//         &proto_map,
//         vtable,
//         self.verbose,
//         self.queue_dir.as_ref(),
//         c.get_downloads_path(),
//         self.force,
//     )?;
// }
// Ok(())
// }
// }
