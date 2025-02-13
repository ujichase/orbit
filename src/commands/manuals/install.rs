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
    install - store an immutable reference to an ip

SYNOPSIS
    orbit install [options]

DESCRIPTION
    This command will place an ip into the cache. By default, the specified version
    is the 'latest' released version orbit can identify.
    
    When this command is ran without specifying the <ip> or a source (such as
    '--url' or '--path'), it will attempt to install the current working ip, if it
    exists.
    
    By default, any dependencies required only for development by the target ip are
    omitted from installation. To also install these dependencies, use '--all'.
    
    If a protocol is recognized using '--protocol', then an optional tag can also 
    be supplied to help the protocol with providing any additional information it
    may require.
    
    The '--path' command can accept a file system path that is either 1) the root 
    directory that contains the manifest file or 2) a zip archive file that when 
    uncompressed, has the manifest file at the root directoy.
    
    To remove ip from the catalog, see the 'remove' command.

OPTIONS
    <ip>
        Ip specification

    --url <url>
        Url to install the ip from the internet

    --path <path>
        Path to install the ip from local file system

    --protocol <name>
        Use a configured protocol to download ip

    --tag <tag>
        Unique tag to provide to the protocol

    --force
        Install the ip regardless of the cache slot occupancy

    --offline
        Skip checking coherency with source

    --list
        View available protocols and exit

    --all
        Install all dependencies (including development)

EXAMPLES
    orbit install
    orbit install lcd_driver:2.0
    orbit install adder:1.0.0 --url https://my.adder/project.zip
    orbit install alu:2.3.7 --path ./projects/alu --force
"#;
