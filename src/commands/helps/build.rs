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
pub const HELP: &str = r#"Plan and execute a target.

Usage:
    orbit build [options] [--] [args]...

Options:
    --target, -t <name>   target to execute
    --top <unit>          set the top level design unit
    --plan <format>       set the blueprint file format
    --target-dir <dir>    the relative directory where the target starts
    --command <path>      overwrite the target's command
    --list                view available targets and exit
    --all                 include all hdl files of the working ip
    --fileset <key=glob>...
                          a glob-style pattern identified by name to include in the blueprint
    --force               force the target to execute 
    --no-clean            do not clean the target folder before execution
    --verbose             display the command being executed
    args                  arguments to pass to the target

Use 'orbit help build' to read more about the command."#;
