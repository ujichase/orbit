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
    new - create a new ip

SYNOPSIS
    orbit new [options] <path>

DESCRIPTION
    Creates a new ip at the target directory '<path>'. The path is assumed to not
    already exist. A new directory will be created at the file system destination
    that contains a minimal manifest file.
    
    If no name is supplied, then the ip's name defaults to the final directory name
    taken from '<path>'. Using the '--name' option allows this field to be
    explicitly set.
    
    The newly created manifest file is intended to be edited by the user. See more
    'Orbit.toml' keys and their definitions at:
    
       https://chaseruskin.github.io/orbit/reference/manifest.html
    
    For initializing an already existing project into an ip, see the 'init' 
    command.

OPTIONS
    <path>
        Directory to create for the ip

    --name <name>
        Set the resulting ip's name

    --lib <lib>
        Set the resulting ip's library

EXAMPLES
    orbit new gates
    orbit new eecs/lab1 --name adder
"#;
