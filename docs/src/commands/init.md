# __orbit init__

## __NAME__

init - initialize an ip from an existing project

## __SYNOPSIS__

```
orbit init [options] [<path>]
```

## __DESCRIPTION__

Initializes an ip at the file system directory `<path>`. If not path is
provided, then it defaults to the current working directory. 

If no name is provided, then the resulting ip's name defaults to the 
directory's name. Using the `--name` option allows the ip's name to be 
explicitly set.

Under certain circumstances, you may need a new uuid. This situation will be
uncommon for many users, but nonetheless it exists. To display a new uuid that
can be copied into an existing manifest, use the `--uuid` option. All other
options are ignored when this option is present. Keep in mind that an ip's uuid
is not intended to change over the course of its lifetime.

The newly created manifest file is intended to be edited by the user. See more
`Orbit.toml` keys and their definitions at:

   https://chaseruskin.github.io/orbit/reference/manifest.html

To create a new ip from a non-existing directory, see the `new` command.

## __OPTIONS__

`<path>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Directory to initialize

`--name <name>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Set the resulting ip's name

`--lib <lib>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Set the resulting ip's library

`--uuid`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Print a new uuid and exit

## __EXAMPLES__

```
orbit init
orbit init projects/gates
orbit init --name adder
orbit init --uuid
```

