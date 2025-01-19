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

//! A `version` contains numeric values at 3 levels for informing about
//! varying degrees of changes within a project's lifetime.

use crate::error::Error;
use std::cmp::Ordering;
use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::error::Hint;
use crate::util::anyerror::Fault;

type VerNum = u16;

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord, Hash)]
pub struct VerStr(String);

impl FromStr for VerStr {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 0 {
            return Err(VersionError::EmptyLabel);
        }
        let invalid = s.chars().find(|c| {
            c.is_ascii_digit() == false
                && c.is_ascii_lowercase() == false
                && c.is_ascii_uppercase() == false
                && c != &'.'
        });
        match invalid {
            Some(c) => Err(VersionError::InvalidChar(c)),
            None => Ok(Self(s.to_string())),
        }
    }
}

impl Display for VerStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Checks if a partial version `self` umbrellas the full version `ver`.
pub fn is_compatible(pv: &PartialVersion, ver: &Version) -> bool {
    if pv.major != ver.major {
        return false;
    }

    if pv.label.is_none() && ver.label.is_none() {
        ()
    } else if let Some(p_l) = &pv.label {
        if let Some(v_l) = ver.get_label() {
            if p_l != v_l {
                return false;
            }
        }
    }

    match pv.minor {
        Some(m) => {
            if m == ver.minor {
                match pv.micro {
                    Some(p) => p == ver.micro,
                    None => true,
                }
            } else {
                false
            }
        }
        None => true,
    }
}

/// Finds the most compatible version matching `ver` among the possible `space`.
///
/// Errors if no version was found.
pub fn get_target_version<'a>(
    ver: &AnyVersion,
    space: &'a Vec<&Version>,
) -> Result<Version, Fault> {
    // find the specified version for the given ip
    let mut latest_version: Option<&Version> = None;
    space
        .into_iter()
        .filter(|f| match &ver {
            AnyVersion::Specific(v) => crate::core::version::is_compatible(v, f),
            AnyVersion::Latest => true,
        })
        .for_each(|tag| {
            if latest_version.is_none() || *tag > latest_version.as_ref().unwrap() {
                latest_version = Some(tag);
            }
        });
    match latest_version {
        Some(v) => Ok(v.clone()),
        None => Err(Box::new(Error::VersionNotFound(
            ver.clone(),
            Hint::ShowVersions,
        ))),
    }
}

#[derive(Debug, Eq, Hash, Clone, PartialEq, Ord, PartialOrd)]
pub enum AnyVersion {
    Specific(PartialVersion),
    Latest,
}

impl std::fmt::Display for AnyVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Latest => write!(f, "latest"),
            Self::Specific(v) => write!(f, "{}", v),
        }
    }
}

impl std::str::FromStr for AnyVersion {
    type Err = VersionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if crate::util::strcmp::cmp_ascii_ignore_case(s, "latest") {
            Ok(Self::Latest)
        } else {
            Ok(Self::Specific(PartialVersion::from_str(s)?))
        }
    }
}

impl AnyVersion {
    pub fn as_specific(&self) -> Option<&PartialVersion> {
        match self {
            Self::Specific(v) => Some(v),
            _ => None,
        }
    }

    pub fn is_latest(&self) -> bool {
        self == &Self::Latest
    }
}

impl From<&Version> for AnyVersion {
    fn from(value: &Version) -> Self {
        Self::Specific(PartialVersion {
            major: value.get_major(),
            minor: Some(value.get_minor()),
            micro: Some(value.get_micro()),
            label: value.get_label().clone(),
        })
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord, Hash)]
pub struct PartialVersion {
    major: VerNum,
    minor: Option<VerNum>,
    micro: Option<VerNum>,
    label: Option<VerStr>,
}

impl PartialVersion {
    pub fn new() -> Self {
        PartialVersion {
            major: 0,
            minor: None,
            micro: None,
            label: None,
        }
    }

    pub fn major(mut self, m: VerNum) -> Self {
        self.major = m;
        self
    }

    pub fn minor(mut self, m: VerNum) -> Self {
        self.minor = Some(m);
        self
    }

    pub fn micro(mut self, m: VerNum) -> Self {
        self.micro = Some(m);
        self
    }

    pub fn label(mut self, l: Option<VerStr>) -> Self {
        self.label = l;
        self
    }

    /// Checks if `other` version is within the same domain as the `self`.
    ///
    /// This means either version can fulfill the same requirements, where one may
    /// be implicitly higher or stricter.
    pub fn in_domain(&self, other: &PartialVersion) -> bool {
        if self.major != other.major {
            return false;
        }
        if self.minor.is_some()
            && other.minor.is_some()
            && other.minor.unwrap() != self.minor.unwrap()
        {
            return false;
        }

        other.micro.is_none() || self.micro.is_none() || other.micro.unwrap() == self.micro.unwrap()
    }

    /// Returns the highest compatible version from the list of versions.
    ///
    /// Returns `None` if the list is empty or there are zero that meet the
    /// criteria.
    #[cfg(test)]
    pub fn find_highest<'a>(&self, vers: &'a [Version]) -> Option<&'a Version> {
        let mut highest = None;
        vers.iter().for_each(|v| {
            if is_compatible(self, v) == true && (highest.is_none() || highest.unwrap() < v) {
                highest = Some(v);
            }
        });
        highest
    }

    /// Checks if there are 3 specified version numbers.
    #[cfg(test)]
    pub fn is_fully_qualified(&self) -> bool {
        self.minor.is_some() && self.micro.is_some()
    }

    pub fn has_label(&self) -> bool {
        self.label.is_some()
    }

    pub fn as_version(&self) -> Option<Version> {
        Some(
            Version::new()
                .major(self.major)
                .minor(self.minor?)
                .micro(self.micro?)
                .label(self.label.clone()),
        )
    }
}

impl Display for PartialVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.major)?;
        if let Some(m) = self.minor {
            write!(f, ".{}", m)?;
            if let Some(p) = self.micro {
                write!(f, ".{}", p)?;
            }
        }
        if let Some(l) = &self.label {
            write!(f, "-{}", l)?;
        }
        Ok(())
    }
}

impl From<PartialVersion> for Version {
    fn from(pv: PartialVersion) -> Self {
        Self {
            major: pv.major,
            minor: pv.minor.unwrap_or(0),
            micro: pv.micro.unwrap_or(0),
            label: pv.label,
        }
    }
}

impl FromStr for PartialVersion {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use VersionError::*;

        let s = s.trim();
        if s.is_empty() {
            return Err(EmptyVersion);
        }

        // check if there is a label
        let (nums, label) = match s.split_once('-') {
            Some((n, l)) => (n, Some(l)),
            None => (s, None),
        };

        let mut levels = nums.split_terminator('.').map(|p| p.parse::<VerNum>());

        let major = if let Some(v) = levels.next() {
            v?
        } else {
            return Err(VersionError::MissingMajor);
        };
        let minor = if let Some(v) = levels.next() {
            Some(v?)
        } else {
            None
        };
        let micro = if let Some(v) = levels.next() {
            if levels.next().is_some() {
                return Err(VersionError::ExtraLevels(3 + levels.count()));
            }
            Some(v?)
        } else {
            None
        };

        // TODO: handle invalid parses internally to return what level gave invalid digit?
        Ok(PartialVersion {
            major: major,
            minor: minor,
            micro: micro,
            label: if let Some(l) = label {
                if minor.is_none() || micro.is_none() {
                    return Err(VersionError::LabelNeedsFullVersion);
                } else {
                    Some(VerStr::from_str(l)?)
                }
            } else {
                None
            },
        })
    }
}

impl<'de> Deserialize<'de> for PartialVersion {
    fn deserialize<D>(deserializer: D) -> Result<PartialVersion, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = PartialVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a semantic version number")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match PartialVersion::from_str(v) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(de::Error::custom(e)),
                }
            }
        }

        deserializer.deserialize_map(LayerVisitor)
    }
}

impl Serialize for PartialVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, PartialEq, Clone, Ord, Eq, Hash)]
pub struct Version {
    major: VerNum,
    minor: VerNum,
    micro: VerNum,
    label: Option<VerStr>,
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.major == other.major {
            false => self.major.partial_cmp(&other.major),
            true => match self.minor == other.minor {
                false => self.minor.partial_cmp(&other.minor),
                true => match self.micro == other.micro {
                    false => self.micro.partial_cmp(&other.micro),
                    true => {
                        if self.label.is_none() && other.label.is_none() {
                            Some(Ordering::Equal)
                        } else if let Some(sl) = &self.label {
                            if let Some(ol) = &other.label {
                                sl.partial_cmp(&ol)
                            } else {
                                Some(Ordering::Less)
                            }
                        } else {
                            Some(Ordering::Greater)
                        }
                    }
                },
            },
        }
    }
}

use serde::de::{self};
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = Version;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a semantic version number")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match Version::from_str(v) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(de::Error::custom(e)),
                }
            }
        }

        deserializer.deserialize_map(LayerVisitor)
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Version {
    pub fn new() -> Self {
        Version {
            major: 0,
            minor: 0,
            micro: 0,
            label: None,
        }
    }

    /// Increments the `major` level and resets `minor` and `patch` levels.
    pub fn inc_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.micro = 0;
        self.label = None;
    }

    /// Increments the `minor` level and resets the `patch` level.
    pub fn inc_minor(&mut self) {
        self.minor += 1;
        self.micro = 0;
        self.label = None;
    }

    /// Increments the `patch` level and resets no levels.
    pub fn inc_micro(&mut self) {
        self.micro += 1;
        self.label = None;
    }

    pub fn major(mut self, m: VerNum) -> Self {
        self.major = m;
        self
    }

    pub fn minor(mut self, m: VerNum) -> Self {
        self.minor = m;
        self
    }

    pub fn micro(mut self, p: VerNum) -> Self {
        self.micro = p;
        self
    }

    pub fn label(mut self, l: Option<VerStr>) -> Self {
        self.label = l;
        self
    }

    pub fn get_major(&self) -> VerNum {
        self.major
    }

    pub fn get_minor(&self) -> VerNum {
        self.minor
    }

    pub fn get_micro(&self) -> VerNum {
        self.micro
    }

    pub fn get_label(&self) -> &Option<VerStr> {
        &self.label
    }

    pub fn has_label(&self) -> bool {
        self.label.is_some()
    }

    pub fn to_partial_version(&self) -> PartialVersion {
        PartialVersion::new()
            .major(self.major)
            .minor(self.minor)
            .micro(self.micro)
            .label(self.label.clone())
    }
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use VersionError::*;

        let s = s.trim();
        if s.is_empty() {
            return Err(EmptyVersion);
        }
        // check if there is a label
        let (nums, label) = match s.split_once('-') {
            Some((n, l)) => (n, Some(l)),
            None => (s, None),
        };

        let mut levels = nums.split_terminator('.').map(|p| p.parse::<VerNum>());
        // @TODO handle invalid parses internally to return what level gave invalid digit?
        Ok(Version {
            major: if let Some(v) = levels.next() {
                v?
            } else {
                return Err(VersionError::MissingMajor);
            },
            minor: if let Some(v) = levels.next() {
                v?
            } else {
                return Err(VersionError::MissingMinor);
            },
            micro: if let Some(v) = levels.next() {
                if levels.next().is_some() {
                    return Err(VersionError::ExtraLevels(3 + levels.count()));
                }
                v?
            } else {
                return Err(VersionError::MissingMicro);
            },
            label: if let Some(l) = label {
                Some(VerStr::from_str(l)?)
            } else {
                None
            },
        })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}.{}.{}{}",
            self.get_major(),
            self.get_minor(),
            self.get_micro(),
            match self.get_label() {
                Some(l) => format!("-{}", l),
                None => String::new(),
            },
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum VersionError {
    EmptyVersion,
    MissingMajor,
    MissingMinor,
    MissingMicro,
    LabelNeedsFullVersion,
    ExtraLevels(usize),
    InvalidDigit(ParseIntError),
    EmptyLabel,
    InvalidChar(char),
}

impl std::error::Error for VersionError {}

impl Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use VersionError::*;
        match self {
            LabelNeedsFullVersion => write!(
                f,
                "version label requires minor and micro version values to be defined"
            ),
            EmptyVersion => write!(f, "empty version"),
            MissingMajor => write!(f, "missing major number"),
            MissingMinor => write!(f, "missing minor number"),
            MissingMicro => write!(f, "missing micro number"),
            EmptyLabel => write!(f, "empty version label"),
            InvalidChar(c) => write!(f, "invalid character '{}' in version label", c),
            ExtraLevels(l) => write!(f, "too many version positions; found {} expected 3", l),
            InvalidDigit(_) => write!(f, "invalid digit in version"),
        }
    }
}

impl From<ParseIntError> for VersionError {
    fn from(e: ParseIntError) -> Self {
        match e {
            _ => VersionError::InvalidDigit(e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod partial_ver {
        use super::*;

        #[test]
        fn is_compat() {
            let pv = PartialVersion {
                major: 1,
                minor: None,
                micro: None,
                label: None,
            };
            let v = Version {
                major: 1,
                minor: 2,
                micro: 3,
                label: None,
            };
            assert_eq!(is_compatible(&pv, &v), true);

            let v = Version {
                major: 2,
                minor: 1,
                micro: 3,
                label: None,
            };
            assert_eq!(is_compatible(&pv, &v), false);

            let pv = PartialVersion {
                major: 2,
                minor: Some(1),
                micro: None,
                label: None,
            };
            let v = Version {
                major: 2,
                minor: 2,
                micro: 3,
                label: None,
            };
            assert_eq!(is_compatible(&pv, &v), false);

            let v = Version {
                major: 2,
                minor: 1,
                micro: 3,
                label: None,
            };
            assert_eq!(is_compatible(&pv, &v), true);

            let v = Version {
                major: 9,
                minor: 1,
                micro: 3,
                label: None,
            };
            assert_eq!(is_compatible(&pv, &v), false);

            let pv = PartialVersion {
                major: 2,
                minor: Some(1),
                micro: Some(3),
                label: None,
            };
            let v = Version {
                major: 2,
                minor: 1,
                micro: 3,
                label: None,
            };
            assert_eq!(is_compatible(&pv, &v), true);
        }

        #[test]
        fn display() {
            let pv = PartialVersion {
                major: 1,
                minor: None,
                micro: None,
                label: None,
            };
            assert_eq!(pv.to_string(), "1");

            let pv = PartialVersion {
                major: 1,
                minor: Some(2),
                micro: None,
                label: None,
            };
            assert_eq!(pv.to_string(), "1.2");

            let pv = PartialVersion {
                major: 1,
                minor: Some(2),
                micro: Some(3),
                label: None,
            };
            assert_eq!(pv.to_string(), "1.2.3");
        }

        #[test]
        fn find_highest() {
            let pv = PartialVersion {
                major: 1,
                minor: None,
                micro: None,
                label: None,
            };
            let versions = vec![
                Version::new().major(2).minor(1).micro(1),
                Version::new().major(4).minor(2).micro(5),
                Version::new().major(7).minor(9).micro(1),
                Version::new().major(1).minor(2).micro(5),
                Version::new().major(1).minor(3).micro(4),
                Version::new().major(1).minor(0).micro(0),
            ];
            assert_eq!(
                pv.find_highest(&versions),
                Some(&Version::new().major(1).minor(3).micro(4))
            );

            let pv = PartialVersion {
                major: 4,
                minor: Some(3),
                micro: None,
                label: None,
            };
            assert_eq!(pv.find_highest(&versions), None);
        }

        #[test]
        fn from_str() {
            // valid cases
            let v = PartialVersion::from_str("1.2.3").unwrap();
            assert_eq!(
                v,
                PartialVersion {
                    major: 1,
                    minor: Some(2),
                    micro: Some(3),
                    label: None,
                }
            );
            assert_eq!(v.is_fully_qualified(), true);
            let v = PartialVersion::from_str("19.4").unwrap();
            assert_eq!(
                v,
                PartialVersion {
                    major: 19,
                    minor: Some(4),
                    micro: None,
                    label: None,
                }
            );
            assert_eq!(v.is_fully_qualified(), false);
        }
    }

    #[test]
    fn new() {
        let v: Version = Version::new();
        assert_eq!(
            v,
            Version {
                major: 0,
                minor: 0,
                micro: 0,
                label: None,
            }
        );
        let v = v.major(1).minor(2).micro(3);
        assert_eq!(
            v,
            Version {
                major: 1,
                minor: 2,
                micro: 3,
                label: None,
            }
        );
    }

    #[test]
    fn inc() {
        let mut v = Version {
            major: 7,
            minor: 1,
            micro: 19,
            label: None,
        };
        v.inc_major();
        assert_eq!(
            v,
            Version {
                major: 8,
                minor: 0,
                micro: 0,
                label: None,
            }
        );

        let mut v = Version {
            major: 7,
            minor: 1,
            micro: 19,
            label: None,
        };
        v.inc_minor();
        assert_eq!(
            v,
            Version {
                major: 7,
                minor: 2,
                micro: 0,
                label: None,
            }
        );

        let mut v = Version {
            major: 7,
            minor: 1,
            micro: 19,
            label: None,
        };
        v.inc_micro();
        assert_eq!(
            v,
            Version {
                major: 7,
                minor: 1,
                micro: 20,
                label: None,
            }
        );
    }

    #[test]
    fn from_str() {
        // valid cases
        let v = Version::from_str("1.2.3").unwrap();
        assert_eq!(
            v,
            Version {
                major: 1,
                minor: 2,
                micro: 3,
                label: None,
            }
        );
        let v = Version::from_str("19.4.73").unwrap();
        assert_eq!(
            v,
            Version {
                major: 19,
                minor: 4,
                micro: 73,
                label: None,
            }
        );
        let v = Version::from_str("1.256.0").unwrap();
        assert_eq!(
            v,
            Version {
                major: 1,
                minor: 256,
                micro: 0,
                label: None,
            }
        );
        let v = Version::from_str("019.004.073").unwrap();
        assert_eq!(
            v,
            Version {
                major: 19,
                minor: 4,
                micro: 73,
                label: None,
            }
        );
        // invalid cases
        let v = Version::from_str("1.2.");
        assert!(v.is_err());
        let v = Version::from_str("1.abc.7");
        assert!(v.is_err());
        let v = Version::from_str("");
        assert!(v.is_err());
        let v = Version::from_str("1.-4.5");
        assert!(v.is_err());
        let v = Version::from_str("1.4.5.9");
        assert!(v.is_err());
        let v = Version::from_str("1.4.1_5");
        assert!(v.is_err());
    }

    #[test]
    fn cmp() {
        let v0 = Version::new().major(1).minor(2).micro(3);
        let v1 = Version::new().major(1).minor(2).micro(3);
        assert_eq!(v0 == v1, true);
        assert_eq!(v0 != v1, false);
        assert_eq!(v0 > v1, false);
        assert_eq!(v0 < v1, false);
        assert_eq!(v0 >= v1, true);
        assert_eq!(v0 <= v1, true);

        let v1 = Version::new().major(2).minor(2).micro(3);
        assert_eq!(v0 == v1, false);
        assert_eq!(v0 != v1, true);
        assert_eq!(v0 > v1, false);
        assert_eq!(v0 < v1, true);
        assert_eq!(v0 >= v1, false);
        assert_eq!(v0 <= v1, true);

        let v1 = Version::new().major(1).minor(3).micro(3);
        assert_eq!(v0 == v1, false);
        assert_eq!(v0 != v1, true);
        assert_eq!(v0 > v1, false);
        assert_eq!(v0 < v1, true);
        assert_eq!(v0 >= v1, false);
        assert_eq!(v0 <= v1, true);

        let v1 = Version::new().major(1).minor(2).micro(4);
        assert_eq!(v0 == v1, false);
        assert_eq!(v0 != v1, true);
        assert_eq!(v0 > v1, false);
        assert_eq!(v0 < v1, true);
        assert_eq!(v0 >= v1, false);
        assert_eq!(v0 <= v1, true);

        let v1 = Version::new().major(7).minor(80).micro(4);
        assert_eq!(v0 == v1, false);
        assert_eq!(v0 != v1, true);
        assert_eq!(v0 > v1, false);
        assert_eq!(v0 < v1, true);
        assert_eq!(v0 >= v1, false);
        assert_eq!(v0 <= v1, true);
    }

    #[test]
    fn to_str() {
        let v = Version {
            major: 20,
            minor: 4,
            micro: 7,
            label: None,
        };
        assert_eq!(v.to_string(), "20.4.7");
    }

    #[test]
    fn partial_ver_cmp() {
        let v0 = PartialVersion::new().major(1);
        let v1 = PartialVersion::new().major(1).minor(5);
        assert_eq!(v0.in_domain(&v1), true);
        assert_eq!(v1.in_domain(&v0), true);

        let v0 = PartialVersion::new().major(1).minor(1);
        let v1 = PartialVersion::new().major(1).minor(0);
        assert_eq!(v0.in_domain(&v1), false);
        assert_eq!(v1.in_domain(&v0), false);

        let v0 = PartialVersion::new().major(2).minor(1).micro(3);
        let v1 = PartialVersion::new().major(1).minor(0).micro(4);
        assert_eq!(v0.in_domain(&v1), false);
        assert_eq!(v1.in_domain(&v0), false);

        let v0 = PartialVersion::new().major(1).minor(0).micro(4);
        let v1 = PartialVersion::new().major(1).minor(0).micro(4);
        assert_eq!(v0.in_domain(&v1), true);
        assert_eq!(v1.in_domain(&v0), true);

        let v0 = PartialVersion::new().major(1);
        let v1 = PartialVersion::new().major(1).minor(2).micro(4);
        assert_eq!(v0.in_domain(&v1), true);
        assert_eq!(v1.in_domain(&v0), true);
    }
}
