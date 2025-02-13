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
    info - display information about an ip

SYNOPSIS
    orbit info [options] [<ip>]

DESCRIPTION
    Displays various bits of information about a particular ip. If no ip is
    provided, then it displays information related to the local ip.
    
    To display manifest information, no additional options are required.
    
    To display the defined HDL design elements within the ip, use the '--units'
    option. For non-local ip, its protected and private design elements are hidden
    from the results. To display design elements of all visibility levels the
    '--all' option must also be present.
    
    To display the known versions for an ip, use the '--versions' option.

OPTIONS
    <ip>
        Ip specification

    --versions, -v
        Display the list of known versions

    --units, -u
        Display the hdl design elements defined for this ip

    --all, -a
        Include any private or hidden results

EXAMPLES
    orbit info --units
    orbit info gates:1.0.0 -u --all
    orbit info gates --versions
    orbit info gates:1 -v
"#;
