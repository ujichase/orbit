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

// This manual page was automatically generated from the mangen.py tool.
pub const MANUAL: &str = r#"NAME
    lock - save the world state of an ip

SYNOPSIS
    orbit lock [options]

DESCRIPTION
    Saves the state of the world for the local ip. To accomplish this, Orbit reads
    the local ip's manifest file, "Orbit.toml", to resolve any missing ip 
    dependencies. It writes the information required to reproduce this state to 
    the ip's lock file, "Orbit.lock".
    
    A local ip must exist for this command to execute.
    
    It is encouraged to check the lock file into version control such that other
    users trying to reconstruct the ip can reproduce the ip's current state. The 
    lock file should not be manually edited by the user.
    
    To capture the world state for the local ip, Orbit downloads and installs any
    unresolved ip dependencies. If an installed dependency's computed checksum 
    does not match the checksum stored in the lock file, it assumes the 
    installation is corrupt and will reinstall the dependency to the cache.

OPTIONS
    --force
        Ignore reading the precomputed lock file

EXAMPLES
    orbit lock
    orbit lock --force
"#;
