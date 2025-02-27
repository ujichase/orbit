# Configuration

The `config.toml` file stores settings and extends Orbit's functionality. It is written in the [TOML](https://toml.io/en/) format. It is maintained by the developer and can be shared across teams for consistent development environments.

> __Note:__ The configuration's file name is "config.toml", with respect to case-sensitivity.

## Paths

When a field is expected to be a file system path, Orbit has the ability to resolve relative paths. The path is determined in relation to the currently processed `config.toml`'s parent directory. This design choice was implemented in order to allow for path definitions to be valid across developer machines when sharing configurations. It is recommended to use relative paths when setting a path to a field in a `config.toml`.

## Precedence

Orbit supports multiple levels of configuration. Each level has its own order of precedence. The order of precedence is the following:

1. Local configuration file (current ip's `.orbit/conifg.toml`)

2. Regional configuration files (parent directories of the current working directory)

3. Global configuration file (Orbit's `$ORBIT_HOME/config.toml`)

4. Included configuration files (order-preserving list found for the global configuration's [`include`](#the-include-field) field)

The configuration files are processed in the order defined above. When a configuration file defines a field, no other configuration files later in the process will be able to override its value. If a field is never provided an explicit value, the Orbit's default will be used.

> __Tip:__ You can modify some values in the configuration file through the command-line by using the `orbit config` command.

Every configuration file consists of the following sections:

- [include](#the-include-field) - Lists other `config.toml` files to process. This field is only allowed for the global configuration file.
- [[general]](#the-general-section) - The general settings.
    - [target-dir](#the-target-dir-field) - Default target directory.
- [[test]](#the-test-section) - The test settings.
    - [default-target](#the-default-target-field) - Set the default target for tests.
- [[build]](#the-build-section) - The build settings.
    - [default-target](#the-default-target-field) - Set the default target for builds.
- [[vhdl-format]](#the-vhdl-format-section) - VHDL code formatting.
- [[verilog-format]](#the-verilog-format-section) - SystemVerilog/Verilog code formatting.
- [[env]](#the-env-section) - The runtime environment variables.
- [[[target]]](#the-target-array) - Define a target.
    - [name](#the-name-field) - The name of the target.
    - [description](#the-description-field) - A short description of the target.
    - [command](#the-command-field) - The command to execute the target.
    - [args](#the-args-field) - Arguments to pass to the command.
    - [plans](#the-plans-field) - The list of supported blueprint file formats.
    - [[fileset]](#the-fileset-section) - Filesets to collect for the target. 
- [[[protocol]]](#the-protocol-array) - Define a protocol.
    - [name](#the-name-field) - The name of the protocol.
    - [description](#the-description-field) - A short description of the protocol.
    - [command](#the-command-field) - The command to execute the protocol.
    - [args](#the-args-field) - Arguments to pass to the command.
- [[[channel]]](#the-channel-array) - Define a channel.
    - [name](#the-name-field) - The name of the channel.
    - [description](#the-description-field) - A short description of the channel.
    - [root](#the-root-field) - The directory where the channel exists.
    - [sync.command](#the-command-field) - The command to execute when synchronizing the channel.
    - [sync.args](#the-args-field) - Arguments to pass to the command during synchronization.
    - [pre.command](#the-command-field) - The command to execute immediately before launch.
    - [pre.args](#the-command-field) - Arguments to pass to the command immediately before launch.
    - [post.command](#the-command-field) - The command to execute immediately after launch.
    - [post.args](#the-args-field) - Arguments to pass to the command immediately after launch.


### The `include` field

``` toml
include = [
    "profiles/p1/config.toml",
    "profiles/p2/config.toml",
    "channels/c1/config.toml"
]
```

### The `[general]` section

### The `target-dir` field

Define the default output directory to create for the planning and building phases. This value can be overridden on the command-line when the `--target-dir` option is available. When this field is not defined, the default value for the build directory is "target".

``` toml
[general]
target-dir = "target"
# ...
```

### The `[test]` section

### The `default-target` field

Sets the default target when calling `orbit test`. If the default target is set to be used and it cannot be found among the known targets, it will error.

``` toml
[test]
default-target = "foo"
```

### The `[build]` section

### The `default-target` field

Sets the default target when calling `orbit build`. If the default target is set to be used and it cannot be found among the known targets, it will error.

``` toml
[build]
default-target = "bar"
```

### The `[vhdl-format]` section

The currently supported entries are demonstrated in the following code snippet. Entries not present will be set to their default values.

``` toml
[vhdl-format]
# enable colored output for VHDL code snippets
highlight-syntax = true
# number of whitespace characters per tab/indentation
tab-size = 2
# insert a tab before 'generic' and 'port' interface declarations
indent-interface = true
# automatically align a signal or constant's subtype with its other identifiers
type-auto-alignment = true
# number of whitespace characters after alignment (before the `:` character)
type-offset = 1
# automatically align an instantiation's mapping along its port connections
mapping-auto-alignment = true
# number of whitespace characters before port connection (before the `=>` character)
mapping-offset = 1
# place a space between generic/port keyword and its opening parenthesis
space-interface-parenthesis = true
# the default instance name
instance-name = "ux"
```

### The `[verilog-format]` section

The currently supported entries are demonstrated in the following code snippet. Entries not present will be set to their default values. This section currently applies its settings to SystemVerilog and Verilog source code.

``` toml
[verilog-format]
# enable colored output for code snippets
highlight-syntax = true
# number of whitespace characters per tab/indentation
tab-size = 2
# automatically align a port or parameter's name with the module's other names
name-auto-alignment = true
# number of additional whitespace characters after alignment
name-offset = 0
# number of whitespaces before a range specifier
range-offset = 1
# automatically align an instantiation's mapping along its port connections
mapping-auto-alignment = true
# number of whitespace characters before port connection (before the `(` character)
mapping-offset = 0
# the default instance name
instance-name = "ux"
```

### The `[env]` section

The user can define an arbitrary number of their own entries with their determined value represented in string format.

``` toml
[env]
foo = "0" # Accessible as ORBIT_ENV_FOO
super-bar = "1" # Accessible as ORBIT_ENV_SUPER_BAR
```

### The `[[target]]` array

### The `name` field

``` toml
[[target]]
name = "dump-blueprint"
```

### The `description` field

``` toml
[[target]]
# ...
description = "Print the blueprint contents to the screen"
```

### The `command` field

``` toml
[[target]]
# ...
command = "cat"
```

### The `args` field

``` toml
[[target]]
# ...
args = ["blueprint.tsv"]
```

### The `plans` field

``` toml
[[target]]
# ...
plans = ["tsv"]
```

The type of blueprint files supported by the particular target. If a list is provided, the default plan used is the first item in the list. If a plan is provided on the command-line, then it must be a valid plan and found within the target's defined list.

If this field is left blank or not defined, then the default plan is "tsv".

### The `[fileset]` section

``` toml
[[target]]
# ...
fileset.pymdl = "{{ orbit.bench }}.py"
```

### The `[[protocol]]` array

### The `name` field

See [[target]](#the-target-array)'s definition.

### The `description` field

See [[target]](#the-target-array)'s definition.

### The `command` field

See [[target]](#the-target-array)'s definition.

### The `args` field

See [[target]](#the-target-array)'s definition.

### The `[[channel]]` array

### The `name` field

See [[target]](#the-target-array)'s definition.

### The `description` field

See [[target]](#the-target-array)'s definition.

### The `root` field

The file system path where the channel exists, relative to the configuration file where it is defined.

``` toml
[[channel]]
# ...
root = "./index"
```

### The `sync.command` field

See [[target]](#the-target-array)'s definition.

### The `sync.args` field

See [[target]](#the-target-array)'s definition.

### The `pre.command` field

See [[target]](#the-target-array)'s definition.

### The `pre.args` field

See [[target]](#the-target-array)'s definition.

### The `post.command` field

See [[target]](#the-target-array)'s definition.

### The `post.args` field

See [[target]](#the-target-array)'s definition.