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

use super::{packagebody::PackageBody, Generics, Identifier, Position};

#[derive(Debug, PartialEq)]
pub struct Package {
    name: Identifier,
    generics: Generics,
    body: Option<PackageBody>,
    refs: RefSet,
    pos: Position,
}

impl Package {
    pub fn new(name: Identifier, refs: RefSet, pos: Position) -> Self {
        Self {
            name: name,
            generics: Generics::new(),
            body: None,
            refs: refs,
            pos: pos,
        }
    }

    pub fn generics(mut self, generics: Generics) -> Self {
        self.generics = generics;
        self
    }

    pub fn body(mut self, body: Option<PackageBody>) -> Self {
        self.body = body;
        self
    }
}

impl Package {
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    /// Accesses the references for the entity.
    pub fn get_refs(&self) -> &RefSet {
        &self.refs
    }

    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    /// Accesses the references as mutable for the entity.
    pub fn get_refs_mut(&mut self) -> &mut RefSet {
        &mut self.refs
    }

    pub fn into_refs(self) -> RefSet {
        self.refs
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
