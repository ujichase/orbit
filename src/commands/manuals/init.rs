//
//  Copyright (C) 2022-2024  Chase Ruskin
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
    init - initialize an ip from an existing project

SYNOPSIS
    orbit init [options] [<path>]

DESCRIPTION
    Initializes an ip at the file system directory '<path>'. If not path is
    provided, then it defaults to the current working directory. 
    
    If no name is provided, then the resulting ip's name defaults to the 
    directory's name. Using the '--name' option allows the ip's name to be 
    explicitly set.
    
    Under certain circumstances, you may need a new uuid. This situation will be
    uncommon for many users, but nonetheless it exists. To display a new uuid that
    can be copied into an existing manifest, use the '--uuid' option. All other
    options are ignored when this option is present. Keep in mind that an ip's uuid
    is not intended to change over the course of its lifetime.
    
    The newly created manifest file is intended to be edited by the user. See more
    'Orbit.toml' keys and their definitions at:
    
       https://chaseruskin.github.io/orbit/reference/manifest.html
    
    To create a new ip from a non-existing directory, see the 'new' command.

OPTIONS
    <path>
        Directory to initialize

    --name <name>
        Set the resulting ip's name

    --lib <lib>
        Set the resulting ip's library

    --uuid
        Print a new uuid and exit

EXAMPLES
    orbit init
    orbit init projects/gates
    orbit init --name adder
    orbit init --uuid
"#;
