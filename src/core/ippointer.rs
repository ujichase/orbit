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

use super::{
    ip::Ip,
    manifest::{self, Manifest, IP_MANIFEST_FILE},
};
use crate::error::LastError;
use crate::util::anyerror::Fault;
use crate::{core::manifest::FromFile, error::Error};
use std::path::PathBuf;

/// The ip pointer stores the manifest for an ip, to be used to grab the ip from another
/// location not already on the user's local file system.
#[derive(Debug, PartialEq)]
pub struct IpPointer {
    manifest: Manifest,
}

impl IpPointer {
    pub fn decouple(self) -> Manifest {
        self.manifest
    }

    /// Loads an IpPointer struct.
    pub fn read(path: PathBuf) -> Result<Self, Fault> {
        let man_path = path.join(IP_MANIFEST_FILE);
        if man_path.exists() == false || man_path.is_file() == false {
            return Err(Error::IpLoadFailed(LastError(
                "a manifest file does not exist".to_string(),
            )))?;
        }
        let man = Manifest::from_file(&man_path)?;
        Ok(Self { manifest: man })
    }

    /// Finds all Manifest files available in the provided path `path`.
    ///
    /// Errors if on filesystem problems.
    pub fn detect_all(path: &PathBuf) -> Result<Vec<Ip>, Fault> {
        let mut result = Vec::new();
        // walk the directory
        for mut entry in manifest::find_file(&path, IP_MANIFEST_FILE, false)? {
            // remove the manifest file to access the ip's root directory
            entry.pop();
            result.push({
                let ptr = IpPointer::read(entry)?;
                Ip::from(ptr)
            });
        }
        Ok(result)
    }
}
