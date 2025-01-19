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

use crate::error::Error;
use serde::de;
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

use crate::util::anyerror::Fault;

const ID_LEN: usize = 25;
const ID_ALPHABET: [char; 36] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord)]
pub struct Uuid {
    inner: uuid::Uuid,
}

impl Uuid {
    pub fn new() -> Self {
        Self {
            inner: uuid::Uuid::new_v4(),
        }
    }

    pub fn nil() -> Self {
        Self {
            inner: uuid::Uuid::nil(),
        }
    }

    pub fn get(&self) -> &uuid::Uuid {
        &self.inner
    }

    /// Encodes the UUID into a base36 string.
    pub fn encode(&self) -> String {
        let uuid25 = uuid25::Uuid25::from_bytes(self.inner.into_bytes());
        uuid25.to_string()
    }

    /// Decodes the UUID from a base36 string.
    pub fn decode(s: &str) -> Result<Self, Fault> {
        if s.len() != ID_LEN {
            return Err(Error::UuidWrongSize(ID_LEN, s.len()))?;
        }
        if let Some(c) = s.chars().find(|p| ID_ALPHABET.contains(p) == false) {
            return Err(Error::UuidInvalidChar(c))?;
        }
        let uuid25 = uuid25::Uuid25::parse_uuid25(s)?;
        // println!("{:?}", uuid25.as_bytes());
        // println!("{}", uuid25);
        // println!("{}", uuid25.as_bytes().len());
        Ok(Self {
            inner: uuid::Uuid::from_bytes(uuid25.to_bytes()),
        })
    }
}

impl FromStr for Uuid {
    type Err = Fault;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::decode(s)
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode())
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = Uuid;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a universal unique identifier")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match Uuid::decode(v) {
                    Ok(r) => Ok(r),
                    Err(e) => Err(de::Error::custom(e)),
                }
            }
        }

        deserializer.deserialize_map(LayerVisitor)
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ut_encode_uuid() {
        let id = Uuid::new();
        println!("{}", id.inner.as_simple().to_string());
        let og_id = id.inner.as_simple().to_string();
        println!("{}", id.encode());
        println!("{}", id.encode().len());
        let s = id.encode();
        let id = Uuid::decode(&s).unwrap();
        println!("{}", id.inner.as_simple().to_string());
        let new_id = id.inner.as_simple().to_string();
        assert_eq!(og_id, new_id);
        // panic!();
    }

    #[test]
    fn ut_encode_nil() {
        let id = Uuid::nil();

        println!("{}", id.inner.as_simple().to_string());
        let og_id = id.inner.as_simple().to_string();
        println!("{}", id.encode());
        println!("{}", id.encode().len());
        let s = id.encode();
        let id = Uuid::decode(&s).unwrap();
        println!("{}", id.inner.as_simple().to_string());
        let new_id = id.inner.as_simple().to_string();
        assert_eq!(og_id, new_id);
        // panic!();
    }

    #[test]
    fn ut_user_given_id() {
        let user_value = "1234839204328092342132431";
        let id = Uuid::from_str(user_value).unwrap();
        assert_eq!(user_value, id.encode());
    }
}
