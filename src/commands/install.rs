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

//! The installation process:
//! 1. Optionally ask for an ip to install (default: current working ip)
//! --!-- Get the folder and change directories to the folder requiring installation --!--
//! * at the point in the process when the program is at the folder, it is assumed all sub-deps are also already installed
//! 2. Write results from computing the available units for the package
//! 3. Write results for information about accessing C,I,S,A
//! 3. Verify a .lock file is available (is this needed? - do dependents even read from this?)
//! 4. Move only relevant files to artifact directory (no .git/, etc.)
//! 5. Compute checksum on entire directory
//! 6. Zip contents and store in "store" for future re-installation
//! 7. Place artifact directory in "cache" for catalog lookup
//!
//! One issue that remains is how to retrieve packages from online automatically.
//!
//! The download process:
//!     - write a lockfile
//!     - ...
//!

use super::plan::Plan;
use super::publish::Publish;
use crate::commands::download::Download;
use crate::commands::download::ProtocolMap;
use crate::commands::helps::install;
use crate::commands::plan;
use crate::commands::remove::Remove;
use crate::core::algo;
use crate::core::catalog::CacheSlot;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::ip::IpSpec;
use crate::core::ip::PartialIpSpec;
use crate::core::iparchive::IpArchive;
use crate::core::lockfile::LockEntry;
use crate::core::manifest::IP_MANIFEST_FILE;
use crate::core::protocol::Protocol;
use crate::core::protocol::ProtocolError;
use crate::core::source::Source;
use crate::core::swap::StrSwapTable;
use crate::core::version;
use crate::core::version::AnyVersion;
use crate::error::Error;
use crate::error::Hint;
use crate::error::LastError;
use crate::util::anyerror::Fault;
use crate::util::environment::Environment;
use crate::util::filesystem;
use crate::util::filesystem::Standardize;
use std::env;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use zip::ZipArchive;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Install {
    ip: Option<PartialIpSpec>,
    url: Option<String>,
    path: Option<PathBuf>,
    protocol: Option<String>,
    offline: bool,
    tag: Option<String>,
    list: bool,
    force: bool,
    verbose: bool,
    all: bool,
}

impl Subcommand<Context> for Install {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(install::HELP))?;
        Ok(Install {
            // Flags
            force: cli.check(Arg::flag("force"))?,
            verbose: cli.check(Arg::flag("verbose"))?,
            all: cli.check(Arg::flag("all"))?,
            list: cli.check(Arg::flag("list"))?,
            offline: cli.check(Arg::flag("offline"))?,
            // Options
            path: cli.get(Arg::option("path"))?,
            url: cli.get(Arg::option("url"))?,
            tag: cli.get(Arg::option("tag"))?,
            protocol: cli.get(Arg::option("protocol").value("name"))?,
            // Positionals
            ip: cli.get(Arg::positional("ip"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // locate the plugin
        let protocol = match &self.protocol {
            // verify the plugin alias matches
            Some(name) => match c.get_config().get_protocols().get(name.as_str()) {
                Some(&p) => Some(p),
                None => return Err(ProtocolError::Missing(name.to_string()))?,
            },
            None => None,
        };

        // display protocol list and exit
        if self.list == true {
            match protocol {
                // display entire contents about the particular plugin
                Some(proto) => println!("{}", proto),
                // display quick overview of all plugins
                None => println!(
                    "{}",
                    Protocol::list_protocols(
                        &mut c
                            .get_config()
                            .get_protocols()
                            .values()
                            .into_iter()
                            .collect::<Vec<&&Protocol>>()
                    )
                ),
            }
            return Ok(());
        }

        // gather the catalog (all manifests)
        let mut catalog = if self.force == true {
            // do not look at the installations
            Catalog::new()
                .set_cache_path(c.get_cache_path())?
                .downloads(c.get_downloads_path())?
                .available(&c.get_config().get_channels())?
        } else {
            Catalog::new()
                .installations(c.get_cache_path())?
                .downloads(c.get_downloads_path())?
                .available(&c.get_config().get_channels())?
        };

        let mut provided_spec = None;

        // check if trying to download from the internet
        let target = if let Some(link) = &self.url {
            provided_spec = Some(
                Self::download_target_from_url(
                    c,
                    &link,
                    &self.protocol,
                    &self.tag,
                    &self.ip,
                    true,
                    self.force,
                )?
                .0
                .to_partial_ip_spec(),
            );
            None
        // check if trying to download from local filesystem
        } else if self.path.is_some() || self.ip.is_none() {
            // verify the path points to a valid ip
            let search_path = filesystem::resolve_rel_path(
                &env::current_dir()?,
                &filesystem::into_std_str(
                    self.path.as_ref().unwrap_or(&PathBuf::from(".")).clone(),
                ),
            );

            // check if specifying an ip
            let search_dir = PathBuf::standardize(PathBuf::from(search_path));
            // check if it is a zip file and get the directory to the unzipped contents if so
            let temp_dir_for_zip = if let Some(ext) = search_dir.extension() {
                if search_dir.is_file() == true && ext.eq_ignore_ascii_case("zip") {
                    // decompress zip file to a temporary directory
                    let temp_dir = tempfile::tempdir()?;
                    let zip_file = File::open(&search_dir)?;
                    let mut zip_archive = ZipArchive::new(zip_file)?;
                    zip_archive.extract(&temp_dir)?;
                    Some(temp_dir.into_path())
                } else {
                    None
                }
            } else {
                None
            };

            let search_dir = match &temp_dir_for_zip {
                Some(p) => p,
                None => &search_dir,
            };

            let search_path = search_dir.join(IP_MANIFEST_FILE);

            let target = match &self.ip {
                Some(entry) => match search_path.exists() {
                    true => {
                        let ip = Ip::load(search_dir.to_path_buf(), true, false)?;
                        if ip.get_man().get_ip().get_name() == entry.get_name()
                            && (entry.get_version().is_latest()
                                || version::is_compatible(
                                    entry.get_version().as_specific().unwrap(),
                                    ip.get_man().get_ip().get_version(),
                                ))
                        {
                            ip
                        } else {
                            if temp_dir_for_zip.is_none() {
                                Err(Error::Custom(format!(
                                    "could not find ip \"{}\" at path \"{}\"",
                                    entry,
                                    filesystem::into_std_str(search_dir.to_path_buf())
                                )))?
                            } else {
                                std::fs::remove_dir_all(temp_dir_for_zip.clone().unwrap())?;
                                Err(Error::Custom(format!(
                                    "could not find ip \"{}\" in archive \"{}\"",
                                    entry,
                                    filesystem::into_std_str(search_dir.to_path_buf())
                                )))?
                            }
                        }
                    }
                    false => {
                        if temp_dir_for_zip.is_none() {
                            Err(Error::Custom(format!(
                                "path \"{}\" does not contain an Orbit.toml file",
                                filesystem::into_std_str(search_dir.to_path_buf())
                            )))?
                        } else {
                            std::fs::remove_dir_all(temp_dir_for_zip.clone().unwrap())?;
                            Err(Error::Custom(format!(
                                "archive \"{}\" does not contain an Orbit.toml file",
                                filesystem::into_std_str(search_dir.to_path_buf())
                            )))?
                        }
                    }
                },
                // make sure there is only 1 ip to load
                None => match search_path.exists() {
                    true => Ip::load(search_dir.to_path_buf(), true, false)?,
                    false => {
                        if temp_dir_for_zip.is_none() {
                            Err(Error::Custom(format!(
                                "path \"{}\" does not contain an Orbit.toml file",
                                filesystem::into_std_str(search_dir.to_path_buf())
                            )))?
                        } else {
                            std::fs::remove_dir_all(temp_dir_for_zip.clone().unwrap())?;
                            Err(Error::Custom(format!(
                                "archive \"{}\" does not contain an Orbit.toml file",
                                filesystem::into_std_str(search_dir.to_path_buf())
                            )))?
                        }
                    }
                },
            };
            // move the ip to the downloads folder if not already there
            let (spec, _) = Download::move_to_download_dir(
                &target.get_root(),
                c.get_downloads_path(),
                Some(
                    &target
                        .get_man()
                        .get_ip()
                        .into_ip_spec()
                        .to_partial_ip_spec(),
                ),
                true,
            )?;
            provided_spec = Some(spec.to_partial_ip_spec());
            Some(target)
        // attempt to find the catalog
        } else {
            None
        };

        let determined_spec = match &provided_spec {
            Some(p) => Some(p),
            None => self.ip.as_ref(),
        };

        // update the downloads
        catalog = catalog.downloads(c.get_downloads_path())?;

        // use the catalog (if no path is provided)
        let target = if self.path.is_none() == true && (self.url.is_some() || self.ip.is_some()) {
            if let Some(spec) = &determined_spec {
                if let Some(lvl) = catalog.translate_name(&spec.to_pkg_name())? {
                    if let Some(slot) = lvl.get(true, true, spec.get_version()) {
                        // extract as download
                        if let Some(bytes) = slot.get_mapping().as_bytes() {
                            // println!("{} {}", "using archive", slot.get_man().get_ip().into_ip_spec());
                            // place the dependency into a temporary directory
                            // @MARK: fix this to cleanup manually since we forced it into_path.
                            let dir = tempfile::tempdir()?.into_path();
                            if let Err(e) = IpArchive::extract(&bytes, &dir) {
                                fs::remove_dir_all(dir)?;
                                return Err(e);
                            }
                            // load the IP
                            let unzipped_ip = match Ip::load(dir.clone(), false, false) {
                                Ok(x) => x,
                                Err(e) => {
                                    fs::remove_dir_all(dir)?;
                                    return Err(e);
                                }
                            };
                            Some(unzipped_ip)
                        // follow pointer to download an archive
                        } else if slot.get_mapping().is_pointer() {
                            // println!("{}", "using pointer");
                            match slot.get_man().get_ip().get_source() {
                                Some(sour) => Some(self.download_target_from_source(
                                    c,
                                    sour,
                                    slot.get_man().get_ip().into_ip_spec(),
                                )?),
                                None => {
                                    return Err(Error::Custom(format!(
                                        "ip requires source to download"
                                    )))?
                                }
                            }
                        // use the physical/local location of the ip? (does this ever occur?)
                        } else {
                            Some(Ip::load(slot.get_root().clone(), false, false)?)
                        }
                    } else {
                        return Err(Error::Custom(format!(
                            "ip {} does not exist in the catalog",
                            spec
                        )))?;
                    }
                } else {
                    return Err(Error::Custom(format!(
                        "failed to find an ip in the catalog"
                    )))?;
                }
            // use the local IP if the ip spec was not provided
            } else {
                target
            }
        // use the local IP if a path was supplied
        } else {
            target
        };
        // println!("{:?},", target);
        let target = match target {
            Some(t) => t,
            None => return Err(Error::Custom(format!("failed to find an ip to install")))?,
        };

        // println!("{:?}", target.get_uuid());

        // verify the ip is not already taken in the cache
        if let Some(ip_levels) =
            catalog.translate_name(&target.get_man().get_ip().into_ip_spec().to_pkg_name())?
        {
            if let Some(cached_ip) = ip_levels.get_install(&AnyVersion::Specific(
                target.get_man().get_ip().get_version().to_partial_version(),
            )) {
                let cached_version = cached_ip.get_man().get_ip().get_version();
                let target_version = target.get_man().get_ip().get_version();

                // compare uuids and versions
                if cached_ip.get_uuid() == target.get_uuid() && cached_version == target_version {
                    // upon force, remove the installations
                    if self.force == true {
                        Remove::remove_install(&cached_ip)?;
                        Remove::remove_dynamics(c.get_cache_path(), &cached_ip, false)?;
                    // tell the user we already have it installed!
                    } else {
                        crate::info!(
                            "ip {} is already installed",
                            target.get_man().get_ip().into_ip_spec()
                        );
                        return Ok(());
                    }
                }
            }
        }

        // now load the installations if previously not loaded
        if self.force == true {
            catalog = catalog.installations(c.get_cache_path())?;
        }

        // perform a series of checks on this ip
        catalog = Self::run_ip_checkpoints(&target, catalog, self.force, &c, self.all)?;

        // add additional check if we can download from online and it matches
        if (self.path.is_some() || self.ip.is_none())
            && target.get_man().get_ip().get_source().is_some()
            && self.offline == false
        {
            crate::info!("{}", "verifying coherency with ip's source  ...");
            let changes = Publish::test_download_and_install(&target, c, false, false)?;
            // remove from install so that we can install again
            if let Some(chg) = changes {
                Remove::remove_install(&chg.cached_ip)?;
            }
        }

        // @MARK: check for when there are multiple uuids that could potentially be for this ip

        // this code is only ran if the lock file matches the manifest and we aren't force to recompute
        // if target.can_use_lock() == true && self.force == false {
        //     crate::info!("{}", "reading dependencies from lockfile ...");
        //     let env = Environment::new()
        //         .from_config(c.get_config())?
        //         .from_ip(&target)?;

        //     let vtable = StrSwapTable::new().load_environment(&env)?;

        //     let le = LockEntry::from((&target, true));

        //     let lf = target.get_lock().keep_dev_dep_entries(&target, self.all);

        //     // verify the ip has no relative ip's listed in manifest
        //     if target.get_man().has_relative_deps() == true {
        //         return Err(Error::IpHasRelativeDependencies)?;
        //     }

        //     plan::download_missing_deps(
        //         vtable,
        //         &lf,
        //         &le,
        //         &catalog,
        //         &c.get_config().get_protocols(),
        //     )?;
        //     // recollect the queued items to update the catalog
        //     catalog = catalog.downloads(c.get_downloads_path())?;

        //     plan::install_missing_deps(&lf, &le, &catalog)?;
        //     // recollect the installations and queued items to update the catalog
        //     catalog = catalog.installations(c.get_cache_path())?;
        // }

        // // @MARK: may be an issue and should error if trying to install with an out-of-date lockfile
        // // generate lock file if it is missing or out of date
        // if target.lock_exists() == false || target.can_use_lock() == false {
        //     // build entire ip graph and resolve with dynamic symbol transformation
        //     let ip_graph = algo::compute_final_ip_graph(&target, &catalog, &c.get_languages())?;
        //     Plan::write_lockfile(&target, &ip_graph, true)?;
        // }

        // install the top-level target
        self.run(&target, &catalog)
    }
}

impl Install {
    fn run_ip_checkpoints<'c>(
        local_ip: &Ip,
        catalog: Catalog<'c>,
        force: bool,
        c: &'c Context,
        all: bool,
    ) -> Result<Catalog<'c>, Fault> {
        let mut catalog = catalog;

        // verify the lock file is generated and up to date
        if force == false {
            crate::info!("{}", "verifying lockfile is up to date ...");
            // TODO: use catalog to find the uuids of all dependencies to fill in
            if local_ip.can_use_lock(&catalog) == false {
                return Err(Box::new(Error::PublishMissingLockfile(Hint::MakeLock)));
            }
        // create the lockfile
        } else if local_ip.can_use_lock(&catalog) == false {
            let ip_graph = algo::compute_final_ip_graph(&local_ip, &catalog)?;
            Plan::write_lockfile(&local_ip, &ip_graph, true, true, &catalog)?;
        }

        crate::info!("{}", "reading dependencies from lockfile ...");
        let env = Environment::new()
            .from_config(c.get_config())?
            .from_ip(&local_ip)?;

        let vtable = StrSwapTable::new().load_environment(&env)?;

        let le = LockEntry::from((local_ip, true));

        let lf = local_ip.get_lock().keep_dev_dep_entries(&local_ip, all);

        plan::download_missing_deps(vtable, &lf, &le, &catalog, &c.get_config().get_protocols())?;

        // recollect the queued items to update the catalog
        catalog = catalog.downloads(c.get_downloads_path())?;

        plan::install_missing_deps(&lf, &le, &catalog)?;
        // recollect the installations and queued items to update the catalog
        catalog = catalog.installations(c.get_cache_path())?;

        // verify the ip has zero relative dependencies
        crate::info!("{}", "verifying all dependencies are stable ...");
        if let Some(dep) = local_ip.get_lock().inner().iter().find(|f| f.is_relative()) {
            return Err(Box::new(Error::PublishRelativeDepExists(
                dep.get_name().clone(),
            )));
        }

        // verify the graph build with no errors
        crate::info!("verifying hardware graph construction ...");
        if let Err(e) = Publish::check_graph_builds_okay(&local_ip, &catalog) {
            return Err(Box::new(Error::PublishHdlGraphFailed(LastError(
                e.to_string(),
            ))))?;
        }

        Ok(catalog)
    }

    pub fn download_target_from_url(
        c: &Context,
        url: &str,
        protocol: &Option<String>,
        tag: &Option<String>,
        ip: &Option<PartialIpSpec>,
        verbose: bool,
        force: bool,
    ) -> Result<(IpSpec, Vec<u8>), Fault> {
        let env = Environment::new().from_config(c.get_config())?;
        let mut vtable = StrSwapTable::new().load_environment(&env)?;
        env.initialize();

        let protocols: ProtocolMap = c.get_config().get_protocols();

        let target_source = Source::new()
            .url(url.to_string())
            .protocol(protocol.clone())
            .tag(tag.clone());

        // fetch from the internet
        let (name, bytes) = Download::download(
            &mut vtable,
            ip.as_ref(),
            &target_source,
            None,
            c.get_downloads_path(),
            &protocols,
            verbose,
            force,
        )?;
        Ok((name, bytes))
    }

    fn download_target_from_source(
        &self,
        c: &Context,
        source: &Source,
        spec: IpSpec,
    ) -> Result<Ip, Fault> {
        let env = Environment::new()
            // read config.toml for setting any env variables
            .from_config(c.get_config())?;
        let mut vtable = StrSwapTable::new().load_environment(&env)?;
        env.initialize();

        let protocols: ProtocolMap = c.get_config().get_protocols();

        // fetch from the internet
        let (_name, bytes) = Download::download(
            &mut vtable,
            Some(&spec.to_partial_ip_spec()),
            &source,
            None,
            c.get_downloads_path(),
            &protocols,
            self.verbose,
            self.force,
        )?;

        let dir = tempfile::tempdir()?.into_path();
        if let Err(e) = IpArchive::extract(&bytes, &dir) {
            fs::remove_dir_all(dir)?;
            return Err(e);
        }
        // load the IP
        let unzipped_ip = match Ip::load(dir.clone(), false, false) {
            Ok(x) => x,
            Err(e) => {
                fs::remove_dir_all(dir)?;
                return Err(e);
            }
        };
        Ok(unzipped_ip)
    }

    pub fn is_checksum_good(root: &PathBuf) -> bool {
        // verify the checksum
        if let Some(sha) = Ip::read_cache_checksum(&root) {
            // make sure the sums match expected
            sha == Ip::compute_checksum(&root)
        // failing to compute a checksum
        } else {
            false
        }
    }

    /// Installs the `ip` with particular partial `version` to the `cache_root`.
    /// It will reinstall if it finds the original installation has a mismatching checksum.
    ///
    /// Returns `true` if the IP was successfully installed and `false` if it already existed.
    pub fn install(
        src: &Ip,
        cache_root: &PathBuf,
        force: bool,
        verbose: bool,
    ) -> Result<Option<Ip>, Fault> {
        // temporary destination to move files for processing and manipulation
        let dest = tempfile::tempdir()?.into_path();
        filesystem::copy(src.get_root(), &dest, true, Some(src.get_files_to_keep()))?;

        // lookup the package name in the index to see if the UUIDs match
        // verify the version for this package is not already logged

        // @note: a package's index file contains all metadata for all versions known to orbit
        // @note: ability to link various index directories (essentially vendors)
        // @note: also want to store zipped archives of installs in the "vault" for quicker retrieval

        // @todo: listing all units

        // @todo: store a LUT for unit names to the correct file to read when computing "get" command

        // @todo: getting the size of the entire directory

        // access the name and version
        let version = src.get_man().get_ip().get_version();
        let ip_spec = src.get_man().get_ip().into_ip_spec();

        if verbose == true {
            crate::info!("stalling ip {} ...", &ip_spec);
        }

        // perform sha256 on the temporary cloned directory
        let checksum = Ip::compute_checksum(&dest);
        // println!("checksum: {}", checksum);

        // use checksum to create new directory slot
        let cache_slot_name = CacheSlot::new(src.get_uuid(), &version, &checksum);
        let cache_slot = cache_root.join(&cache_slot_name.to_string());
        // find a directory that has this name beginning if using force
        if force == true {
            for dir in fs::read_dir(cache_root)? {
                // only check valid directory entries
                if let Ok(entry) = dir {
                    // println!("{:?}", entry.file_name());
                    let file_name = entry.file_name();

                    if let Some(kid_cache_slot) =
                        CacheSlot::try_from_str(&file_name.to_string_lossy())
                    {
                        if cache_slot_name.is_child_slot(&kid_cache_slot) == false {
                            continue;
                        }
                        // check for same UUID
                        let cached_ip = Ip::load(entry.path().to_path_buf(), false, false)?;
                        if cached_ip.get_uuid() == src.get_uuid() {
                            // remove the slot no matter if it is dynamic or not
                            fs::remove_dir_all(entry.path())?;
                        }
                    }
                }
            }
        }
        // check if the slot is occupied in the cache
        if cache_slot.exists() == true {
            // check if we should proceed with force regardless if the installation is valid
            if force == true {
                std::fs::remove_dir_all(&cache_slot)?;
            } else {
                // ip is already installed
                if Self::is_checksum_good(&cache_slot) == true {
                    // clean up the temporary directory ourself
                    fs::remove_dir_all(dest)?;
                    return Ok(None);
                } else {
                    if verbose == true {
                        crate::info!("reinstalling ip {} due to bad checksum ...", ip_spec);
                    }
                    // blow directory up for re-install
                    std::fs::remove_dir_all(&cache_slot)?;
                }
            }
        }
        // copy contents into cache slot from temporary destination
        crate::util::filesystem::copy(&dest, &cache_slot, false, Some(src.get_files_to_keep()))?;

        // clean up the temporary directory ourself
        fs::remove_dir_all(dest)?;

        let installed_ip = Ip::load(cache_slot, false, false)?;

        // write the checksum to the directory (this file is excluded from auditing)
        installed_ip.write_cache_checksum(&checksum)?;
        // write the metadata
        installed_ip.write_cache_metadata()?;

        Ok(Some(installed_ip))
    }

    fn run(&self, target: &Ip, catalog: &Catalog) -> Result<(), Fault> {
        let result = Self::install(&target, &catalog.get_cache_path(), self.force, true)?;
        match result {
            Some(_) => (),
            None => crate::info!(
                "ip {} is already installed",
                target.get_man().get_ip().into_ip_spec()
            ),
        }

        Ok(())
        // store results from expensive computations into specific orbit files

        // print download list for top-level package
        // if self.compile == true {
        //     for s in Self::compile_download_list(ip.get_lock(), &catalog, false) {
        //         println!("{}", s);
        //     }
        //     return Ok(())
        // }

        // _pkg.get_lock().save_to_disk(&_pkg.get_root())?;
        // todo!();

        // @todo: check lockfile to process installing any IP that may be already downloaded to the queue

        // verify each requirement for the IP is also installed (o.w. install)

        // if let Some(lock) = man.get_lockfile() {
        //     Self::install_from_lock_file(&self, &lock, &catalog)?;
        // }
        // if the lockfile is invalid, then it will only install the current request and zero dependencies
    }
}

// # install from online using custom protocol
// orbit install toolbox:1.0.1 --url https://github.com/chaseruskin/toolbox.git --protocol git-op

// # install from local path
// orbit install hamming:1.0.0 --path .

// # install from online using default protocol
// orbit install --url https://github.com/chaseruskin/toolbox/archive/refs/tags/1.0.1.zip

// # install from queue
// orbit install toolbox:1.0.1

// error if multiple packages are located in a downloaded area (then they must supply a ip spec)
