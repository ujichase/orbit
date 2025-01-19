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

use crate::core::lang::reference::RefSet;

use super::{Identifier, Position};

#[derive(Debug, PartialEq)]
pub struct PackageBody {
    owner: Identifier,
    refs: RefSet,
    pos: Position,
}

impl PackageBody {
    pub fn new(owner: Identifier, refs: RefSet, pos: Position) -> Self {
        Self {
            owner: owner,
            refs: refs,
            pos: pos,
        }
    }
}

impl PackageBody {
    /// Accesses the references for the entity.
    pub fn get_refs(&self) -> &RefSet {
        &self.refs
    }

    pub fn get_owner(&self) -> &Identifier {
        &self.owner
    }

    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    pub fn take_refs(self) -> RefSet {
        self.refs
    }

    /// Accesses the references as mutable for the entity.
    pub fn get_refs_mut(&mut self) -> &mut RefSet {
        &mut self.refs
    }

    pub fn into_refs(self) -> RefSet {
        self.refs
    }
}

impl Display for PackageBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "package body for {}", self.owner)
    }
}
