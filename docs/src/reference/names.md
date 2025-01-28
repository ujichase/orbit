# Names

In order to identify an ip among others, Orbit requires users to assign a human-readable _name_ to each created ip. In addition to the human-readable name, Orbit assigns a _universally unique identifier_ (UUID) to each ip. UUIDs are required in order to avoid namespace collisions at the ip level in Orbit's decentralized system.

## Name 

The ip _name_ is a unique string of characters that abides by a certain set of rules. It is a single name and is defined under the "name" field in an ip's manifest. Every ip is required to have a name. The name should not change over the course of an ip's lifetime.

``` toml
[ip]
name = "cpu"
# ...
```

### Rules

The following rules currently apply to a name values:
- begins with an ASCII letter (`a-z`, `A-Z`)
- contains only ASCII letters (`a-z`, `A-Z`), ASCII digits (`0-9`), dashes `-`, and underscores `_`
- cannot end with a dash `-` or underscore `_`

## UUID 

The ip _uuid_ is a unique string of characters encoded in base36 (a-z0-9). An encoded UUID is 25 characters long and is generated using Version 4 UUID. It is defined under the "uuid" field in an ip's manifest. Every ip is required to have a uuid. The uuid should not change over the course of an ip's lifetime.

``` toml
[ip]
# ...
uuid = "71vs0nyo7lqjji6p6uzfviaoi"
```

### Rules

The following rules currently apply to uuid values:
- contains only ASCII lowercase letters (`a-z`) and ASCII digits (`0-9`)
- is 25 characters long

## Version 

The ip _version_ is a series of 3 numbers separated by decimal characters (`.`) with an optional label suffix attached with a dash character (`-`). The version should be updated over the course of an ip's lifetime when significant enough changes to the project require a new version value.

``` toml
[ip]
# ...
version = "1.0.0"
```

### Rules

The following rules currently apply to version values:
- contains only ASCII digits (`0-9`) for each of the 3 version fields
- each version field is separated by a decimal character (`.`)
- a label can be attached to the end by using a dash character (`-`)
- labels can only contain ASCII letters (`a-z`, `A-Z`), ASCII digits (`0-9`), and decimal characters (`.`)

## Ip specification

An ip specification, commonly abbreviated to _spec_, is the total unambiguous reference to a specific ip at a particular version.

```
spec ::= <name>[+uuid][:version]
```

### Example specifications

The following provides various valid inputs when defining an ip's spec and how it decomposes into its parts.

Spec          | Name | UUID | Version |         
--------------|------    |-| --------|            
`gates:1.0.0`   |`gates` | Automatically resolved if only 1 ip exists with the name `gates`|  `1.0.0`   |    
`ram`           |`ram`   | Automatically resolved if only 1 ip exists with the name `ram`| `latest`   |
`fifo:2.3`      |`fifo`  | Automatically resolved if only 1 ip exists with the name `fifo` | `2.3.*`    |  
`cpu+71vs0nyo7lqjji6p6uzfviaoi:1.0.0` | `cpu` | `71vs0nyo7lqjji6p6uzfviaoi` | `1.0.0` |

### Namespace Collisions

Two different ip's may share a common name within the catalog even though their contents are different. Two names are considered equal if their lowercase mapping is identical, where dashes (`-`) also map to underscores (`_`).

Spec 1        | Spec 2  | Collision |         
--------------|---------|-----------|            
`gates`         |`GATES`    |true       |    
`ram`           |`rom`      |false      |
`fifo_cdc`      |`Fifo-CDC` |true       | 

To resolve namespace collisions at the ip level, Orbit uses UUIDs. When there are multiple ips in the catalog that share the same name, a user must then explicitly include the UUID of the requested ip to disambiguate between ips under the same name.

## Libraries

An ip can optionally belong to a library. An ip's _library_ is a higher-level scope that loosely groups together multiple ips. This library identification is used for grouping the HDL source code itself into their language-defined libraries as well.

A library can be defined through the "library" field in the ip's manifest file. Its format follows the same rules as the ip's name. If no library is defined in the ip's manifest, then the default library is the ip's name.

An ip is _not_ allowed to have "work" explicitly set as its library in the ip's manifest. The "work" library is a reserved library within the context of VHDL used to reference other primary design units within that of the current library.
