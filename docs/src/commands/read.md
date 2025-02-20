# __orbit read__

## __NAME__

read - lookup hdl source code

## __SYNOPSIS__

```
orbit read [options] <unit>
```

## __DESCRIPTION__

Navigates hdl source code to lookup requested hdl code snippets. Looking up
hdl source code to see its implementation can help gain a better understanding
of the code being reused in your current design.

By default, the resulting code is displayed to the console. To write the
results to a file for improved readability, use the `--save` option. Combining 
the `--locate` option with the `--save` option will append the line and column
number of the identified code snippet to the end of the resulting file path.

If no ip is provided by the `--ip` option, then it will assume to search the
local ip for the provided design unit.

The values for options `--start`, `--end`, and `--doc` must be valid hdl code. 
The code is interpreted in the native language of the provided design unit.

The `--doc` option will attempt to find the comments immediately preceding the
identified code snippet. 

A design unit must visible in order for it to return the respective source
code. When reading a design unit that exists within the local ip, it can be 
any visibility. When reading a design unit that exists outside of the
local ip, its visibility must be "public" or "protected". Design units that 
are set to "private" visibility are not allowed to be read outside of their
ip.

Every time this command is called, it attempts to clean the temporary
directory where it saves resulting files. To keep existing files on the next
call of this command, use the `--no-clean` option.

## __OPTIONS__

`<unit>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Read the file for this primary design unit

`--ip <spec>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Ip specification

`--doc <code>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Find the preceding comments to the code snippet

`--save`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Write the results to a temporary read-only file

`--start <code>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Start the lookup after jumping to this code snippet

`--end <code>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Stop the lookup after finding this code snippet

`--limit <n>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Maximum number of source code lines to return

`--no-clean`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Do not clean the temporary directory of existing files

`--locate`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Append the line and column number to the resulting file

## __EXAMPLES__

```
orbit read and_gate --limit 25
orbit read math_pkg --ip math --doc "function clog2" --start "package math_pkg"
orbit read math_pkg --ip math --doc "function flog2p1" --save --locate
```

