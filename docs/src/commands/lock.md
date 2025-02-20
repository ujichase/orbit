# __orbit lock__

## __NAME__

lock - save the world state of an ip

## __SYNOPSIS__

```
orbit lock [options]
```

## __DESCRIPTION__

Saves the state of the world for the local ip. To accomplish this, Orbit reads
the local ip's manifest file, "Orbit.toml", to resolve any missing ip 
dependencies. It writes the information required to reproduce this state to 
the ip's lock file, "Orbit.lock".

A local ip must exist for this command to execute.

It is encouraged to check the lock file into version control such that other
users trying to reconstruct the ip can reproduce the ip's current state. The 
lock file should not be manually edited by the user.

To capture the world state for the local ip, Orbit downloads and installs any
unresolved ip dependencies. If an installed dependency's computed checksum 
does not match the checksum stored in the lock file, it assumes the 
installation is corrupt and will reinstall the dependency to the cache.

## __OPTIONS__

`--force`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Ignore reading the precomputed lock file

## __EXAMPLES__

```
orbit lock
orbit lock --force
```

