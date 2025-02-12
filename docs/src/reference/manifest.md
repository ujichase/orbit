# Manifest

The `Orbit.toml` file for each ip is called its manifest. It is written in the [TOML](https://toml.io/en/) format. It is maintained by the user and contains metadata that is needed to build the ip. The manifest is read by Orbit to help automatically generate the ip's lock file, `Orbit.lock`.

> __Note:__ The manifest's file name is "Orbit.toml", with respect to case-sensitivity.

Every manifest file consists of the following sections:

- [[ip]](#the-ip-section) - Defines an ip.
    - [name](#the-name-field) - The name of the ip.
    - [uuid](#the-uuid-field) - The universally unique identifier of the ip.
    - [description](#the-description-field) - A short description of the ip.
    - [version](#the-version-field) - The version of the ip.
    - [authors](#the-authors-field) - The authors of the ip.
    - [library](#the-library-field) - The HDL library for the design units within the ip.
    - [keywords](#the-keywords-field) - A list of simple words categorizing the ip.
    - [source](#the-source-field) - The URL for remotely retrieving the ip.
    - [channels](#the-channels-field) - The channels to update when publishing the ip.
    - [public](#the-public-field) - Files to be visible to other ip.
    - [include](#the-include-field) - Files to include during file discovery.
    - [exclude](#the-exclude-field) - Files to exclude during file discovery.
    - [readme](#the-readme-field) - The path to the README file.
    - [[metadata]](#the-metadata-section) - An unchecked section for custom fields.
- [[dependencies]](#the-dependencies-section) - Ip dependencies.
- [[dev-dependencies]](#the-dev-dependencies-section) - Ip dependencies only used for ongoing development.

### The `[ip]` section

The first section in a `Orbit.toml` file is `[ip]`.

``` toml
[ip]
name = "my-ip" # the name of the package
uuid = "ecj831jmc018hhhgl1d4rzgw8" # the universally unique identifier
version = "0.1.0" # the current version
```

The only fields required by Orbit are name, uuid, and version.

### The `name` field

``` toml
[ip]
name = "my-ip"
# ...
```

### The `uuid` field

A random string consisting of 25 characters in base36 encoding (a-z0-9).

``` toml
[ip]
# ...
uuid = "ecj831jmc018hhhgl1d4rzgw8"
```

### The `version` field

``` toml
[ip]
# ...
version = "0.1.0"
```

### The `authors` field

``` toml
[ip]
# ...
authors = ["Duncan Idaho", "Gurney Halleck"]

```

### The `library` field

``` toml
[ip]
# ...
library = "work"
```

### The `description` field

``` toml
[ip]
# ...
description = "A short description of the ip"
```

### The `keywords` field

``` toml
[ip]
# ...
keywords = ["cpu", "risc"]
```

### The `source` field

``` toml
[ip]
# ...
source = "https://github.com/chaseruskin/orbit/archive/refs/tags/1.0.0.zip"
```

``` toml
[ip]
# ...
source = { url = "https://github.com/chaseruskin/orbit.git", protocol = "git", tag = "1.0.0" }
```

The source field's "url" and "tag" support [_string swapping_](./../topic/swapping.md).

### The `channels` field

``` toml
[ip]
# ...
channels = ["hyperspace-labs"]
```

### The `public` field

``` toml
[ip]
# ...
public = ["/rtl"]
```

The `public` field can be used to explicitly specify which files are visible to other ip when being when being referenced as a dependency. The list contains glob-style patterns that conform to .gitignore file semantics, and are always compared relative that ip's root directory.

If no `public` field is present, then all files are implicitly specified as visible (public) to other ip when being referenced as a dependency.

### The `include` field

The `include` field can be used to explicitly specify which files to include during source code analysis.

``` toml
[ip]
# ...
include = ["/rtl"]
```

Using `include` and `exclude` is mutually exclusive; setting `include` will override any value of `exclude`. If `include` and `exclude` are omitted, then all files from the root of the ip will be included.

### The `exclude` field

The `exclude` field can be used to explicitly specify which files to exclude during source code analysis.

``` toml
[ip]
# ...
exclude = ["/deprec"]
```

Using `include` and `exclude` is mutually exclusive; setting `include` will override any value of `exclude`. If `include` and `exclude` are omitted, then all files from the root of the ip will be included.

Files that are always excluded are those found in directories that contain a "CACHEDIR.TAG" file. For example, every target output directory Orbit creates is excluded because they contain this file.

### The `readme` field

``` toml
[ip]
# ...
readme = "README.md"
```

### The `[metadata]` section

Any type of TOML entry is allowed in this section, as Orbit ignores this section.

``` toml
[ip.metadata]
custom-field-1 = true
custom-field-2 = "hello world"
# ...
```

### The `[dependencies]` section

The `[dependencies]` section is a table of direct dependencies required for the current ip.

``` toml
[dependencies]
gates = "1.0.0"
uart = "2.3.1"
```

If the ip has no dependencies, the section can be omitted from the manifest. The ips listed in this section will always be included in the build graph.

### The `[dev-dependencies]` section

The `[dev-dependencies]` section is a table of direct dependencies required for the current ip.

``` toml
[dev-dependencies]
testkit = "1.3.7"
logic-analyzer = "4.8.0"
```

If the ip has no development dependencies, the section can be omitted from the manifest. The ips listed in this section will not be included in the build graph for when this ip is used as a dependency itself.
