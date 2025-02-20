# __orbit publish__

## __NAME__

publish - post an ip to a channel

## __SYNOPSIS__

```
orbit publish [options]
```

## __DESCRIPTION__

Performs a series of checks for a local ip and then releases it to its
specified channel(s).

There are multiple checks that are performed before an ip can be published. 
First, the ip must have an up to date lockfile with no relative dependencies. 
The ip's manifest must also have a value for the source field. In addition,
Orbit must be able to construct the hdl source code graph without errors.
Finally, the ip is downloaded from its source url and temporarily installed
to verify its contents match those of the local ip.

Posting an ip to a channel involves copying the ip's manifest file to a path 
within the channel known as the index. For every publish of an ip, the index 
corresponds to a unique path within the channel that gets created by Orbit.
A channel's pre-publish and post-publish hooks can get the value for the ip's 
index by reading the ORBIT_IP_INDEX environment variable.

By default, this command performs a dry run, which executes all of the steps 
in the process except for actually posting the ip to its channel(s). 
To run the command to completion, use the `--ready` option.

## __OPTIONS__

`--ready, -y`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Run the operation to completion

`--no-install`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Do not install the ip for future use

`--list`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; View available channels and exit

## __EXAMPLES__

```
orbit publish
orbit publish --ready
```

