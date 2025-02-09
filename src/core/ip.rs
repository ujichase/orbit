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

use crate::core::manifest;
use crate::core::manifest::Manifest;
use crate::error::LastError;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::CodeFault;
use crate::util::anyerror::Fault;
use std::path::PathBuf;

use super::catalog::Catalog;
use super::catalog::PkgName;
use super::iparchive::IpArchive;
use super::ippointer::IpPointer;
use super::lang;
use super::lang::LangIdentifier;
use super::lang::LangUnit;
use super::lockfile::LockFile;
use super::lockfile::IP_LOCK_FILE;
use super::manifest::FromFile;
use super::version::PartialVersion;
use super::visibility::VipList;
use super::visibility::Visibility;
use crate::core::lockfile::LockEntry;
use crate::core::manifest::IP_MANIFEST_FILE;
use crate::core::manifest::ORBIT_METADATA_FILE;
use crate::core::manifest::ORBIT_SUM_FILE;
use crate::core::uuid::Uuid;
use crate::error::Error;
use crate::error::Hint;
use crate::util::sha256::Sha256Hash;
use colored::Colorize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;
use toml_edit::Document;

// add state to `root` (make enum) to determine if is real path or not
#[derive(Debug, PartialEq)]
pub enum Mapping {
    Physical,
    Virtual(Vec<u8>),
    Relative(PathBuf),
    Imaginary,
}

impl Mapping {
    pub fn is_physical(&self) -> bool {
        match &self {
            Self::Physical | Self::Relative(_) => true,
            _ => false,
        }
    }

    pub fn is_relative(&self) -> bool {
        match &self {
            Self::Relative(_) => true,
            _ => false,
        }
    }

    pub fn is_pointer(&self) -> bool {
        match &self {
            Self::Imaginary => true,
            _ => false,
        }
    }

    pub fn as_relative_path(&self) -> Option<&PathBuf> {
        match &self {
            Self::Relative(p) => Some(p),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> Option<&Vec<u8>> {
        match &self {
            Self::Virtual(b) => Some(b),
            _ => None,
        }
    }
}

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    protected: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct Ip {
    mapping: Mapping,
    /// The base directory for the entire [Ip] structure.
    root: PathBuf,
    /// The metadata for the [Ip].
    data: Manifest,
    /// The lockfile for the [Ip].
    lock: LockFile,
    /// The UUID for the [Ip].
    uuid: Uuid,
}

impl From<IpPointer> for Ip {
    fn from(value: IpPointer) -> Self {
        let man = value.decouple();
        Self {
            uuid: man.get_ip().get_uuid().clone(),
            mapping: Mapping::Imaginary,
            root: PathBuf::new(),
            data: man,
            lock: LockFile::new(),
        }
    }
}

impl From<IpArchive> for Ip {
    fn from(value: IpArchive) -> Self {
        let (man, lock, archive) = value.decouple();
        let uuid = match lock.get_self_entry(man.get_ip().get_name()) {
            Some(entry) => entry.get_uuid().clone(),
            None => match lock.get(
                man.get_ip().get_name(),
                &man.get_ip().get_version().to_partial_version(),
            ) {
                Some(entry) => entry.get_uuid().clone(),
                None => Uuid::new(),
            },
        };
        Self {
            mapping: Mapping::Virtual(archive),
            root: PathBuf::new(),
            data: man,
            lock: lock,
            uuid: uuid,
        }
    }
}

impl Ip {
    pub fn has_public_list(&self) -> bool {
        VipList::new(&self.get_root(), self.get_man().get_ip().get_publics())
            .unwrap()
            .exists()
    }

    pub fn into_public_list(&self) -> VipList {
        VipList::new(&self.get_root(), self.get_man().get_ip().get_publics()).unwrap()
    }

    /// Generates a list of files that are known to either be public or protected.
    pub fn into_non_private_list(&self) -> VipList {
        let meta = Ip::read_cache_metadata(self.get_root());
        let mut list = match meta {
            Some(m) => match m.protected.len() {
                0 => None,
                _ => Some(m.protected),
            },
            None => None,
        };
        if let Some(public) = self.get_man().get_ip().get_publics() {
            list = match list {
                Some(mut l) => {
                    l.extend(public.clone());
                    Some(l)
                }
                None => Some(public.clone()),
            };
        }
        VipList::new(&self.get_root(), &list).unwrap()
    }

    pub fn get_root(&self) -> &PathBuf {
        &self.root
    }

    pub fn get_mapping(&self) -> &Mapping {
        &self.mapping
    }

    pub fn get_man(&self) -> &Manifest {
        &self.data
    }

    pub fn get_lock(&self) -> &LockFile {
        &self.lock
    }

    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// Try to get the checksum with least effort possible. Will not work for
    /// non physical mappings of an ip.
    pub fn get_checksum(&self) -> Option<Sha256Hash> {
        match self.get_mapping() {
            Mapping::Physical => match Ip::read_cache_checksum(&self.get_root()) {
                Some(sum) => Some(sum),
                None => Some(Ip::compute_checksum(&self.get_root())),
            },
            _ => None,
        }
    }

    /// Gets the protected files listed in the .orbit-metadata file.
    pub fn get_protected_files(&self) -> Option<Vec<String>> {
        todo!()
    }

    pub fn check_illegal_files(root: &PathBuf) -> Result<(), Fault> {
        // verify no reserved files exist
        if let Ok(mut rd) = std::fs::read_dir(&root) {
            let pat = ".orbit-";
            // upon force, will remove all installations (even dynamics)
            while let Some(d) = rd.next() {
                if let Ok(p) = d {
                    if p.file_name().into_string().unwrap().starts_with(&pat) == true {
                        return Err(AnyError(format!("Illegal file {:?} found in ip; files starting with \"{}\" are reserved for internal use", p.path(), pat)))?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Loads an ip from the `root` path, which is read from a manifest file located
    /// at `base_path`.
    pub fn relate(root: PathBuf, base_path: &PathBuf) -> Result<Self, Fault> {
        // resolve the path if it is relative
        let resolved_root = filesystem::resolve_rel_path2(&base_path, &root);
        let mut relative_ip = Ip::load(resolved_root, false, false)?;
        relative_ip.mapping = Mapping::Relative(root);
        // verify this ip has a lockfile
        let lock_path = relative_ip.get_root().join(IP_LOCK_FILE);
        if lock_path.exists() == false || lock_path.is_file() == false {
            return Err(Error::LockfileLoadFailed(LastError(
                "a lockfile does not exist".to_string(),
            )))?;
        }
        match LockFile::from_file(&lock_path) {
            Ok(_) => (),
            Err(e) => return Err(Error::LockfileLoadFailed(LastError(e.to_string())))?,
        }
        // check to make sure we are not the same package
        Ok(relative_ip)
    }

    /// Load an [Ip] instance from the `root` path.
    ///
    /// If `is_working_ip` is true, then it verifies there are no files created
    /// by the user that are reserved for orbit's internal usage.
    pub fn load(
        root: PathBuf,
        is_working_ip: bool,
        force_apply_new_uuid: bool,
    ) -> Result<Self, Fault> {
        let man_path = root.join(IP_MANIFEST_FILE);
        if man_path.exists() == false || man_path.is_file() == false {
            return Err(Error::IpLoadFailed(LastError(
                "a manifest file does not exist".to_string(),
            )))?;
        }
        let man = Manifest::from_file(&man_path)?;

        // verify the public list is okay
        VipList::new(&root, man.get_ip().get_publics())?;

        if is_working_ip == true {
            // verify there are no files that created by user that are reserved for orbit's internal use
            match Self::check_illegal_files(&root) {
                Ok(()) => (),
                Err(e) => return Err(Error::IpLoadFailed(LastError(e.to_string())))?,
            }
        }

        let lock_path = root.join(IP_LOCK_FILE);

        let lock = match LockFile::from_file(&lock_path) {
            Ok(l) => l,
            Err(e) => {
                println!(
                    "{}: failed to parse lockfile \"{}\": {}",
                    "warning".yellow().bold(),
                    filesystem::into_std_str(lock_path),
                    e
                );
                LockFile::new()
            }
        };

        let uuid = man.get_ip().get_uuid().clone();

        // verify the UUIDs between the manifest and lockfile are the same
        if is_working_ip == true {
            //  println!("manifest: {:?}", uuid);
            if let Some(lf) = lock.get_self_entry(man.get_ip().get_name()) {
                // println!("lockfile: {:?}", lf.get_uuid());
                if lf.get_uuid() != &uuid && force_apply_new_uuid == false {
                    return Err(Error::UuidModified(
                        man.get_ip().get_name().clone(),
                        Hint::ConfirmUuidChange(lf.get_uuid().encode()),
                    ))?;
                }
            }
        }

        Ok(Self {
            mapping: Mapping::Physical,
            root: root,
            data: man,
            lock: lock,
            uuid: uuid,
        })
    }

    /// Checks if the given path hosts a valid manifest file.
    pub fn is_valid(path: &PathBuf) -> Result<(), Fault> {
        let man_path = path.join(IP_MANIFEST_FILE);
        if man_path.exists() == false || man_path.is_file() == false {
            return Err(AnyError(format!("A manifest file does not exist")))?;
        }
        // attempt to load the manifest file
        let _ = Manifest::from_file(&man_path)?;
        return Ok(());
    }

    /// Finds all Manifest files available in the provided path `path`.
    ///
    /// Errors if on filesystem problems.
    fn detect_all_sub(
        path: &PathBuf,
        name: &str,
        is_exclusive: bool,
        is_working: bool,
    ) -> Result<Vec<Self>, Fault> {
        let mut result = Vec::new();
        // walk the ORBIT_PATH directory @TODO recursively walk inner directories until hitting first 'Orbit.toml' file
        for mut entry in manifest::find_file(&path, &name, is_exclusive)? {
            // remove the manifest file to access the ip's root directory
            entry.pop();
            result.push(Ip::load(entry, is_working, false)?);
        }
        Ok(result)
    }

    /// Finds all IP manifest files along the provided path `path`.
    ///
    /// Wraps Manifest::detect_all.
    pub fn detect_all(
        path: &PathBuf,
        is_working: bool,
    ) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        Self::detect_all_sub(path, IP_MANIFEST_FILE, true, is_working)
    }

    /// Checks the metadata file for a entry for `dynamic`.
    pub fn is_dynamic(&self) -> bool {
        self.get_mapping().is_physical() == true
            && self.get_root().join(".orbit-dynamic").exists() == true
    }

    /// Gets the proper library name for the ip. If there is a "collision" with the library name and an identifier that
    /// required symbol transformation, this function will return the transformed name for the library name as well.
    pub fn get_hdl_library(&self) -> LangIdentifier {
        match self.is_dynamic() {
            // check if we made a match with a name under contention
            true => match self.library_collides_with_dst() {
                Some(transform) => LangIdentifier::from_str(&transform).unwrap(),
                None => self.get_man().get_hdl_library(),
            },
            false => self.get_man().get_hdl_library(),
        }
    }

    /// Creates the lookup table for the DST algorithm.
    pub fn generate_dst_lut(&self) -> HashMap<LangIdentifier, String> {
        // compose the lut for symbol transformation
        let mut lut = HashMap::new();

        if self.mapping.is_physical() == false {
            return lut;
        }
        // @todo: read units from metadata to speed up results
        let units = self.collect_units(true, self.has_public_list()).unwrap();
        let checksum = Ip::read_cache_checksum(self.get_root()).unwrap();

        units.into_iter().for_each(|(key, _)| {
            lut.insert(
                key.clone(),
                "_".to_string() + checksum.to_string().get(0..10).unwrap(),
            );
        });
        lut
    }

    pub fn set_as_dynamic(&self, mapping: &HashMap<LangIdentifier, String>) -> () {
        let contents = mapping.iter().fold(String::new(), |mut acc, (k, v)| {
            acc.push_str(&format!("{}\t{}\n", k, v));
            acc
        });
        let _ = std::fs::write(self.get_root().join(".orbit-dynamic"), &contents).unwrap();
    }

    fn library_collides_with_dst(&self) -> Option<String> {
        match self.is_dynamic() {
            // check the list of symbols
            true => {
                let words =
                    std::fs::read_to_string(self.get_root().join(".orbit-dynamic")).unwrap();
                let lib = self.get_man().get_hdl_library().to_string();
                words.split_terminator('\n').find_map(|entry| {
                    let (key, val) = entry.split_once('\t').unwrap();
                    match key == &lib {
                        true => Some(format!("{}{}", key, val)),
                        false => None,
                    }
                })
            }
            false => None,
        }
    }

    /// Checks if needing to read off the lock file.
    ///
    /// This determines if the lock file's data matches the Orbit.toml manifest data,
    /// indicating it is safe to pull data from the lock file and no changes would be
    /// made to the lock file.
    pub fn can_use_lock(&self, catalog: &Catalog) -> bool {
        let target = self.get_lock().get(
            self.get_man().get_ip().get_name(),
            &self.get_man().get_ip().get_version().to_partial_version(),
        );
        let target_is_ok = match target {
            Some(entry) => entry.matches_target(&LockEntry::from((self, true)), &catalog),
            None => false,
        };
        if target_is_ok == false {
            return false;
        }
        // check that all entries are valid of dependencies and dev dependencies
        for dep in self.get_man().get_deps_list(true, true) {
            if let Some(entry) = self.get_lock().get(dep.0, dep.1.get_version()) {
                if let Some(relative_ip) = dep.1.as_ip() {
                    if &LockEntry::from((relative_ip, true)) == entry {
                        ()
                    } else {
                        return false;
                    }
                }
            } else {
                return false;
            }
        }
        true
    }

    /// Checks if the lockfile exists
    pub fn lock_exists(&self) -> bool {
        self.lock.is_empty() == false
    }

    /// Computes the checksum on the root of the IP.
    ///
    /// Changes the current working directory to the root for consistent computation.
    pub fn compute_checksum(dir: &PathBuf) -> Sha256Hash {
        let ip_files = crate::util::filesystem::gather_current_files(&dir, true);
        let checksum = crate::util::checksum::checksum(&ip_files, &dir);
        checksum
    }

    /// Gets the already calculated checksum from an installed ip from [ORBIT_SUM_FILE].
    ///
    /// Returns `None` if the file does not exist, is unable to read into a string, or
    /// if the sha cannot be parsed.
    pub fn read_cache_checksum(dir: &PathBuf) -> Option<Sha256Hash> {
        let sum_file = dir.join(ORBIT_SUM_FILE);
        if sum_file.exists() == false {
            None
        } else {
            match std::fs::read_to_string(&sum_file) {
                Ok(text) => match Sha256Hash::from_str(&text.trim()) {
                    Ok(sha) => Some(sha),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        }
    }

    /// Gets the already cached internal metadata for the install ip from [ORBIT_METADATA_FILE].
    pub fn read_cache_metadata(dir: &PathBuf) -> Option<Metadata> {
        let meta_file = dir.join(ORBIT_METADATA_FILE);
        if meta_file.exists() == false {
            None
        } else {
            match std::fs::read_to_string(&meta_file) {
                Ok(text) => match serde_json::from_str(&text) {
                    Ok(data) => Some(data),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        }
    }

    pub fn write_cache_checksum(&self, sum: &Sha256Hash) -> Result<(), Fault> {
        let path = self.get_root().join(manifest::ORBIT_SUM_FILE);
        std::fs::write(&path, sum.to_string().as_bytes())?;
        Ok(())
    }

    pub fn write_cache_metadata(&self) -> Result<(), Fault> {
        // generate the unit map
        let umap = self.collect_units(false, true)?;
        let protected: Vec<String> = umap
            .iter()
            .filter(|(_, v)| v.get_visibility().is_protected())
            .map(|(_, v)| {
                filesystem::into_std_str(filesystem::remove_base(
                    self.get_root(),
                    &PathBuf::from(v.get_source_file()),
                ))
            })
            .collect();

        let meta = Metadata {
            protected: protected,
        };

        let serialized = serde_json::to_string(&meta)?;
        let path = self.get_root().join(manifest::ORBIT_METADATA_FILE);
        std::fs::write(&path, serialized)?;
        Ok(())
    }

    /// Caches the result of collecting all the primary design units for the given package.
    ///
    /// Writes the data to the toml data structure. Note, this function does not save the manifest data to file.
    // pub fn stash_units(&mut self) -> () {
    //     // collect the units
    //     let units = Self::collect_units(true).unwrap();
    //     let tbl = self.get_manifest_mut().get_mut_doc()["ip"].as_table_mut().unwrap();
    //     tbl.insert("units", toml_edit::Item::Value(toml_edit::Value::Array(Array::new())));
    //     let arr = tbl["units"].as_array_mut().unwrap();
    //     // map the units into a serialized data format
    //     for (_, unit) in &units {
    //         arr.push(unit.to_toml());
    //     }
    //     tbl["units"].as_array_mut().unwrap().iter_mut().for_each(|f| {
    //         f.decor_mut().set_prefix("\n    ");
    //         f.decor_mut().set_suffix("");
    //     });
    //     tbl["units"].as_array_mut().unwrap().set_trailing("\n");
    // }

    /// Gathers the list of primary design units for the current ip.
    ///
    /// If the manifest has an toml entry for `units` and `force` is set to `false`,
    /// then it will return that list rather than go through files.
    pub fn collect_units(
        &self,
        force: bool,
        hide_private: bool,
    ) -> Result<HashMap<LangIdentifier, LangUnit>, CodeFault> {
        let public_list = self.into_public_list();
        // try to read from metadata file
        match force == false && Self::read_units_from_metadata(&self.get_root()).is_some() {
            // use precomputed result
            true => Ok(Self::read_units_from_metadata(&self.get_root()).unwrap()),
            false => {
                // collect all files
                let files = self.gather_current_files();

                let mut map = lang::collect_units(&files)?;

                // work to remove files that are totally private
                if public_list.exists() == true {
                    // track which files are private and have no references or only private references
                    let mut private_set: HashSet<LangIdentifier> = map
                        .iter_mut()
                        .filter_map(|(k, v)| {
                            // the node is implicitly private, but so far only known to be protected
                            if v.is_listed_public(&public_list) == false {
                                v.set_visibility(Visibility::Protected);
                                Some(k.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                    let mut visited: HashSet<LangIdentifier> = HashSet::new();
                    map.iter().for_each(|(_k, v)| {
                        // the node is explicitly public
                        if v.is_listed_public(&public_list) == true {
                            // if the reference is used by a public then it and its nesteddeps are not totally invisible
                            let mut stack = v.get_references();
                            while let Some(item) = stack.pop() {
                                // remove this item from the private map
                                private_set.remove(&item);
                                if visited.contains(&item) == false {
                                    if let Some(id) = map.get(&item) {
                                        for refer in id.get_references() {
                                            // println!("{:?}", refer);
                                            stack.push(refer);
                                        }
                                    }
                                }
                                visited.insert(item);
                            }
                        }
                    });
                    // println!("totally private: {:?}", private_set);
                    for k in &private_set {
                        if let Some(v) = map.get_mut(k) {
                            v.set_visibility(Visibility::Private);
                        }
                    }
                    if hide_private == true {
                        // remove totally invisible units from list
                        map = map
                            .into_iter()
                            .filter(|(k, _v)| private_set.contains(k) == false)
                            .collect();
                    }
                }
                Ok(map)
            }
        }
    }

    pub fn read_units_from_metadata(dir: &PathBuf) -> Option<HashMap<LangIdentifier, LangUnit>> {
        let meta_file: PathBuf = dir.join(ORBIT_METADATA_FILE);
        if Path::exists(&meta_file) == true {
            if let Ok(contents) = fs::read_to_string(&meta_file) {
                if let Ok(toml) = contents.parse::<Document>() {
                    let entry = toml.get("ip")?.as_table()?.get("units")?.as_array()?;
                    let mut map = HashMap::new();
                    for unit in entry {
                        let lu = LangUnit::from_toml(unit.as_inline_table()?)?;
                        map.insert(lu.get_name().clone(), lu);
                    }
                    Some(map)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_include_list(&self) -> Result<VipList, Fault> {
        VipList::new(&self.root, &self.get_man().get_ip().get_include())
    }

    pub fn get_exclude_list(&self) -> Result<VipList, Fault> {
        VipList::new(&self.root, &self.get_man().get_ip().get_exclude())
    }

    pub fn gather_current_files(&self) -> Vec<String> {
        let inc = match self.get_include_list() {
            Ok(vip) => match vip.exists() {
                true => Some(vip),
                false => None,
            },
            Err(_) => None,
        };
        let exc = match self.get_exclude_list() {
            Ok(vip) => match vip.exists() {
                true => Some(vip),
                false => None,
            },
            Err(_) => None,
        };
        filesystem::gather_current_files(&self.root, false)
            .into_iter()
            .filter(|f| match &inc {
                Some(vip) => vip.is_included(f.as_ref()) == true,
                None => match &exc {
                    Some(vip) => vip.is_included(f.as_ref()) == false,
                    None => true,
                },
            })
            .collect()
    }

    /// Compile a list of referenced paths to make sure are copied into a directory
    /// when moving an IP around the filesystem.
    pub fn get_files_to_keep(&self) -> HashSet<PathBuf> {
        let mut list = HashSet::new();
        // keep the readme if set in manifest
        if let Some(readme) = self.get_man().get_ip().get_readme() {
            // resolve a relative path
            list.insert(filesystem::resolve_rel_path2(self.get_root(), readme));
        }
        list
    }
}

use crate::core::pkgid::PkgPart;
use crate::core::version::Version;
use crate::util::filesystem;
use std::fs;
use std::path::Path;

const SPEC_DELIM: &str = ":";
const UUID_DELIM: &str = "+";

#[derive(PartialEq, Hash, Eq, Clone, PartialOrd)]
pub struct IpSpec(PkgPart, Uuid, Version);

impl IpSpec {
    pub fn new(id: PkgPart, uuid: Uuid, version: Version) -> Self {
        Self(id, uuid, version)
    }

    pub fn get_name(&self) -> &PkgPart {
        &self.0
    }

    pub fn get_uuid(&self) -> &Uuid {
        &self.1
    }

    pub fn get_version(&self) -> &Version {
        &self.2
    }

    pub fn to_partial_ip_spec(&self) -> PartialIpSpec {
        PartialIpSpec(
            self.0.clone(),
            Some(self.1.clone()),
            AnyVersion::Specific(self.2.to_partial_version()),
        )
    }

    pub fn to_pkg_name(&self) -> PkgName {
        PkgName::new(&self.0, Some(&self.1))
    }
}

impl FromStr for IpSpec {
    type Err = Fault;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // split by delimiter
        match s.rsplit_once(SPEC_DELIM) {
            Some((rem, v)) => match rem.rsplit_once(UUID_DELIM) {
                Some((name, id)) => Ok(Self::new(
                    PkgPart::from_str(name)?,
                    Uuid::from_str(id)?,
                    Version::from_str(v)?,
                )),
                None => Err(Box::new(AnyError(format!(
                    "missing uuid delimiter {}",
                    UUID_DELIM
                )))),
            },
            None => Err(Box::new(AnyError(format!(
                "missing version delimiter {}",
                SPEC_DELIM
            )))),
        }
    }
}

impl std::fmt::Debug for IpSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}{}",
            self.get_name(),
            UUID_DELIM,
            self.get_uuid().encode(),
            SPEC_DELIM,
            self.get_version()
        )
    }
}

impl std::fmt::Display for IpSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.get_name(), SPEC_DELIM, self.get_version())
    }
}

impl From<(PkgPart, Uuid, Version)> for IpSpec {
    fn from(value: (PkgPart, Uuid, Version)) -> Self {
        Self(value.0, value.1, value.2)
    }
}

use serde::de::{self};
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;

impl<'de> Deserialize<'de> for IpSpec {
    fn deserialize<D>(deserializer: D) -> Result<IpSpec, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = IpSpec;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an identifier and a version")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match IpSpec::from_str(v) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(de::Error::custom(e)),
                }
            }
        }

        deserializer.deserialize_map(LayerVisitor)
    }
}

impl Serialize for IpSpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:?}", self))
    }
}

use crate::core::version::AnyVersion;

#[derive(PartialEq, Clone, Hash, Eq)]
pub struct PartialIpSpec(PkgPart, Option<Uuid>, AnyVersion);

impl PartialIpSpec {
    pub fn new(name: PkgPart, uuid: Option<Uuid>, version: PartialVersion) -> Self {
        Self(name, uuid, AnyVersion::Specific(version))
    }

    pub fn get_name(&self) -> &PkgPart {
        &self.0
    }

    pub fn as_uuid(&self) -> &Option<Uuid> {
        &self.1
    }

    pub fn get_version(&self) -> &AnyVersion {
        &self.2
    }

    pub fn as_ip_spec(&self) -> Option<IpSpec> {
        Some(IpSpec::new(
            self.0.clone(),
            self.1.as_ref()?.clone(),
            self.2.as_specific()?.as_version()?,
        ))
    }

    pub fn to_pkg_name(&self) -> PkgName {
        PkgName::new(&self.0, self.1.as_ref())
    }
}

impl std::fmt::Debug for PartialIpSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.1 {
            Some(id) => write!(f, "{}{}{}{}{}", self.0, UUID_DELIM, id, SPEC_DELIM, self.2),
            None => write!(f, "{}{}{}", self.0, SPEC_DELIM, self.2),
        }
    }
}

impl<'de> Deserialize<'de> for PartialIpSpec {
    fn deserialize<D>(deserializer: D) -> Result<PartialIpSpec, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = PartialIpSpec;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an identifier and a version")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match PartialIpSpec::from_str(v) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(de::Error::custom(e)),
                }
            }
        }

        deserializer.deserialize_map(LayerVisitor)
    }
}

impl Serialize for PartialIpSpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:?}", self))
    }
}

struct FullName {
    name: PkgPart,
    id: Uuid,
}

impl FromStr for PartialIpSpec {
    type Err = AnyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.rsplit_once(SPEC_DELIM) {
            // split by delimiter (beginning from rhs)
            Some((rem, ver)) => match rem.rsplit_once(UUID_DELIM) {
                Some((name, id)) => Ok(Self(
                    match PkgPart::from_str(name) {
                        Ok(p) => p,
                        Err(e) => return Err(AnyError(e.to_string())),
                    },
                    match Uuid::from_str(id) {
                        Ok(u) => Some(u),
                        Err(e) => return Err(AnyError(e.to_string())),
                    },
                    match AnyVersion::from_str(ver) {
                        Ok(w) => w,
                        Err(e) => return Err(AnyError(e.to_string())),
                    },
                )),
                None => Ok(Self(
                    match PkgPart::from_str(rem) {
                        Ok(p) => p,
                        Err(e) => return Err(AnyError(e.to_string())),
                    },
                    None,
                    match AnyVersion::from_str(ver) {
                        Ok(w) => w,
                        Err(e) => return Err(AnyError(e.to_string())),
                    },
                )),
            },
            None => match s.rsplit_once(UUID_DELIM) {
                Some((name, id)) => Ok(Self(
                    match PkgPart::from_str(name) {
                        Ok(p) => p,
                        Err(e) => return Err(AnyError(e.to_string())),
                    },
                    match Uuid::from_str(id) {
                        Ok(u) => Some(u),
                        Err(e) => return Err(AnyError(e.to_string())),
                    },
                    AnyVersion::Latest,
                )),
                None => Ok(Self(
                    match PkgPart::from_str(s) {
                        Ok(p) => p,
                        Err(e) => return Err(AnyError(e.to_string())),
                    },
                    None,
                    AnyVersion::Latest,
                )),
            },
        }
    }
}

impl std::fmt::Display for PartialIpSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.get_name(), SPEC_DELIM, self.get_version())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compute_checksum() {
        let sum = Ip::compute_checksum(&PathBuf::from("./tests/t6/"));
        assert_eq!(
            sum,
            Sha256Hash::from_u32s([
                2618638528, 43206360, 3250567762, 2433311059, 2801205324, 1302924505, 140461672,
                3678450175
            ])
        )
    }

    #[test]
    fn from_str_ip_spec() {
        let ip = format!(
            "name{}71vs0nyo7lqjji6p6uzfviaoi{}1.0.0",
            UUID_DELIM, SPEC_DELIM
        );

        assert_eq!(
            IpSpec::new(
                PkgPart::from_str("name").unwrap(),
                Uuid::from_str("71vs0nyo7lqjji6p6uzfviaoi").unwrap(),
                Version::from_str("1.0.0").unwrap()
            ),
            IpSpec::from_str(&ip).unwrap()
        );
    }

    #[test]
    fn from_str_ip_spec_bad() {
        let ip = format!("name");

        assert_eq!(IpSpec::from_str(&ip).is_err(), true);
    }
}
