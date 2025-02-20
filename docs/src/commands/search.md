# __orbit search__

## __NAME__

search - browse the ip catalog

## __SYNOPSIS__

```
orbit search [options] [<ip>]
```

## __DESCRIPTION__

Returns a list of the ip found in the catalog.

By default, all ip in the catalog will be returned. To filter by ip name, use
the `<ip>` option. To limit the number of results, use the `--limit` option.

An ip can be stored across three different levels: installed in the cache,
downloaded to the archive, and available via channels. By default, all levels
are searched for ip. Applying a level filter (`--install`, `--download`, 
`--available` options) will restrict the search to only checking the filtered
levels for ip.

A resulting ip is only read from one level, even when multiple levels are
searched. When an ip exists at multiple levels, the catalog imposes a priority
on which level to choose. Installed ip have higher priority over downloaded ip,
and downloaded ip have higher priority over available ip.

Results can also be filtered by keyword using the `--keyword` option. By
default, if an ip matches at least one filter then it will be returned in the
result. To collect only ip that match each presented filter, use the `--match`
option.

If an ip has a higher version that exists and is not currently installed, then
an asterisk character "*" will appear next the ip's version. To update the ip
to the latest version, see the `install` command.

## __OPTIONS__

`<ip>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Ip's name

`--install, -i`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Filter ip installed to the cache

`--download, -d`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Filter ip downloaded to the archive

`--available, -a`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Filter ip available via channels

`--keyword <term>...`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Include ip that have this keyword

`--limit <n>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Maximum number of results to return

`--match`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Return results that pass each filter

## __EXAMPLES__

```
orbit search axi
orbit search --keyword memory --keyword ecc
orbit search --keyword cdc --limit 20 -i
```

