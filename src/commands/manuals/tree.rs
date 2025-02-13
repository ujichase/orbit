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
    tree - show the dependency graph

SYNOPSIS
    orbit tree [options] [<unit>...]

DESCRIPTION
    Shows the hierarchical tree structure of the hardware design starting from a
    root node.
    
    By default, it will try to automatically detect the root node for the 
    local ip. If there is ambiguity in determining what node can be the root, then 
    all root nodes and their respective trees will be displayed. To only display
    the tree of a particular node, use the '<unit>' option.
    
    There are two trees available to view: hdl and ip. By default, the hdl
    dependency graph is displayed. The hdl graph shows the composition of usable 
    entities/modules. To generate this graph, it analyzes each VHDL architecture 
    and ignores Verilog compiler directives. If an unidentified entity is 
    instantiated, it will appear as a leaf in the graph and will be considered as 
    a "black box" denoted by the "?" character next to its position in the tree.
    
    Using the '--format' option can alter how much information is displayed for
    each hdl design unit in the tree composition. By default, only the design
    unit's name is displayed for each unit.
    
    To display the ip dependency graph, use the '--ip' option.
    
    If the tree's character output is not displaying properly, then the tree can
    be displayed using a set of standard ASCII characters with the '--ascii'
    option.

OPTIONS
    <unit>...
        Uppermost hdl unit of the dependency tree

    --format <fmt>
        Determine how to display nodes ('long', 'short')

    --ascii
        Limit the textual tree characters to the 128 ascii set

    --ip
        Switch to the ip dependency graph

EXAMPLES
    orbit tree
    orbit tree top --format long
    orbit tree --ip --ascii
"#;
