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
    search - browse the ip catalog

SYNOPSIS
    orbit search [options] [<ip>]

DESCRIPTION
    Returns a list of the ip found in the catalog.
    
    By default, all ip in the catalog will be returned. To filter by ip name, use
    the '<ip>' option. To limit the number of results, use the '--limit' option.
    
    An ip can be stored across three different levels: installed in the cache,
    downloaded to the archive, and available via channels. By default, all levels
    are searched for ip. Applying a level filter ('--install', '--download', 
    '--available' options) will restrict the search to only checking the filtered
    levels for ip.
    
    A resulting ip is only read from one level, even when multiple levels are
    searched. When an ip exists at multiple levels, the catalog imposes a priority
    on which level to choose. Installed ip have higher priority over downloaded ip,
    and downloaded ip have higher priority over available ip.
    
    Results can also be filtered by keyword using the '--keyword' option. By
    default, if an ip matches at least one filter then it will be returned in the
    result. To collect only ip that match each presented filter, use the '--match'
    option.
    
    If an ip has a higher version that exists and is not currently installed, then
    an asterisk character "*" will appear next the ip's version. To update the ip
    to the latest version, see the 'install' command.

OPTIONS
    <ip>
        Ip's name

    --install, -i
        Filter ip installed to the cache

    --download, -d
        Filter ip downloaded to the archive

    --available, -a
        Filter ip available via channels

    --keyword <term>...
        Include ip that have this keyword

    --limit <n>
        Maximum number of results to return

    --match
        Return results that pass each filter

EXAMPLES
    orbit search axi
    orbit search --keyword memory --keyword ecc
    orbit search --keyword cdc --limit 20 -i
"#;
