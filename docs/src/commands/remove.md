# __orbit remove__

## __NAME__

remove - delete an ip from the catalog

## __SYNOPSIS__

```
orbit remove [options] <ip>
```

## __DESCRIPTION__

Deletes save data for a known ip from the catalog. The ip's data for its
particular version is removed from the catalog's cache and the catalog's
archive.

By default, an interactive prompt will appear to confirm with the user if the 
correct ip is okay to be removed. To skip this interactive prompt and assume
it is correct without confirmation, use the `--force` option.

To add ip to the catalog, see the `install` command.

## __OPTIONS__

`<ip>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Ip specification

`--force`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Skip interactive prompts

`--verbose`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Display where the removal occurs

## __EXAMPLES__

```
orbit remove gates
orbit remove gates:1.0.1 --force
```

