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

//! File     : pkgid.rs
//! Abstract :
//!     A `pkgid` is formed is a unique string following VLNV format that allows
//!     reference to a particular package/ip.

use serde::{de, Deserialize};
use serde_derive::Serialize;
use std::error::Error;
use std::fmt::{self, Display};
use std::str::FromStr;

#[derive(Debug, PartialOrd, Clone, Eq, Hash, Serialize, Ord)]
#[serde(transparent)]
pub struct PkgPart(String);

impl<'de> Deserialize<'de> for PkgPart {
    fn deserialize<D>(deserializer: D) -> Result<PkgPart, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = PkgPart;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an ip name")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match PkgPart::from_str(v) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(de::Error::custom(e)),
                }
            }
        }

        deserializer.deserialize_map(LayerVisitor)
    }
}

impl PkgPart {
    pub fn new() -> Self {
        PkgPart(String::new())
    }

    /// Normalizes the identifier by converting `-` to `_` and all lowercase letters.
    pub fn to_normal(&self) -> PkgPart {
        PkgPart(self.0.replace('-', "_").to_lowercase())
    }

    pub fn as_ref(&self) -> &str {
        &self.0
    }

    /// Checks if the current [PkgPart] is a superset of the `rhs`.
    pub fn contains(&self, rhs: &Self) -> bool {
        self.to_normal()
            .to_string()
            .contains(rhs.to_normal().as_ref())
    }

    /// Checks if the current [PkgPart] is a superset of the `rhs` starting
    /// from position 0.
    pub fn starts_with(&self, rhs: &Self) -> bool {
        self.to_normal()
            .to_string()
            .starts_with(rhs.to_normal().as_ref())
    }
}

impl AsRef<std::path::Path> for PkgPart {
    fn as_ref(&self) -> &std::path::Path {
        self.0.as_ref()
    }
}

impl From<&PkgPart> for toml_edit::Value {
    fn from(p: &PkgPart) -> Self {
        From::<&String>::from(&p.0)
    }
}

impl std::str::FromStr for PkgPart {
    type Err = PkgIdError;

    /// Verifies a part follows the `PkgId` specification.
    ///
    /// First character must be `alphabetic`. Remaining characters must be
    /// `ascii alphanumeric`, `-`, or `_`.
    fn from_str(s: &str) -> Result<Self, PkgIdError> {
        use PkgIdError::*;

        if let Some(c) = s.chars().next() {
            if c.is_ascii_alphabetic() == false {
                return Err(NotAlphabeticFirst(c));
            }
        }
        // find first char in pkgid part not following spec
        let result = s
            .chars()
            .find(|&c| !c.is_ascii_alphanumeric() && !(c == '_') && !(c == '-'));
        if let Some(r) = result {
            Err(InvalidChar(r))
        } else {
            // verify the last char
            if let Some(c) = s.chars().last() {
                if c == '_' || c == '-' {
                    return Err(InvalidEnding);
                }
            }
            Ok(Self(s.to_owned()))
        }
    }
}

impl std::cmp::PartialEq for PkgPart {
    /// Two `PkgId`'s are considered equivalent if they have identical case
    /// insensitive string parts. Different than `==` operator. Converting '-'
    /// to '_' is also applied.
    fn eq(&self, other: &Self) -> bool {
        self.to_normal().0 == other.to_normal().0
    }

    fn ne(&self, other: &Self) -> bool {
        self.eq(other) == false
    }
}

impl std::fmt::Display for PkgPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct PkgId {
    vendor: Option<PkgPart>,
    library: Option<PkgPart>,
    name: PkgPart,
}

impl Eq for PkgId {}

use std::hash::Hash;

impl Ord for PkgId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let r = self
            .vendor
            .as_ref()
            .unwrap()
            .0
            .cmp(&other.vendor.as_ref().unwrap().0);
        match r {
            std::cmp::Ordering::Equal => {
                let s = self
                    .library
                    .as_ref()
                    .unwrap()
                    .0
                    .cmp(&other.library.as_ref().unwrap().0);
                match s {
                    std::cmp::Ordering::Equal => self.name.0.cmp(&other.name.0),
                    _ => s,
                }
            }
            _ => r,
        }
    }
}

impl Hash for PkgId {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        if let Some(v) = &self.vendor {
            v.to_normal().0.hash(state);
        }
        if let Some(l) = &self.library {
            l.to_normal().0.hash(state);
        }
        self.name.to_normal().0.hash(state);
    }
}

#[derive(Debug, PartialEq)]
pub struct PkgIdIter<'a> {
    inner: &'a PkgId,
    index: usize,
}

impl<'a> Iterator for PkgIdIter<'a> {
    type Item = &'a PkgPart;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        self.index += 1;
        match self.index {
            1 => Some(&self.inner.name),
            2 => self.inner.library.as_ref(),
            3 => self.inner.vendor.as_ref(),
            _ => None,
        }
    }
}

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

impl PkgId {
    pub fn new() -> Self {
        PkgId {
            vendor: None,
            library: None,
            name: PkgPart::new(),
        }
    }

    /// Checks if the `other` `PkgId` fits under current name.
    ///
    /// Assumes `other` is fully qualified.
    pub fn partial_match(&self, other: &PkgId) -> bool {
        if self.name.as_ref().is_empty() == false && self.name != other.name {
            return false;
        }
        if self.library.is_some()
            && self.library.as_ref().unwrap().as_ref().is_empty() == false
            && self.library != other.library
        {
            return false;
        }
        if self.vendor.is_some()
            && self.vendor.as_ref().unwrap().as_ref().is_empty() == false
            && self.vendor != other.vendor
        {
            return false;
        }
        true
    }

    pub fn into_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn iter(&self) -> PkgIdIter {
        PkgIdIter {
            inner: &self,
            index: 0,
        }
    }

    pub fn name(mut self, n: &str) -> Result<Self, PkgIdError> {
        self.name = PkgPart::from_str(n)?;
        Ok(self)
    }

    pub fn library(mut self, l: &str) -> Result<Self, PkgIdError> {
        self.library = Some(PkgPart::from_str(l)?);
        Ok(self)
    }

    pub fn vendor(mut self, v: &str) -> Result<Self, PkgIdError> {
        self.vendor = Some(PkgPart::from_str(v)?);
        Ok(self)
    }

    /// Checks if all the parts for a `PkgId` are specified and nonempty.
    pub fn fully_qualified(&self) -> Result<(), PkgIdError> {
        if self.name.0.len() == 0 {
            Err(PkgIdError::Empty)
        } else if self.library.is_none() || self.library.as_ref().unwrap().0.len() == 0 {
            Err(PkgIdError::MissingLibrary)
        } else if self.vendor.is_none() || self.vendor.as_ref().unwrap().0.len() == 0 {
            Err(PkgIdError::MissingVendor)
        } else {
            Ok(())
        }
    }

    pub fn get_name(&self) -> &PkgPart {
        &self.name
    }

    pub fn get_library(&self) -> &Option<PkgPart> {
        &self.library
    }

    pub fn get_vendor(&self) -> &Option<PkgPart> {
        &self.vendor
    }

    /// Transforms the `PkgId` is a `Vec` of its components.
    pub fn into_vec(self) -> Vec<Option<PkgPart>> {
        vec![Some(self.name), self.library, self.vendor]
    }

    /// Transforms into a complete `Vec`.
    ///
    /// Errors if the PkgId is not fully qualified.
    pub fn into_full_vec(&self) -> Result<Vec<PkgPart>, PkgIdError> {
        self.fully_qualified()?;
        Ok(vec![
            self.name.clone(),
            self.library.as_ref().unwrap().clone(),
            self.vendor.as_ref().unwrap().clone(),
        ])
    }

    /// Borrows the `PkgId` as a complete `Vec`.
    ///
    /// Errors if the PkgId is not fully qualified.
    pub fn as_full_vec(&self) -> Result<Vec<&PkgPart>, PkgIdError> {
        self.fully_qualified()?;
        Ok(vec![
            &self.name,
            self.library.as_ref().unwrap(),
            self.vendor.as_ref().unwrap(),
        ])
    }

    /// Transform a vector of `PkgPart` parts into a `PkgId`.
    pub fn from_vec(mut vec: Vec<PkgPart>) -> Self {
        Self {
            name: vec.remove(0),
            library: Some(vec.remove(0)),
            vendor: Some(vec.remove(0)),
        }
    }
}

impl Display for PkgId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{0}{1}{2}{3}{4}",
            self.vendor.as_ref().unwrap_or(&PkgPart::new()),
            if self.vendor.is_some() { "." } else { "" },
            self.library.as_ref().unwrap_or(&PkgPart::new()),
            if self.library.is_some() { "." } else { "" },
            self.name
        )
    }
}

impl FromStr for PkgId {
    type Err = PkgIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chunks: Vec<&str> = s.rsplit('.').collect();
        if chunks.len() > 3 {
            return Err(PkgIdError::BadLen(s.to_owned(), chunks.len()));
        }

        Ok(PkgId {
            name: if let Some(&n) = chunks.get(0) {
                PkgPart::from_str(n)?
            } else {
                return Err(PkgIdError::Empty);
            },
            library: if let Some(&l) = chunks.get(1) {
                Some(PkgPart::from_str(l)?)
            } else {
                None
            },
            vendor: if let Some(&v) = chunks.get(2) {
                Some(PkgPart::from_str(v)?)
            } else {
                None
            },
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum PkgIdError {
    NotAlphabeticFirst(char),
    BadLen(String, usize),
    Empty,
    InvalidChar(char),
    MissingVendor,
    MissingLibrary,
    InvalidEnding,
}

impl Error for PkgIdError {}

impl Display for PkgIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use PkgIdError::*;
        match self {
            NotAlphabeticFirst(ch) => write!(
                f,
                "expects first character to be alphabetic but found '{}'",
                ch
            ),
            InvalidChar(ch) => write!(
                f,
                "character '{}' is not alphanumeric, a dash, or an underscore",
                ch
            ),
            Empty => write!(f, "cannot be empty"),
            BadLen(id, len) => write!(
                f,
                "bad length for pkgid '{}'; expecting 3 parts but found {}",
                id, len
            ),
            MissingLibrary => write!(f, "missing library part"),
            MissingVendor => write!(f, "missing vendor part"),
            InvalidEnding => write!(f, "expects last character to not be a dash or underscore"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iter() {
        let p1 = PkgId::new()
            .name("NAME")
            .unwrap()
            .library("library")
            .unwrap()
            .vendor("Vendor")
            .unwrap();
        let mut p1_iter = p1.iter();

        assert_eq!(p1_iter.next(), Some(&PkgPart("NAME".to_owned())));
        assert_eq!(p1_iter.next(), Some(&PkgPart("library".to_owned())));
        assert_eq!(p1_iter.next(), Some(&PkgPart("Vendor".to_owned())));
        assert_eq!(p1_iter.next(), None);
    }

    #[test]
    fn from_vec() {
        let parts = vec![
            PkgPart("NAME".to_owned()),
            PkgPart("library".to_owned()),
            PkgPart("Vendor".to_owned()),
        ];
        assert_eq!(
            PkgId::from_vec(parts),
            PkgId::new()
                .name("NAME")
                .unwrap()
                .library("library")
                .unwrap()
                .vendor("Vendor")
                .unwrap()
        );
    }

    #[test]
    fn into_vec() {
        let p1 = PkgId::new()
            .name("NAME")
            .unwrap()
            .library("library")
            .unwrap()
            .vendor("Vendor")
            .unwrap();
        assert_eq!(
            p1.into_full_vec(),
            Ok(vec![
                PkgPart("NAME".to_owned()),
                PkgPart("library".to_owned()),
                PkgPart("Vendor".to_owned()),
            ])
        );

        let p1 = PkgId::new()
            .name("NAME")
            .unwrap()
            .library("library")
            .unwrap()
            .vendor("Vendor")
            .unwrap();
        assert_eq!(
            p1.into_vec(),
            vec![
                Some(PkgPart("NAME".to_owned())),
                Some(PkgPart("library".to_owned())),
                Some(PkgPart("Vendor".to_owned())),
            ]
        );

        let p1 = PkgId::new()
            .name("NAME")
            .unwrap()
            .library("library")
            .unwrap();
        assert_eq!(p1.into_full_vec().is_err(), true);

        let p1 = PkgId::new().name("NAME").unwrap();
        assert_eq!(
            p1.into_vec(),
            vec![Some(PkgPart("NAME".to_owned())), None, None,]
        );
    }

    #[test]
    fn new() {
        let pkgid = PkgId::new();
        assert_eq!(
            pkgid,
            PkgId {
                name: PkgPart::new(),
                library: None,
                vendor: None,
            }
        );

        let pkgid = PkgId::new()
            .name("name")
            .unwrap()
            .library("rary")
            .unwrap()
            .vendor("vendor")
            .unwrap();
        assert_eq!(
            pkgid,
            PkgId {
                name: PkgPart("name".to_owned()),
                library: Some(PkgPart("rary".to_owned())),
                vendor: Some(PkgPart("vendor".to_owned())),
            }
        );

        assert_eq!(pkgid.get_name(), &PkgPart("name".to_owned()));
        assert_eq!(
            pkgid.get_library().as_ref().unwrap(),
            &PkgPart("rary".to_owned())
        );
        assert_eq!(
            pkgid.get_vendor().as_ref().unwrap(),
            &PkgPart("vendor".to_owned())
        );
    }

    #[test]
    fn equivalence() {
        let p1 = PkgId::new()
            .name("NAME")
            .unwrap()
            .library("library")
            .unwrap()
            .vendor("Vendor")
            .unwrap();

        let p2 = PkgId::new()
            .name("name")
            .unwrap()
            .library("LIBRARY")
            .unwrap()
            .vendor("vendoR")
            .unwrap();
        assert_eq!(p1, p2);

        let p2 = PkgId::new()
            .name("name")
            .unwrap()
            .library("library2")
            .unwrap()
            .vendor("vendor")
            .unwrap();
        assert_ne!(p1, p2);

        let p2 = PkgId::new()
            .name("name4")
            .unwrap()
            .library("library")
            .unwrap()
            .vendor("vendor")
            .unwrap();
        assert_ne!(p1, p2);

        let p2 = PkgId::new()
            .name("name")
            .unwrap()
            .library("library")
            .unwrap()
            .vendor("ven_dor")
            .unwrap();
        assert_ne!(p1, p2);

        // Converting '-' to '_' is applied for equivalence
        let p1 = PkgId::new()
            .name("name")
            .unwrap()
            .library("lib_rary")
            .unwrap()
            .vendor("Vendor")
            .unwrap();

        let p2 = PkgId::new()
            .name("name")
            .unwrap()
            .library("lib-rary")
            .unwrap()
            .vendor("vendor")
            .unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn validate() {
        //okays
        let s = "name";
        assert_eq!(PkgPart::from_str(s), Ok(PkgPart(s.to_owned())));
        let s = "NAME_1";
        assert_eq!(PkgPart::from_str(s), Ok(PkgPart(s.to_owned())));
        let s = "NAME_1-0";
        assert_eq!(PkgPart::from_str(s), Ok(PkgPart(s.to_owned())));
        let s = "N9A-ME_1N--A432ME";
        assert_eq!(PkgPart::from_str(s), Ok(PkgPart(s.to_owned())));

        //errors
        assert!(PkgPart::from_str("ven dor").is_err());
        assert!(PkgPart::from_str("2name").is_err());
        assert!(PkgPart::from_str("_name").is_err());
        assert!(PkgPart::from_str("-name").is_err());
        assert!(PkgPart::from_str("path/name").is_err());
        assert!(PkgPart::from_str("na!me").is_err());
    }

    #[test]
    fn fully_qualified() {
        let pkgid = PkgId {
            vendor: Some(PkgPart("vendor".to_owned())),
            library: Some(PkgPart("library".to_owned())),
            name: PkgPart("name".to_owned()),
        };
        assert_eq!(pkgid.fully_qualified().is_ok(), true);

        let pkgid = PkgId {
            vendor: Some(PkgPart("".to_owned())),
            library: Some(PkgPart("library".to_owned())),
            name: PkgPart("name".to_owned()),
        };
        assert_eq!(
            pkgid.fully_qualified().unwrap_err(),
            PkgIdError::MissingVendor
        );

        let pkgid = PkgId {
            vendor: None,
            library: Some(PkgPart("library".to_owned())),
            name: PkgPart("name".to_owned()),
        };
        assert_eq!(
            pkgid.fully_qualified().unwrap_err(),
            PkgIdError::MissingVendor
        );

        let pkgid = PkgId {
            vendor: Some(PkgPart("vendor".to_owned())),
            library: Some(PkgPart("".to_owned())),
            name: PkgPart("name".to_owned()),
        };
        assert_eq!(
            pkgid.fully_qualified().unwrap_err(),
            PkgIdError::MissingLibrary
        );

        let pkgid = PkgId {
            vendor: Some(PkgPart("vendor".to_owned())),
            library: None,
            name: PkgPart("name".to_owned()),
        };
        assert_eq!(
            pkgid.fully_qualified().unwrap_err(),
            PkgIdError::MissingLibrary
        );

        let pkgid = PkgId {
            vendor: Some(PkgPart("vendor".to_owned())),
            library: Some(PkgPart("library".to_owned())),
            name: PkgPart("".to_owned()),
        };
        assert_eq!(pkgid.fully_qualified().unwrap_err(), PkgIdError::Empty);

        let pkgid = PkgId {
            vendor: None,
            library: None,
            name: PkgPart("name".to_owned()),
        };
        assert_eq!(
            pkgid.fully_qualified().unwrap_err(),
            PkgIdError::MissingLibrary
        );
    }

    #[test]
    fn from_str() {
        assert_eq!(
            PkgId::from_str("vendor.library.name"),
            Ok(PkgId {
                vendor: Some(PkgPart("vendor".to_owned())),
                library: Some(PkgPart("library".to_owned())),
                name: PkgPart("name".to_owned()),
            })
        );

        assert_eq!(
            PkgId::from_str("library.name"),
            Ok(PkgId {
                vendor: None,
                library: Some(PkgPart("library".to_owned())),
                name: PkgPart("name".to_owned()),
            })
        );

        assert_eq!(
            PkgId::from_str("name"),
            Ok(PkgId {
                vendor: None,
                library: None,
                name: PkgPart("name".to_owned()),
            })
        );

        assert_eq!(
            PkgId::from_str(".name"),
            Ok(PkgId {
                vendor: None,
                library: Some(PkgPart("".to_owned())),
                name: PkgPart("name".to_owned()),
            })
        );

        assert_eq!(
            PkgId::from_str("..name"),
            Ok(PkgId {
                vendor: Some(PkgPart("".to_owned())),
                library: Some(PkgPart("".to_owned())),
                name: PkgPart("name".to_owned()),
            })
        );

        assert_eq!(
            PkgId::from_str("Ven-dor.Lib_Rary.name10"),
            Ok(PkgId {
                vendor: Some(PkgPart("Ven-dor".to_owned())),
                library: Some(PkgPart("Lib_Rary".to_owned())),
                name: PkgPart("name10".to_owned()),
            })
        );

        // invalid pkgids
        assert!(PkgId::from_str("vendor.library.name.").is_err());
        assert!(PkgId::from_str(".vendor.library.name").is_err());
        assert!(PkgId::from_str("vendor?.library.name").is_err());
        assert!(PkgId::from_str("vendor.library.name.extra").is_err());
        assert!(PkgId::from_str("0vendor.library.name").is_err());
        assert!(PkgId::from_str("vendor.0library.name").is_err());
        assert!(PkgId::from_str("vendor.library.0name").is_err());
        assert!(PkgId::from_str("vendor.library.name=").is_err());
        assert!(PkgId::from_str("vendor.lib rary.name").is_err());

        assert!(PkgId::from_str("v$ndor.library.name").is_err());
        assert!(PkgId::from_str("vendor.l*brary.name").is_err());
        assert!(PkgId::from_str("vendor.lib/rary.name").is_err());
        assert!(PkgId::from_str("vendor.lib'rary.name").is_err());
        assert!(PkgId::from_str("vendor.lib!rary.name").is_err());
        assert!(PkgId::from_str("vendor/library/name").is_err());
    }

    #[test]
    fn to_string() {
        let p1 = PkgId::from_str("gates").unwrap();
        assert_eq!(p1.to_string(), "gates");
        let p1 = PkgId::from_str("rary.gates").unwrap();
        assert_eq!(p1.to_string(), "rary.gates");
    }

    #[test]
    fn partial_match() {
        let p1 = PkgId::from_str("ks-tech.rary.gates").unwrap();
        // only by name
        let p2 = PkgId::from_str("gates").unwrap();
        assert_eq!(p2.partial_match(&p1), true);

        // not equal
        let p2 = PkgId::from_str("gate").unwrap();
        assert_eq!(p2.partial_match(&p1), false);

        // only by vendor
        let p2 = PkgId::from_str("ks-tech..").unwrap();
        assert_eq!(p2.partial_match(&p1), true);

        // only by library
        let p2 = PkgId::from_str(".rary.").unwrap();
        assert_eq!(p2.partial_match(&p1), true);

        // full pkgid
        let p2 = PkgId::from_str("ks-tech.rary.gates").unwrap();
        assert_eq!(p2.partial_match(&p1), true);

        // vendor idenifier in wrong position
        let p2 = PkgId::from_str("ks-tech").unwrap();
        assert_eq!(p2.partial_match(&p1), false);
    }
}
