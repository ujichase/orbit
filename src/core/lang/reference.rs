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

use std::{collections::HashSet, fmt::Display};

use super::LangIdentifier;
use std::hash::Hash;
type VhdlIdentifier = super::vhdl::token::Identifier;
type VerilogIdentifier = super::verilog::token::identifier::Identifier;

pub type RefSet = HashSet<CompoundIdentifier>;

/// A `CompoundIdentifier` is a pattern in the code that catches `<library>.<primary-unit>`. We
/// assume the pattern can be found anywhere.
///
/// A special case is just a simple name (1 identifier) when referencing a component name.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct CompoundIdentifier {
    prefix: Option<LangIdentifier>,
    suffix: LangIdentifier,
}

impl Display for CompoundIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.prefix {
            Some(p) => write!(f, "{}.{}", p, self.suffix),
            None => write!(f, "{}", self.suffix),
        }
    }
}

impl CompoundIdentifier {
    pub fn new_vhdl(prefix: VhdlIdentifier, suffix: VhdlIdentifier) -> Self {
        Self {
            prefix: Some(LangIdentifier::Vhdl(prefix)),
            suffix: LangIdentifier::Vhdl(suffix),
        }
    }

    pub fn new(prefix: LangIdentifier, suffix: LangIdentifier) -> Self {
        Self {
            prefix: Some(prefix),
            suffix: suffix,
        }
    }

    pub fn new_verilog(prefix: VerilogIdentifier, suffix: VerilogIdentifier) -> Self {
        Self {
            prefix: Some(LangIdentifier::Verilog(prefix)),
            suffix: LangIdentifier::Verilog(suffix),
        }
    }

    pub fn new_minimal_vhdl(suffix: VhdlIdentifier) -> Self {
        Self {
            prefix: None,
            suffix: LangIdentifier::Vhdl(suffix),
        }
    }

    pub fn new_minimal_verilog(suffix: VerilogIdentifier) -> Self {
        Self {
            prefix: None,
            suffix: LangIdentifier::Verilog(suffix),
        }
    }

    pub fn entity(prefix: VhdlIdentifier, suffix: VhdlIdentifier) -> Self {
        Self {
            prefix: Some(LangIdentifier::Vhdl(prefix)),
            suffix: LangIdentifier::Vhdl(suffix),
        }
    }

    pub fn get_suffix(&self) -> &LangIdentifier {
        &self.suffix
    }

    pub fn get_prefix(&self) -> Option<&LangIdentifier> {
        self.prefix.as_ref()
    }

    /// Checks if the identifiers `prefix` and `suffix` align with the those of
    /// `self`. Ignores checking the `prefix` if self does not have a prefix.
    pub fn is_match(&self, prefix: &LangIdentifier, suffix: &LangIdentifier) -> bool {
        let first_match = match &self.prefix {
            Some(p) => p == prefix,
            None => true,
        };
        first_match && &self.suffix == suffix
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::lang::verilog::token::identifier::Identifier::Basic;
    use crate::core::lang::vhdl::token::identifier::Identifier::Basic as VBasic;
    use crate::core::lang::LangIdentifier::Verilog;
    use crate::core::lang::LangIdentifier::Vhdl;

    #[test]
    fn ut_equal() {
        let vhdl = CompoundIdentifier {
            prefix: None,
            suffix: Vhdl(VBasic("b".to_string())),
        };
        let verilog = CompoundIdentifier {
            prefix: None,
            suffix: Verilog(Basic("b".to_string())),
        };
        assert_eq!(vhdl, verilog);
    }
}
