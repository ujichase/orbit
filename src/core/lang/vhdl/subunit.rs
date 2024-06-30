use crate::core::lang::reference::{CompoundIdentifier, RefSet};

use super::{symbols, token::identifier::Identifier};

#[derive(Debug, PartialEq)]
pub enum SubUnit {
    Configuration(symbols::configuration::Configuration),
    Architecture(symbols::architecture::Architecture),
    PackageBody(symbols::packagebody::PackageBody),
}

impl SubUnit {
    pub fn from_arch(arch: symbols::architecture::Architecture) -> Self {
        Self::Architecture(arch)
    }

    pub fn from_config(cfg: symbols::configuration::Configuration) -> Self {
        Self::Configuration(cfg)
    }

    pub fn from_body(body: symbols::packagebody::PackageBody) -> Self {
        Self::PackageBody(body)
    }

    pub fn into_refs(self) -> RefSet {
        match self {
            Self::Architecture(u) => u.into_refs(),
            Self::Configuration(u) => u.into_refs(),
            Self::PackageBody(u) => u.into_refs(),
        }
    }

    /// Returns an ordered list of compound indentifiers for consist graph building.
    pub fn get_edge_list(&self) -> Vec<&CompoundIdentifier> {
        let mut list = Vec::with_capacity(self.get_refs().len());
        self.get_refs().iter().for_each(|f| {
            list.push(f);
        });
        list.extend(self.get_edge_list_entities());
        list.sort();
        list
    }

    /// Returns the list of compound identifiers that were parsed from entity instantiations.
    pub fn get_edge_list_entities(&self) -> Vec<&CompoundIdentifier> {
        let mut list = match self {
            Self::Architecture(arch) => arch.get_deps().into_iter().collect(),
            _ => Vec::new(),
        };
        list.sort();
        list
    }

    pub fn get_entity(&self) -> &Identifier {
        match self {
            Self::Architecture(u) => u.entity(),
            Self::Configuration(u) => u.entity(),
            Self::PackageBody(u) => u.get_owner(),
        }
    }

    pub fn get_refs(&self) -> &RefSet {
        match self {
            Self::Architecture(u) => u.get_refs(),
            Self::Configuration(u) => u.get_refs(),
            Self::PackageBody(u) => u.get_refs(),
        }
    }

    pub fn get_refs_mut(&mut self) -> &mut RefSet {
        match self {
            Self::Architecture(u) => u.get_refs_mut(),
            Self::Configuration(u) => u.get_refs_mut(),
            Self::PackageBody(u) => u.get_refs_mut(),
        }
    }
}
