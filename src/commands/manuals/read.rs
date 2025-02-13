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
    read - lookup hdl source code

SYNOPSIS
    orbit read [options] <unit>

DESCRIPTION
    Navigates hdl source code to lookup requested hdl code snippets. Looking up
    hdl source code to see its implementation can help gain a better understanding
    of the code being reused in your current design.
    
    By default, the resulting code is displayed to the console. To write the
    results to a file for improved readability, use the '--save' option. Combining 
    the '--locate' option with the '--save' option will append the line and column
    number of the identified code snippet to the end of the resulting file path.
    
    If no ip is provided by the '--ip' option, then it will assume to search the
    local ip for the provided design unit.
    
    The values for options '--start', '--end', and '--doc' must be valid hdl code. 
    The code is interpreted in the native language of the provided design unit.
    
    The '--doc' option will attempt to find the comments immediately preceding the
    identified code snippet. 
    
    A design unit must visible in order for it to return the respective source
    code. When reading a design unit that exists within the local ip, it can be 
    any visibility. When reading a design unit that exists outside of the
    local ip, its visibility must be "public" or "protected". Design units that 
    are set to "private" visibility are not allowed to be read outside of their
    ip.
    
    Every time this command is called, it attempts to clean the temporary
    directory where it saves resulting files. To keep existing files on the next
    call of this command, use the '--no-clean' option.

OPTIONS
    <unit>
        Read the file for this primary design unit

    --ip <spec>
        Ip specification

    --doc <code>
        Find the preceding comments to the code snippet

    --save
        Write the results to a temporary read-only file

    --start <code>
        Start the lookup after jumping to this code snippet

    --end <code>
        Stop the lookup after finding this code snippet

    --limit <n>
        Maximum number of source code lines to return

    --no-clean
        Do not clean the temporary directory of existing files

    --locate
        Append the line and column number to the resulting file

EXAMPLES
    orbit read and_gate --limit 25
    orbit read math_pkg --ip math --doc "function clog2" --start "package math_pkg"
    orbit read math_pkg --ip math --doc "function flog2p1" --save --locate
"#;
