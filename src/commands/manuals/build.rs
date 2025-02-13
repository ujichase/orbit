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
    build - plan and execute a target

SYNOPSIS
    orbit build [options] [--] [args]...

DESCRIPTION
    This command prepares a given target and then executes the target.
    
    While this command functions similar to 'orbit test', the targets that are 
    encouraged to be used with this command are ones that produce artifacts at the
    end of their execution process. This command does not allow the top to be a
    testbench, if you want to set a testbench, see 'orbit test'.
    
    A target must be provided for the build command to run. A default target can
    be specified in a configuration file, which will be used when a target is
    omitted from the command-line.
    
    If '--list' is used, then it will display a list of the available targets to
    the user. Using '--list' in combination with a target from '--target' will
    display any detailed help information the target has documented in its 
    definition.
    
    A target typically goes through three steps for the building process:  
       1. Parse the blueprint file  
       2. Process the referenced files listed in the blueprint  
       3. Generate a artifact(s)
    
    Any command-line arguments entered after the terminating flag '--' will be
    passed in the received order as arguments to the subprocess's command. If a 
    target already has defined arguments, the additional arguments passed from the 
    command-line will follow the previously defined arguments.
    
    The target's process will spawn from the current working ip's output directory,
    which is $ORBIT_TARGET_DIR/$ORBIT_TARGET.

OPTIONS
    --target, -t <name>
        Target to execute

    --top <unit>
        Set the top level design unit

    --plan <format>
        Set the blueprint file format

    --target-dir <dir>
        The relative directory where the target starts

    --command <path>
        Overwrite the target's command

    --list
        View available targets and exit

    --all
        Include all hdl files of the working ip

    --fileset <key=glob>...
        A glob-style pattern identified by name to include in the blueprint

    --force
        Force the target to execute 

    --no-clean
        Do not clean the target folder before execution

    --verbose
        Display the command being executed

    args
        Arguments to pass to the target

EXAMPLES
    orbit build --target xsim -- --elab
    orbit build --command python3 --target pysim
    orbit build --all --target-dir build --target ghdl
    orbit build --target xsim --force -- --help
"#;
