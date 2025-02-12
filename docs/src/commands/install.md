# __orbit install__

## __NAME__

install - store an immutable reference to an ip

## __SYNOPSIS__

```
orbit install [options]
```

## __DESCRIPTION__

This command will place an ip into the cache. By default, the specified version
is the 'latest' released version orbit can identify.

When this command is ran without specifying the <ip> or a source (such as
`--url` or `--path`), it will attempt to install the current working ip, if it
exists.

By default, any dependencies required only for development by the target ip are
omitted from installation. To also install these dependencies, use `--all`.

If a protocol is recognized using `--protocol`, then an optional tag can also 
be supplied to help the protocol with providing any additional information it
may require.

The `--path` command can accept a file system path that is either 1) the root 
directory that contains the manifest file or 2) a zip archive file that when 
uncompressed, has the manifest file at the root directoy.

To remove ip from the catalog, see the `remove` command.

## __OPTIONS__

`<ip>`  
      Ip specification

`--url <url>`  
      Url to install the ip from the internet

`--path <path>`  
      Path to install the ip from local file system

`--protocol <name>`  
      Use a configured protocol to download ip

`--tag <tag>`  
      Unique tag to provide to the protocol

`--force`  
      Install the ip regardless of the cache slot occupancy

`--offline`  
      Skip checking coherency with source

`--list`  
      View available protocols and exit

`--all`  
      Install all dependencies (including development)

## __EXAMPLES__

```
orbit install
orbit install lcd_driver:2.0
orbit install adder:1.0.0 --url https://my.adder/project.zip
orbit install alu:2.3.7 --path ./projects/alu --force
```

