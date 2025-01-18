//
//  Copyright (C) 2022-2024  Chase Ruskin
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

#![allow(dead_code)]

use crate::core::ip::IpSpec;
use crate::core::pkgid::PkgPart;
use crate::core::source::Source;
use crate::core::{source, version};
use crate::error::Error;
use crate::util::anyerror::{AnyError, Fault};
use serde::de::{self, MapAccess, Visitor};
use serde_derive::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::path::PathBuf;
use std::{collections::HashMap, str::FromStr};

use super::ip::Ip;
use super::lang::vhdl::token::identifier::Identifier;
use super::lang::LangIdentifier;
use super::uuid::Uuid;

pub type IpName = PkgPart;
pub type IpVersion = crate::core::version::Version;
pub type DepVersion = crate::core::version::PartialVersion;

#[derive(Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields, transparent)]
pub struct Dependency {
    version: DepVersion,
    #[serde(skip_serializing)]
    path: Option<PathBuf>,
    #[serde(skip_serializing)]
    relative_ip: Option<Ip>,
    #[serde(skip_serializing)]
    uuid: Option<Uuid>,
}

impl Dependency {
    pub fn is_relative(&self) -> bool {
        self.path.is_some()
    }

    pub fn get_version(&self) -> &DepVersion {
        &self.version
    }

    pub fn as_path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    pub fn as_ip(&self) -> Option<&Ip> {
        self.relative_ip.as_ref()
    }

    pub fn as_uuid(&self) -> Option<&Uuid> {
        self.uuid.as_ref()
    }
}

impl<'de> serde::Deserialize<'de> for Dependency {
    fn deserialize<D>(deserializer: D) -> Result<Dependency, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        enum Field {
            Path,
            Version,
            Uuid,
        }

        // This part could also be generated independently by:
        //
        //    #[derive(Deserialize)]
        //    #[serde(field_identifier, rename_all = "lowercase")]
        //    enum Field { Secs, Nanos }
        impl<'de> serde::Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`path` or `version` or `uuid`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "path" => Ok(Field::Path),
                            "version" => Ok(Field::Version),
                            "uuid" => Ok(Field::Uuid),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        // This is a Visitor that forwards string types to T's `FromStr` impl and
        // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
        // keep the compiler from complaining about T being an unused generic type
        // parameter. We need T in order to know the Value type for the Visitor
        // impl.
        struct LayerVisitor;

        impl<'de> Visitor<'de> for LayerVisitor {
            type Value = Dependency;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string or map")
            }

            fn visit_str<E>(self, value: &str) -> Result<Dependency, E>
            where
                E: de::Error,
            {
                Ok(Dependency {
                    path: None,
                    version: match DepVersion::from_str(value) {
                        Ok(v) => v,
                        Err(e) => return Err(de::Error::custom(e))?,
                    },
                    relative_ip: None,
                    uuid: None,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Dependency, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut path: Option<PathBuf> = None;
                let mut version: Option<DepVersion> = None;
                let mut id: Option<Uuid> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Path => {
                            if path.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            path = Some(map.next_value()?);
                        }
                        Field::Version => {
                            if version.is_some() {
                                return Err(de::Error::duplicate_field("version"));
                            }
                            version = Some(map.next_value()?);
                        }
                        Field::Uuid => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("uuid"));
                            }
                            id = Some(map.next_value()?);
                        }
                    }
                }
                let path = path;
                let version = version.ok_or_else(|| de::Error::missing_field("version"))?;
                let id = id;
                Ok(Dependency {
                    path: path,
                    version: version,
                    relative_ip: None,
                    uuid: id,
                })
            }
        }

        const FIELDS: &[&str] = &["path", "version", "uuid"];
        deserializer.deserialize_struct("Dependency", FIELDS, LayerVisitor)
    }
}

type Dependencies = HashMap<IpName, Dependency>;

pub const IP_MANIFEST_FILE: &str = "Orbit.toml";
pub const ORBIT_SUM_FILE: &str = ".orbit-checksum";
pub const ORBIT_METADATA_FILE: &str = ".orbit-metadata";

const DEPENDENCIES_KEY: &str = "dependencies";

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    ip: Package,
    #[serde(skip_serializing_if = "map_is_empty", default)]
    dependencies: Dependencies,
    #[serde(
        rename = "dev-dependencies",
        skip_serializing_if = "map_is_empty",
        default
    )]
    dev_dependencies: Dependencies,
}

pub trait FromFile: FromStr
where
    Self: Sized,
    <Self as std::str::FromStr>::Err: 'static + std::error::Error,
{
    fn from_file(path: &PathBuf) -> Result<Self, Fault> {
        // try to open the file in read-only mode
        let text = std::fs::read_to_string(&path)?;
        Ok(Self::from_str(&text)?)
    }
}

impl FromFile for Manifest {
    fn from_file(path: &PathBuf) -> Result<Self, Fault> {
        // open file
        let contents = std::fs::read_to_string(&path)?;
        // parse toml syntax
        let mut man = match Self::from_str(&contents) {
            Ok(r) => r,
            // enter a blank lock file if failed (do not exit)
            Err(e) => {
                return Err(AnyError(format!(
                    "failed to parse {} file at path {:?}: {}",
                    IP_MANIFEST_FILE, path, e
                )))?
            }
        };
        // verify there are no duplicate entries between tables
        if let Some(e) = man.is_deps_valid().err() {
            return Err(AnyError(format!(
                "failed to parse {} file at path {:?}: {}",
                IP_MANIFEST_FILE, path, e
            )))?;
        }

        let local_name = man.get_ip().get_name().clone();
        let local_version = man.get_ip().get_version().clone();

        // verify contents of manifest
        for (name, dep) in man.get_deps_list_mut(true, false) {
            if dep.is_relative() == true {
                if dep.as_ip().is_none() {
                    let ip = Ip::relate(
                        dep.as_path().unwrap().clone(),
                        &path.parent().unwrap().to_path_buf(),
                    )?;
                    // verify the ip loaded has the correct version assigned by the user
                    let ip_version = ip.get_man().get_ip().get_version();
                    if version::is_compatible(dep.get_version(), ip_version) == false {
                        return Err(Error::DependencyIpRelativeBadVersion(
                            dep.get_version().clone(),
                            ip_version.clone(),
                        ))?;
                    }
                    // verify the ip loaded has the correct name assigned by the user
                    let ip_name = ip.get_man().get_ip().get_name();
                    if ip_name != name {
                        return Err(Error::DependencyIpRelativeBadName(
                            name.clone(),
                            ip_name.clone(),
                        ))?;
                    }
                    dep.uuid = Some(ip.get_uuid().clone());
                    dep.relative_ip = Some(ip);
                }
            }
            // verify there are no cycles in the ip dependency graph
            if name == &local_name && version::is_compatible(dep.get_version(), &local_version) {
                return Err(Error::CyclicDependencyIp(local_name))?;
            }
        }
        Ok(man)
    }
}

impl FromStr for Manifest {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

impl Manifest {
    /// Establishes a minimal bare [Manifest].
    pub fn new() -> Self {
        Self {
            ip: Package {
                name: PkgPart::new(),
                version: IpVersion::new(),
                uuid: Uuid::new(),
                source: None.into(),
                keywords: Vec::new(),
                description: None,
                channels: None,
                public: None,
                library: None,
                readme: None,
                include: None,
                exclude: None,
                authors: None,
                metadata: HashMap::new(),
            },
            dependencies: Dependencies::new(),
            dev_dependencies: Dependencies::new(),
        }
    }

    /// Returns the library name to be used for HDL. If a library is not specified,
    /// then it chooses "work".
    pub fn get_hdl_library(&self) -> LangIdentifier {
        match self.get_ip().get_library().as_ref() {
            Some(l) => LangIdentifier::Vhdl(Identifier::from(l)),
            // IDEA: or for none -> Identifier::from(self.get_man().get_ip().get_name()),
            // OR: Identifier::new_working()
            None => LangIdentifier::Vhdl(Identifier::from(self.get_ip().get_name())),
        }
    }

    pub fn has_relative_deps(&self) -> bool {
        self.dependencies
            .iter()
            .find(|(_, v)| v.is_relative())
            .is_some()
            || self
                .dev_dependencies
                .iter()
                .find(|(_, v)| v.is_relative())
                .is_some()
    }

    pub fn write_manifest_ref_help() -> String {
        String::from("see more `Orbit.toml` keys and their definitions at https://chaseruskin.github.io/orbit/reference/manifest.html")
    }

    /// Composes a [String] to write to a clean manifest file.
    pub fn write_empty_manifest(name: &IpName, lib: &Option<String>) -> String {
        let uuid = Uuid::new();
        match lib {
            None => {
                format!(
                    r#"[ip]
name = "{}"
version = "0.1.0"
uuid = "{}"

[dependencies]
"#,
                    name, uuid
                )
            }
            Some(lib) => {
                format!(
                    r#"[ip]
name = "{}"
version = "0.1.0"
uuid = "{}"
library = "{}"

[dependencies]
"#,
                    name, uuid, lib
                )
            }
        }
    }

    pub fn get_ip(&self) -> &Package {
        &self.ip
    }

    /// Returns the list of dependencies found only under the "dependencies"
    /// table.
    pub fn get_deps(&self) -> &Dependencies {
        &self.dependencies
    }

    pub fn get_dev_deps(&self) -> &Dependencies {
        &self.dev_dependencies
    }

    pub fn is_deps_valid(&self) -> Result<(), AnyError> {
        for (key, _) in &self.dependencies {
            if let Some(_) = self.dev_dependencies.get(key) {
                return Err(AnyError(format!(
                    "duplicate key '{}' in [dependencies] and [dev-dependencies]",
                    key
                )));
                // println!("{}: Dependency {} is used instead of dev-dependency {}",
                //     "warning".yellow().bold(),
                //     IpSpec::from((key.clone(), val.clone())),
                //     IpSpec::from((key.clone(), rep.clone())),
                // )
            }
        }
        Ok(())
    }

    /// Returns the list of dependencies found under "dependencies" and
    /// "dev-dependencies".
    pub fn get_deps_list(&self, include_dev: bool, ordered: bool) -> Vec<(&PkgPart, &Dependency)> {
        let mut result = Vec::with_capacity(
            self.dependencies.len()
                + match include_dev {
                    true => self.dev_dependencies.len(),
                    false => 0,
                },
        );
        if include_dev == true {
            result.extend(self.dev_dependencies.iter());
        }
        result.extend(self.dependencies.iter());
        if ordered == true {
            result.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());
        }
        result
    }

    /// Returns the list of dependencies found under "dependencies" and
    /// "dev-dependencies".
    pub fn get_deps_list_mut(
        &mut self,
        include_dev: bool,
        ordered: bool,
    ) -> Vec<(&PkgPart, &mut Dependency)> {
        let mut result = Vec::with_capacity(
            self.dependencies.len()
                + match include_dev {
                    true => self.dev_dependencies.len(),
                    false => 0,
                },
        );
        if include_dev == true {
            result.extend(self.dev_dependencies.iter_mut());
        }
        result.extend(self.dependencies.iter_mut());
        if ordered == true {
            result.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());
        }
        result
    }
}

impl Display for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", toml::to_string(&self).unwrap())
    }
}

fn vec_is_empty<T>(field: &Vec<T>) -> bool {
    field.is_empty()
}

fn map_is_empty<K, V>(field: &HashMap<K, V>) -> bool {
    field.is_empty()
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Package {
    name: IpName,
    uuid: Uuid,
    version: IpVersion,
    library: Option<IpName>,
    description: Option<String>,
    authors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "vec_is_empty", default)]
    keywords: Vec<String>,
    /// Describes the URL for fetching the captured state's code (expects .ZIP file)
    #[serde(deserialize_with = "source::string_or_struct", default)]
    source: Source,
    /// Known channels where this ip should be published to
    channels: Option<Vec<String>>,
    /// Filepaths that should be explictly known to the user for ip referencing
    public: Option<Vec<String>>,
    /// Filepaths that should be explicitly included during source code analysis
    include: Option<Vec<String>>,
    /// Filepaths that should be explicitly excluded during source code analysis
    exclude: Option<Vec<String>>,
    /// Filepath to the project's README.
    readme: Option<PathBuf>,
    /// Ignore this field and never use it for any processing
    #[serde(skip_serializing_if = "map_is_empty", default)]
    metadata: HashMap<String, toml::Value>,
}

impl Package {
    pub fn get_name(&self) -> &IpName {
        &self.name
    }

    pub fn get_publics(&self) -> &Option<Vec<String>> {
        &self.public
    }

    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn get_version(&self) -> &IpVersion {
        &self.version
    }

    pub fn get_keywords(&self) -> &Vec<String> {
        &self.keywords
    }

    pub fn get_library(&self) -> &Option<IpName> {
        &self.library
    }

    pub fn get_source(&self) -> Option<&Source> {
        self.source.as_option()
    }

    pub fn get_channels(&self) -> &Option<Vec<String>> {
        &self.channels
    }

    /// Clones into a new [IpSpec] struct.
    pub fn into_ip_spec(&self) -> IpSpec {
        IpSpec::new(
            self.get_name().clone(),
            self.uuid.clone(),
            self.get_version().clone(),
        )
    }

    pub fn get_readme(&self) -> &Option<PathBuf> {
        &self.readme
    }

    pub fn get_include(&self) -> &Option<Vec<String>> {
        &self.include
    }

    pub fn get_exclude(&self) -> &Option<Vec<String>> {
        &self.exclude
    }
}

/// Takes an iterative approach to iterating through directories to find a file
/// matching `name`.
///
/// Note: `name` is become a glob-style pattern.
///
/// Stops descending the directories upon finding first match of `name`.
/// The match must be case-sensitive. If `is_exclusive` is `false`, then the directory
/// with match will continued to be searched at that level and then re-track.
pub fn find_file(path: &PathBuf, name: &str, is_exclusive: bool) -> Result<Vec<PathBuf>, Fault> {
    // create a glob-style pattern
    let pattern = glob::Pattern::new(name).unwrap();
    // list of directories to continue to process
    let mut to_process: Vec<PathBuf> = Vec::new();
    let mut result = Vec::new();
    // start at base path
    if path.is_dir() {
        to_process.push(path.to_path_buf());
    // only look at file and exit
    } else if path.is_file() && pattern.matches(path.file_name().unwrap().to_str().unwrap()) {
        return Ok(vec![path.to_path_buf()]);
    }
    // process next directory to read
    while let Some(entry) = to_process.pop() {
        // needs to look for more clues deeper in the filesystem
        if entry.is_dir() {
            let mut next_to_process = Vec::new();
            let mut found_file = false;
            // iterate through all next-level directories for potential future processing
            for e in std::fs::read_dir(entry)? {
                let e = e?;
                if pattern.matches(e.file_name().to_str().unwrap()) {
                    result.push(e.path());
                    found_file = true;
                    if is_exclusive == true {
                        break;
                    }
                } else if e.file_type().unwrap().is_dir() == true {
                    next_to_process.push(e.path());
                }
            }
            // add next-level directories to process
            if found_file == false {
                to_process.append(&mut next_to_process);
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

    mod deser {
        use super::*;

        #[test]
        fn ut_minimal() {
            let man: Manifest = toml::from_str(EX2).unwrap();

            assert_eq!(man.ip.name, PkgPart::from_str("Lab1").unwrap());
            assert_eq!(man.ip.version, IpVersion::new().major(1));
            assert_eq!(man.ip.get_source(), None);
            assert_eq!(man.dependencies, HashMap::new());
            assert_eq!(man.dev_dependencies, HashMap::new());
        }

        #[test]
        fn ut_complex() {
            let man: Manifest = toml::from_str(EX1).unwrap();

            assert_eq!(man.ip.name, PkgPart::from_str("gates").unwrap());
            assert_eq!(
                man.ip.get_source(),
                Some(
                    &Source::from_str(
                        "https://github.com/ks-tech/gates/archive/refs/tags/0.1.0.zip"
                    )
                    .unwrap()
                )
            );
            assert_eq!(man.dependencies.len(), 1);
            assert_eq!(man.dev_dependencies.len(), 2);
            assert_eq!(man.ip.library, Some(PkgPart::from_str("common").unwrap()));
        }

        #[test]
        fn ut_bad() {
            let man = toml::from_str::<Manifest>(ERR1);

            assert_eq!(man.is_err(), true);
        }

        #[test]
        fn ut_serialize() {
            // @note: keys in an table-array (hashmap) are not guaranteed to be in the same order
            let man: Manifest = toml::from_str(EX3).unwrap();
            let text = man.to_string();
            assert_eq!(text, EX3);
        }

        #[test]
        fn ut_complex_source() {
            let man: Manifest = match toml::from_str(EX4) {
                Ok(m) => m,
                Err(e) => panic!("{}", e.to_string()),
            };

            println!("{}", toml::to_string(&man).unwrap());

            assert_eq!(man.ip.get_source().is_some(), true);
            assert_eq!(
                man.ip.get_source().as_ref().unwrap().get_url(),
                "https://some.url"
            );
            assert_eq!(
                man.ip
                    .get_source()
                    .as_ref()
                    .unwrap()
                    .get_protocol()
                    .as_ref()
                    .unwrap(),
                "ktsp"
            );

            let man: Manifest = match toml::from_str(EX5) {
                Ok(m) => m,
                Err(e) => panic!("{}", e.to_string()),
            };

            assert_eq!(man.ip.get_source().is_some(), true);
            assert_eq!(
                man.ip.get_source().as_ref().unwrap().get_url(),
                "https://some.url"
            );
            assert_eq!(
                man.ip
                    .get_source()
                    .as_ref()
                    .unwrap()
                    .get_protocol()
                    .as_ref(),
                None
            );

            let man: Manifest = match toml::from_str(EX6) {
                Ok(m) => m,
                Err(e) => panic!("{}", e.to_string()),
            };

            assert_eq!(man.ip.get_source().is_some(), true);
            assert_eq!(
                man.ip.get_source().as_ref().unwrap().get_url(),
                "https://some.url"
            );
            assert_eq!(
                man.ip
                    .get_source()
                    .as_ref()
                    .unwrap()
                    .get_protocol()
                    .as_ref(),
                None
            );
        }

        #[test]
        #[should_panic]
        fn ut_source_missing_url() {
            // missing required key "url"
            let _man: Manifest = match toml::from_str(EX7) {
                Ok(m) => m,
                Err(e) => panic!("{}", e.to_string()),
            };
        }
    }
}

const EX1: &str = r#"[ip]
name = "gates"
uuid = "0000000000000000000000000"
version = "0.1.0"
library = "common"
source = "https://github.com/ks-tech/gates/archive/refs/tags/0.1.0.zip"

[ip.metadata]
foo = 1
bar = 2

[ip.metadata.subtable]
foo = "hello world"

[dependencies]
some-package = "10.0.0"

[dev-dependencies]
top-builder = "1.0.0"
my-testing-framework = "0.1.0"


"#;

const EX2: &str = r#"[ip]
name = "Lab1"
uuid = "0000000000000000000000000"
version = "1.0.0"
"#;

const EX3: &str = r#"[ip]
name = "lab2"
uuid = "0000000000000000000000000"
version = "1.20.0"
keywords = ["circuits"]

[dependencies]
some-package = "9.0.0"

[dev-dependencies]
top-builder = "1.0.0"
"#;

const EX4: &str = r#"[ip]
name = "lab2"
uuid = "0000000000000000000000000"
version = "1.20.0"
source = { url = "https://some.url", protocol = "ktsp" }
"#;

const EX5: &str = r#"[ip]
name = "lab2"
uuid = "0000000000000000000000000"
version = "1.20.0"
source = { url = "https://some.url" }
"#;

const EX6: &str = r#"[ip]
name = "lab2"
uuid = "0000000000000000000000000"
version = "1.20.0"
source = "https://some.url"
"#;

const EX7: &str = r#"[ip]
name = "lab2"
uuid = "0000000000000000000000000"
version = "1.20.0"
source = { protocol = "ktsp" }
"#;

const ERR1: &str = r#"[ip]
"#;
