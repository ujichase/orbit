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

// Automatically generated from the mansync.py script.
pub const HELP: &str = r#"Lookup hdl source code.

Usage:
    orbit read [options] <unit>

Arguments:
    <unit>                read the file for this primary design unit

Options:
    --ip <spec>           ip specification
    --doc <code>          find the preceding comments to the code snippet
    --save                write the results to a temporary read-only file
    --start <code>        start the lookup after jumping to this code snippet
    --end <code>          stop the lookup after finding this code snippet
    --limit <n>           maximum number of source code lines to return
    --no-clean            do not clean the temporary directory of existing files
    --locate              append the line and column number to the resulting file

Use 'orbit help read' to read more about the command."#;
