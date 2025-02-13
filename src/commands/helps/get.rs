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
pub const HELP: &str = r#"Fetch an hdl unit for code integration.

Usage:
    orbit get [options] <unit>

Arguments:
    <unit>                primary design unit identifier

Options:
    --ip <spec>           ip specification
    --json                export the unit's information as valid json
    --library, -l         display the unit's library declaration
    --component, -c       display the unit's declaration
    --signals, -s         display the constant and signal declarations
    --instance, -i        display the unit's instantiation
    --architecture, -a    display the unit's architectures
    --name <identifier>   set the instance's identifier
    --signal-prefix <str>
                          prepend information to the instance's signals
    --signal-suffix <str>
                          append information to the instance's signals

Use 'orbit help get' to read more about the command."#;
