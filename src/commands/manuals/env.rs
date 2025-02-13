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
    env - print orbit environment information

SYNOPSIS
    orbit env [options]

DESCRIPTION
    Displays environment variables as key-value pairs related to Orbit.
    
    By default, this command prints information as a shell script. If one or more
    variable names are given as arguments using '<key>', then it will print the 
    value of each provided variable on its own line.
    
    Environment information can change based on where the command is executed.
    
    Environment variables that are known only at runtime are not displayed. Be
    sure to review the documentation for a list of all environment variables set 
    by Orbit.

OPTIONS
    <key>...
        Display this variable's value

EXAMPLES
    orbit env
    orbit env ORBIT_HOME
    orbit env ORBIT_MANIFEST_DIR ORBIT_IP_NAME
"#;
