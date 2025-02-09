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

use crate::commands::helps::new;
use crate::commands::orbit::AnyResult;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::lang::vhdl::token::Identifier;
use crate::core::manifest::{Manifest, IP_MANIFEST_FILE};
use crate::core::pkgid::PkgPart;
use crate::error::{Error, Hint, LastError};
use crate::util::filesystem::Standardize;
use std::borrow::Cow;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

use super::lock::Lock;

#[derive(Debug, PartialEq)]
pub struct New {
    /// Specify where to create the new ip on the local machine.
    path: PathBuf,
    /// Optionally give the name for the ip, by default tries to be the parent folder's name.
    name: Option<PkgPart>,
    /// Optionally set a library for the ip
    library: Option<Identifier>,
}

impl Subcommand<Context> for New {
    fn interpret(cli: &mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(new::HELP))?;
        Ok(Self {
            name: cli.get(Arg::option("name"))?,
            library: cli.get(Arg::option("lib"))?,
            path: cli.require(Arg::positional("path"))?,
        })
    }

    fn execute(self, _: &Context) -> proc::Result {
        // verify we are not already in an ip directory
        {
            // resolve any relative path
            let dest = PathBuf::standardize(self.path.clone());
            if let Some(p) = Context::find_ip_path(&dest) {
                return Err(Error::IpExistsAtPath(p))?;
            }
        }

        // verify the path does not exist
        if self.path.exists() == true {
            // TODO: give user more helpful error message
            // 1. if the manifest already exists at this directory
            // 2. if no manifest already exists at this directory
            return Err(Error::PathAlreadyExists(
                self.path.clone(),
                Hint::InitNotNew,
            ))?;
        }

        let ip_name = Self::extract_name(self.name.as_ref(), &self.path)?;

        match self.create_ip(&ip_name) {
            Ok(r) => Ok(r),
            Err(e) => Err(Error::FailedToCreateNewIp(LastError(e.to_string())))?,
        }
    }
}

impl New {
    /// Determines the final name to use for the ip based on the given `name` and falls back
    /// to the `path`'s file name if not `name` is given.
    pub fn extract_name<'a>(
        name: Option<&'a PkgPart>,
        path: &PathBuf,
    ) -> AnyResult<Cow<'a, PkgPart>> {
        match name {
            Some(n) => Ok(Cow::Borrowed(n)),
            // try to use the path's ending name as the ip name
            None => match path.file_name() {
                Some(fname) => {
                    let s = fname.to_string_lossy();
                    match PkgPart::from_str(s.as_ref()) {
                        Ok(r) => Ok(Cow::Owned(r)),
                        Err(e) => Err(Error::CannotAutoExtractNameFromPath(
                            s.to_string(),
                            LastError(e.to_string()),
                            Hint::IpNameSeparate,
                        ))?,
                    }
                }
                None => Err(Error::MissingFileSystemPathName(
                    path.clone(),
                    Hint::IpNameSeparate,
                ))?,
            },
        }
    }
}

impl New {
    /// Creates a new directory at the given `dest` with a new manifest file.
    fn create_ip(&self, ip: &PkgPart) -> AnyResult<()> {
        // create the directory
        std::fs::create_dir_all(&self.path)?;

        // create the file directly nested within the destination path
        let manifest_path = {
            let mut p = self.path.clone();
            p.push(IP_MANIFEST_FILE);
            p
        };

        let lib_str = match &self.library {
            Some(s) => Some(s.to_string()),
            None => None,
        };

        // create the manifest
        let mut manifest = std::fs::File::create(&manifest_path)?;
        manifest.write_all(Manifest::write_empty_manifest(&ip, &lib_str).as_bytes())?;
        // display the help message
        println!("info: {}", Manifest::write_manifest_ref_help());

        // write the lockfile
        let local_ip = Ip::load(self.path.clone(), true, false)?;
        Lock::write_new_lockfile(&local_ip)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ut_extract_name() {
        let name = None;
        let path = PathBuf::from("gates");
        assert_eq!(
            New::extract_name(name.as_ref(), &path).unwrap().as_ref(),
            &PkgPart::from_str("gates").unwrap()
        );

        let name = Some(PkgPart::from_str("sha256").unwrap());
        let path = PathBuf::from("gates");
        assert_eq!(
            New::extract_name(name.as_ref(), &path).unwrap().as_ref(),
            &PkgPart::from_str("sha256").unwrap()
        );

        let name = None;
        let path = PathBuf::from("./a/long/path/to/project");
        assert_eq!(
            New::extract_name(name.as_ref(), &path).unwrap().as_ref(),
            &PkgPart::from_str("project").unwrap()
        );

        let name = None;
        let path = PathBuf::from("./a/long/path/to/Project/");
        assert_eq!(
            New::extract_name(name.as_ref(), &path).unwrap().as_ref(),
            &PkgPart::from_str("Project").unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn ut_extract_name_no_file_name() {
        let name = None;
        let path = PathBuf::from(".");
        assert_eq!(
            New::extract_name(name.as_ref(), &path).unwrap().as_ref(),
            &PkgPart::from_str("sha256").unwrap()
        );
    }
}
