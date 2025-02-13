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
pub const HELP: &str = r#"Store an immutable reference to an ip.

Usage:
    orbit install [options]

Options:
    <ip>                  ip specification
    --url <url>           URL to install the ip from the internet
    --path <path>         path to install the ip from local file system
    --protocol <name>     use a configured protocol to download ip
    --tag <tag>           unique tag to provide to the protocol
    --force               install the ip regardless of the cache slot occupancy
    --offline             skip checking coherency with source
    --list                view available protocols and exit
    --all                 install all dependencies (including development)

Use 'orbit help install' to read more about the command."#;
