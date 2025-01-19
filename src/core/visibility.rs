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

use std::fmt::Display;
use std::path::PathBuf;

use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::util::anyerror::Fault;

/// A Very-Important-Path List (VipList) that follows .gitignore syntax.
#[derive(Debug)]
pub struct VipList {
    inner: Option<Gitignore>,
}

impl Default for VipList {
    fn default() -> Self {
        Self { inner: None }
    }
}

impl VipList {
    pub fn new(root: &PathBuf, list: &Option<Vec<String>>) -> Result<Self, Fault> {
        let plist = match list {
            Some(globs) => {
                let mut builder = GitignoreBuilder::new(&root);
                for g in globs {
                    builder.add_line(None, g)?;
                }
                Some(builder.build()?)
            }
            None => None,
        };

        Ok(Self { inner: plist })
    }

    /// Checks if the given filepath is included. If there is no public list,
    /// then it will always return true.
    pub fn is_included(&self, path: &str) -> bool {
        match &self.inner {
            Some(ig) => ig.matched_path_or_any_parents(path, false).is_ignore(),
            None => true,
        }
    }

    pub fn exists(&self) -> bool {
        self.inner.is_some()
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

impl Visibility {
    pub fn new() -> Self {
        Self::Public
    }

    pub fn is_public(&self) -> bool {
        self == &Self::Public
    }

    pub fn is_protected(&self) -> bool {
        self == &Self::Protected
    }

    pub fn is_private(&self) -> bool {
        self == &Self::Private
    }
}

impl Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Public => "public",
                Self::Protected => "protected",
                Self::Private => "private",
            }
        )
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Public
    }
}
