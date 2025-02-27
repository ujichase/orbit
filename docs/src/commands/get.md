# __orbit get__

## __NAME__

get - fetch an hdl unit for code integration

## __SYNOPSIS__

```
orbit get [options] <unit>
```

## __DESCRIPTION__

Returns hdl code snippets for the provided design unit to be integrated into
the current design. The code snippets are returned in the native hdl
language of the identified design unit. Code snippets are designed to be
copy and pasted from the console to the current design for quick code 
integration.

If an ip is not provided with the `--ip` option, then it will search the local
ip for the requested design unit.

If the design unit is in VHDL with the `--instance` option being used without
the `--component` option, then it will return the direct instantiation code
style (VHDL-93 feature).

Copying unit instantiations into higher-level entities will not 
automatically track source code references across ips. In order to properly
establish source code reference tracking across ips, the local ip's manifest
must have an up to date `[dependencies]` table that lists all the ips from
which it references source code.

An identifier prefix or suffix can be attached to the signal declarations and
the instantiation's port connection signals by using `--signal-prefix` and 
`--signal-suffix` respectively. These optional texts are treated as normal
strings and are not checked for correct hdl coding syntax.

When no output options are specified, this command by default will display
the unit's declaration.

A design unit must visible in order for it to return the respective code
snippets. When fetching a design unit that exists within the local ip, it
can be any visibility. When fetching a design unit that exists outside of the
local ip, its visibility must be "public". Design units that are set to 
"protected" or "private" visibility are not allowed to be referenced across
ips.

Exporting the unit's declaration information can be accomplished by using the
`--json` option. The valid json is unformatted for encouragement to be 
processed by other programs.

By default, the code snippets will be displayed in the design unit's native
hardware description language. To return the code snippets in a particular
language, use the `--language` option. Valid values are "vhdl", "sv", or 
"native".

## __OPTIONS__

`<unit>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Primary design unit identifier

`--ip <spec>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Ip specification

`--json`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Export the unit's information as valid json

`--library, -l`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Display the unit's library declaration

`--component, -c`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Display the unit's declaration

`--signals, -s`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Display the constant and signal declarations

`--instance, -i`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Display the unit's instantiation

`--language <hdl>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Display in the specified language (vhdl, sv, native)

`--architecture, -a`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Display the unit's architectures

`--name <identifier>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Set the instance's identifier

`--signal-prefix <str>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Prepend information to the instance's signals

`--signal-suffix <str>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Append information to the instance's signals

## __EXAMPLES__

```
orbit get and_gate --ip gates:1.0.0 --component
orbit get ram --ip mem:2 -csi
orbit get uart -si --name uart_inst0
orbit get or_gate --ip gates --json
```

