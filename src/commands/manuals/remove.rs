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
    remove - delete an ip from the catalog

SYNOPSIS
    orbit remove [options] <ip>

DESCRIPTION
    Deletes save data for a known ip from the catalog. The ip's data for its
    particular version is removed from the catalog's cache and the catalog's
    archive.
    
    By default, an interactive prompt will appear to confirm with the user if the 
    correct ip is okay to be removed. To skip this interactive prompt and assume
    it is correct without confirmation, use the '--force' option.
    
    To add ip to the catalog, see the 'install' command.

OPTIONS
    <ip>
        Ip specification

    --force
        Skip interactive prompts

    --verbose
        Display where the removal occurs

EXAMPLES
    orbit remove gates
    orbit remove gates:1.0.1 --force
"#;
