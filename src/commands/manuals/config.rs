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
    config - modify configuration data

SYNOPSIS
    orbit config [options] [<path>]

DESCRIPTION
    Provides an entry point to the current configuration data through the
    command-line.
    
    To list the configuration files that are currently being used, use the
    '--list' option. The configuration files are sorted in order from highest
    precedence to lowest precedence. This means values that are set in files
    higher in the list overwrite values that may have existed from files lowering
    in the list.
    
    Providing the path of a configuration file using the '<path>' option will
    limit the accessible data to only the data found in the file. If no path is 
    specified, then it will display the aggregated result of the current
    configuration data from across all files in use.
    
    If there are no options set to modify data, then the resulting configuration
    data will be displayed.
    
    To modify a field, the full key must be provided. Fields located inside
    tables require decimal characters "." to delimit between the key names. Each 
    modified field is edited in the configuration file has the lowest precedence
    and would allow the changes to take effect. Files that won't be edited are
    configuration files that are included in the global config file. If the
    field does not exist in any configuration level, then the field will be
    modified at in the global config file.
    
    When modifying data, additions are processed before deletions. This means all
    '--push' options occur before '--pop' options, and all '--set' options occur 
    before '--unset' options. Not every configuration field can be edited through 
    the command-line. More complex fields may require manual edits by opening its
    respective file.

OPTIONS
    <path>
        The destination to read/write configuration data

    --push <key=value>...
        Add a new value to a key's list

    --pop <key>...
        Remove the last value from a key's list

    --set <key=value>...
        Store the value as the key's entry

    --unset <key>...
        Delete the key's entry

    --list
        Print the list of configuration files and exit

EXAMPLES
    orbit config --push include="profiles/hyperspacelab"
    orbit config ~/.orbit/config.toml --unset env.vivado_path
"#;
