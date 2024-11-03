# Overview

Orbit is an agile package manager and extensible build tool for HDLs.

![](./../images/architecture2.svg)

## Key concepts

- Orbit manages your ips using a group of file system directories that together make up the __Catalog__. The catalog has 3 levels that store increasingly more information about a particular ip: __Channels__, the __Archive__, and the __Cache__.

- An ip's manifest may be stored in a user-defined channel so that a user can find that ip. Running `orbit install` will download the ip from its defined source found in its channel and create a compressed snapshot of the ip in the archive. Once the compressed snapshot is saved, Orbit will decompress the archived snapshot into an immutable reference of the ip at the cache level. The usage of checksums prevents users from editing ips in the cache.

- Every ip requires a __Manifest__ file, named `Orbit.toml`. This is a simple TOML file maintained by the user. The manifest file documents basic metadata about the ip, like its name and version, as well as the ip's list of direct dependencies.

- An ip saves its world state by storing a __Lockfile__, called `Orbit.lock`, alongside the manifest. A lockfile lists all of the resolved ip dependencies required for the local ip and how to retrieve those ips if necessary again. Running `orbit lock` will build an ip-level graph to resolve the entire ip-level dependency tree and store this information in the lockfile.

- Users customize their experience with Orbit using __Configurations__, which collections of __Targets__, __Settings__, and __Protocols__. All of these items are defined in an Orbit configuration file, called `config.toml`. Configurations allow users to reuse and share their workflows across teams and ips.

- To build (or test) a design within a local ip, Orbit runs a __Build Process__. The build process takes as input the local ip's __Lockfile__, __Source Files__ (hdl code), __Auxiliary Files__ (any other file types needed), and a specified __Target__. Running `orbit build` (or `orbit test`) will enter the build process.

- The build process occurs in 2 stages: the __Planning Stage__ and the __Execution Stage__. During the planning stage, Orbit generates a __Blueprint__, which is a single file that lists all the files required to perform the build. During the execution stage, Orbit calls the specified __Target__'s commmand, which typically reads the previously generated blueprint and processes the files using some user-defined EDA tool. The final output from the build process is typically one or more __Artifacts__, which are one or more files generated from the user-defined EDA tool.

- Publish a new version of an ip when it is ready by posting it to a user-defined channel. This method enables other users who also have that channel configured to seamlessly discover and access that new version of the ip. Running `orbit publish` will run a series of checks and then copy the ip's manifest to the proper location within the specified channel.

## Other notes

- Backend EDA tools and workflows (makefiles, TCL scripts, etc.) are decoupled from ip and are able to be reused across projects by creating targets in the configuration file (`config.toml`).

- Orbit does not require a version control system (VCS). Orbit is intended to work with any VCS (git, mercurial, svn, etc.).

- Orbit solves the namespace collision problem by a variant of name mangling when primary design unit identifiers conflict in the dependency tree (_dynamic symbol transformation_).

- Orbit generates a lockfile (`Orbit.lock`) during the planning stage of the build process. The lockfile saves the entire state such that Orbit can return to this state at a later time or on a different computing system. All necessary data that is required to reproduce the build is stored in the lockfile. The lockfile is maintained by Orbit and should be checked into versionc control.

- Orbit generates a blueprint during the planning stage of the build process. The blueprint is a single file that lists the HDL source code files required for the particular build in topologically sorted order. Targets can also specify other file types to be collected into the blueprint. The blueprint is an artifact to be consumed by the target's process during the exection stage of the build process. Since it can frequently change with each build, it should not be checked into version control.
