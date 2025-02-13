# __orbit info__

## __NAME__

info - display information about an ip

## __SYNOPSIS__

```
orbit info [options] [<ip>]
```

## __DESCRIPTION__

Displays various bits of information about a particular ip. If no ip is
provided, then it displays information related to the local ip.

To display manifest information, no additional options are required.

To display the defined HDL design elements within the ip, use the `--units`
option. For non-local ip, its protected and private design elements are hidden
from the results. To display design elements of all visibility levels the
`--all` option must also be present.

To display the known versions for an ip, use the `--versions` option.

## __OPTIONS__

`<ip>`  
      Ip specification

`--versions, -v`  
      Display the list of known versions

`--units, -u`  
      Display the hdl design elements defined for this ip

`--all, -a`  
      Include any private or hidden results

## __EXAMPLES__

```
orbit info --units
orbit info gates:1.0.0 -u --all
orbit info gates --versions
orbit info gates:1 -v
```

