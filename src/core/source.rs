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

use super::swap::{self, StrSwapTable};
use crate::util::anyerror::AnyError;
use serde::ser::SerializeMap;
use serde_derive::Deserialize;
use std::str::FromStr;

/// A [Source] outlines the process and location for extracting packages from the internet.
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    protocol: Option<String>,
    url: String,
    /// A `tag`is optional user-defined information that is needed to proceed with the protocol.
    tag: Option<String>,
    // Valid is triggered true when built with a function other than "default".
    #[serde(skip, default = "set_true")]
    valid: bool,
}

fn set_true() -> bool {
    true
}

impl Source {
    pub fn protocol(mut self, p: Option<String>) -> Self {
        self.protocol = p;
        self
    }

    pub fn url(mut self, url: String) -> Self {
        self.url = url;
        self
    }

    pub fn tag(mut self, tag: Option<String>) -> Self {
        self.tag = tag;
        self
    }

    pub fn new() -> Self {
        Self {
            protocol: None,
            url: String::new(),
            valid: true,
            tag: None,
        }
    }

    pub fn get_protocol(&self) -> &Option<String> {
        &self.protocol
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_tag(&self) -> &Option<String> {
        &self.tag
    }

    pub fn is_default(&self) -> bool {
        self.protocol.is_none()
    }

    pub fn as_option(&self) -> Option<&Source> {
        match &self.valid {
            true => Some(&self),
            false => None,
        }
    }

    pub fn replace_vars_in_url(mut self, vtable: &StrSwapTable) -> Self {
        self.url = swap::substitute(self.url, vtable);
        self
    }

    pub fn replace_vars_in_tag(mut self, vtable: &StrSwapTable) -> Self {
        self.tag = match self.tag {
            Some(t) => Some(swap::substitute(t, vtable)),
            None => None,
        };
        self
    }
}

impl From<Option<Source>> for Source {
    fn from(value: Option<Source>) -> Self {
        match value {
            Some(s) => s,
            None => Source::default(),
        }
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.protocol {
            Some(p) => {
                write!(f, "{}+{}", p, self.url)
            }
            None => {
                write!(f, "{}", self.url)
            }
        }
    }
}

impl Default for Source {
    fn default() -> Self {
        Self {
            protocol: None,
            url: String::new(),
            valid: false,
            tag: None,
        }
    }
}

impl FromStr for Source {
    type Err = AnyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            url: s.to_string(),
            protocol: None,
            tag: None,
            valid: true,
        })
    }
}

use serde::de::{self};
use serde::de::{MapAccess, Visitor};
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;

pub fn string_or_struct<'de, D>(deserializer: D) -> Result<Source, D::Error>
where
    D: de::Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct LayerVisitor;

    impl<'de> Visitor<'de> for LayerVisitor {
        type Value = Source;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<Source, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<Source, M::Error>
        where
            M: MapAccess<'de>,
        {
            // falls back on the derived version of deser for the [Source] struct
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(LayerVisitor)
}

impl Serialize for Source {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // check if needing to serialize
        match &self.valid {
            true => {
                // // serializer.serialize_str(&self.to_string()),
                let mut map = match self.get_protocol() {
                    Some(_) => serializer.serialize_map(Some(2)),
                    None => serializer.serialize_map(Some(1)),
                }?;

                map.serialize_entry("url", self.get_url())?;
                if let Some(p) = self.get_protocol() {
                    map.serialize_entry("protocol", p)?;
                }
                if let Some(p) = self.get_tag() {
                    map.serialize_entry("tag", p)?;
                }
                map.end()
            }
            false => serializer.serialize_none(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_str() {
        let src: &str = "https://some.url";

        assert_eq!(
            Source::from_str(src).unwrap(),
            Source {
                protocol: None,
                tag: None,
                url: String::from("https://some.url"),
                valid: true,
            }
        );
    }

    #[test]
    fn deser_struct() {
        let src: Source = match toml::from_str(EX1) {
            Ok(r) => r,
            Err(e) => panic!("{}", e.to_string()),
        };

        assert_eq!(src.is_valid(), true);
    }

    const EX1: &str = r#"url = "https://some.url"
protocol = "ktsp""#;
}
