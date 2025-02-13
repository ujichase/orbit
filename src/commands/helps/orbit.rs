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
pub const HELP: &str = r#"Orbit is an hdl package manager and build tool.

Usage:
    orbit [options] [command]

Commands:
    new                   create a new ip
    init                  initialize an ip from an existing project
    info                  display information about an ip
    read                  lookup hdl source code
    get                   fetch an hdl unit for code integration
    tree                  show the dependency graph
    lock                  save the world state of an ip
    test, t               run a test
    build, b              plan and execute a target
    publish               post an ip to a channel
    search                browse the ip catalog
    install               store an immutable reference to an ip
    remove                delete an ip from the catalog
    env                   print orbit environment information
    config                modify configuration data

Options:
    --version             print version information and exit
    --upgrade             check for the latest orbit binary
    --license             print license information and exit
    --sync                synchronize configured channels
    --force               bypass interactive prompts
    --color <when>        coloring: auto, always, never
    --help, -h            print help information

Use 'orbit help <command>' for more information about a command."#;
