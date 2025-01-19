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

use super::{Identifier, Position};
use crate::core::lang::reference::RefSet;
use serde_derive::Serialize;

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(transparent)]
pub struct Architecture {
    name: Identifier,
    #[serde(skip_serializing)]
    owner: Identifier,
    #[serde(skip_serializing)]
    refs: RefSet,
    #[serde(skip_serializing)]
    deps: RefSet,
    #[serde(skip_serializing)]
    pos: Position,
}

impl Architecture {
    pub fn new(
        name: Identifier,
        owner: Identifier,
        refs: RefSet,
        deps: RefSet,
        pos: Position,
    ) -> Self {
        Self {
            name: name,
            owner: owner,
            refs: refs,
            deps: deps,
            pos: pos,
        }
    }
}

impl Architecture {
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    pub fn get_owner(&self) -> &Identifier {
        &self.owner
    }

    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    pub fn entity(&self) -> &Identifier {
        &self.owner
    }

    pub fn edges(&self) -> &RefSet {
        &self.refs
    }

    pub fn get_deps(&self) -> &RefSet {
        &self.deps
    }

    /// Accesses the references for the entity.
    pub fn get_refs(&self) -> &RefSet {
        &self.refs
    }

    /// Accesses the references as mutable for the entity.
    pub fn get_refs_mut(&mut self) -> &mut RefSet {
        &mut self.refs
    }

    pub fn into_refs(self) -> RefSet {
        self.refs
    }
}
