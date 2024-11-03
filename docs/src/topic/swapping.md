# String Swapping

_String swapping_ is the process of injecting runtime information into specific locations of strings.

This process allows permissible strings to become generic enough to avoid having the user frequently update them with redundant information or accidently recall the incorrect value.

## Details 

String swap works with key-value pairs. When Orbit sees the correct syntax indicating a known key, it will replace the key's contents with its value in its location within the string.

To have a key substituted with its value, use double opening curly brackets `{{` to denote the beginning of a key and double closing curly brackets `}}` to end the key. Whitespace is ignored around the  key within the curly bracket sequences.

When Orbit gets a permissible string, it will parse the characters to check if a key exists and should be swapped with its value. If it finds a valid known key, then it replaces everything from and within the curly bracket sequences with the variable's value. If it cannot find a valid key that matches the name, it leaves that sequence of the string unmodified.

## Permissible strings

Not every string is checked for string swapping. Strings that are not allowed to have string swapping ignore any existing keys in the text, leaving the entire string unmodified.

The following lists the instances when a string is permitted to perform string swapping:

### Manifest files

The string pattern for an ip's `source.url` field is allowed to contain any of the following keys:

- `orbit.ip.name`: The name of the ip being downloaded.
- `orbit.ip.version`: The version of the ip being downloaded.

The string pattern for an ip's `source.tag` field is allowed to contain any of the following keys:

- `orbit.ip.name`: The name of the ip being downloaded.
- `orbit.ip.version`: The version of the ip being downloaded.

### Fileset patterns

The string pattern for a target's fileset configuration is allowed to contain any of the following keys:

- `orbit.top.name`: The top-level design unit name.
- `orbit.tb.name`: The testbench design unit name.
- `orbit.dut.name`: The design-under-test design unit name.
- `orbit.env.*`: Any environment variables loaded from configuration files.

### Protocol arguments

The argument list defined in a protocol's configuration is allowed to contain any of the following keys:

- `orbit.queue`: The directory that Orbit expects the ip temporarily placed immediately after download.
- `orbit.ip.name`: The name of the ip being downloaded.
- `orbit.ip.version`: The version of the ip being downloaded.
- `orbit.ip.source.url`: The URL for the ip being downloaded.
- `orbit.ip.source.protocol`: The protocol specified by the ip being downloaded
- `orbit.ip.source.tag`: The tag (if provided) specified by the ip being downloaded.
- `orbit.env.*`: Any environment variables loaded from configuration files.

### Target arguments

The argument list defined in a target's configuration is allowed to contain any of the following keys:

- `orbit.ip.name`: The name of the local ip.
- `orbit.ip.library`: The HDL library of the local ip.
- `orbit.ip.version`: The version of the local ip.
- `orbit.ip.checksum`: The truncated most recent checksum of the local ip.
- `orbit.env.*`: Any environment variables loaded from configuration files.
- `orbit.top.name`: The top-level design unit name.
- `orbit.tb.name`: The testbench design unit name.
- `orbit.dut.name`: The design-under-test design unit name.


## Example

Consider an ip with the following manifest data:

``` toml
[ip]
name = "foo"
version = "1.2.0"
source = "https://github.com/hyperspace-labs/foo/archive/refs/tags/{{orbit.ip.version}}.zip
```

The `source` field of an ip's manifest is one string that is allowed to string swap. For its string, we specify a key, "orbit.ip.version", by enclosing it in double curly brackets. This tells Orbit that any time it uses this string, it should replace `{{orbit.ip.version}}` with `1.2.0`, the value associated with that key.

By using string swapping, we can reduce the amount of times redundant information has to be maintained, or delay providing information when we may not know the value until runtime.

## Environment variable translation examples

A key recognized by Orbit during string swapping can be an environment variable key. For an environment variable key to be recognized as a key in the context of string swapping, the environment variable key is converted to lowercase with each "_" character replaced by a "." character.

Consider the environment variables set in an Orbit configuration file:

``` toml
[env]
foo = "bar"
github-user = "chaseruskin"
Yilinx_Path = "/Users/chase/fpga/bin/yilinx"
```

This configuration translates to the following variables:

| TOML `[env]` entry | Environment variable | Substitution variable |  
| - | - | - | 
| `foo` | `ORBIT_ENV_FOO` | `orbit.env.foo` |
| `github-user` | `ORBIT_ENV_GITHUB_USER` | `orbit.env.github.user` |  
| `Yilinx_Path` | `ORBIT_ENV_YILINX_PATH` | `orbit.env.yilinx.path` |
