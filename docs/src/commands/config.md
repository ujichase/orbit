# __orbit config__

## __NAME__

config - modify configuration data

## __SYNOPSIS__

```
orbit config [options] [<path>]
```

## __DESCRIPTION__

Provides an entry point to the current configuration data through the
command-line.

To list the configuration files that are currently being used, use the
`--list` option. The configuration files are sorted in order from highest
precedence to lowest precedence. This means values that are set in files
higher in the list overwrite values that may have existed from files lowering
in the list.

Providing the path of a configuration file using the `<path>` option will
limit the accessible data to only the data found in the file. If no path is 
specified, then it will display the aggregated result of the current
configuration data from across all files in use.

If there are no options set to modify data, then the resulting configuration
data will be displayed.

To modify a field, the full key must be provided. Fields located inside
tables require decimal characters "." to delimit between the key names. Each 
modified field is edited in the configuration file has the lowest precedence
and would allow the changes to take effect. Files that won't be edited are
configuration files that are included in the global config file. If the
field does not exist in any configuration level, then the field will be
modified at in the global config file.

When modifying data, additions are processed before deletions. This means all
`--push` options occur before `--pop` options, and all `--set` options occur 
before `--unset` options. Not every configuration field can be edited through 
the command-line. More complex fields may require manual edits by opening its
respective file.

## __OPTIONS__

`<path>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; The destination to read/write configuration data

`--push <key=value>...`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Add a new value to a key's list

`--pop <key>...`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Remove the last value from a key's list

`--set <key=value>...`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Store the value as the key's entry

`--unset <key>...`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Delete the key's entry

`--list`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Print the list of configuration files and exit

## __EXAMPLES__

```
orbit config --push include="profiles/hyperspacelab"
orbit config ~/.orbit/config.toml --unset env.vivado_path
```

