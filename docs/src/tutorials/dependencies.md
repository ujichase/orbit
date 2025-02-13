# Dependencies: Half adder

In this tutorial, you will learn how to:

[1.](#referencing-external-ips) Specify an external ip as a dependency  
[2.](#learning-about-ips) Use Orbit to learn more about external ips and their design units  
[3.](#integrating-design-units-across-ips) Leverage Orbit across ips to integrate an entity into a separate ip  

## Referencing external ips

After completing the gates project from the previous tutorial ahead of schedule, you take a well deserved vacation. Now you have returned to work and are tasked with building a half adder.

Let's create a new project. Navigate to a directory in your file system where you would like to store the project.
```
$ orbit new half-add --lib adding
```
For the rest of this tutorial, we will be working relative to the project directory "/half-add" that was created by the previous command.

Remembering our impressive work with the gates project, we realize we can reuse some of the already designed and tested components from there. Let's make sure it's installed so that we can use it.
```
$ orbit search gates
```
```
gates                       0.1.0       install
```
Awesome! Our next step is tell Orbit that our current project, half-add, wants to use gates as a dependency.

Add a new entry for gates to the dependencies table in our project's manifest, Orbit.toml.

Filename: Orbit.toml
``` toml
[ip]
name = "half-add"
library = "adding"
version = "0.1.0"

# See more keys and their definitions at https://chaseruskin.github.io/orbit/reference/manifest.html

[dependencies]
gates = "0.1.0"
```

We've referenced it, now we have to use it!

## Learning about ips

Your memory is a little foggy on what gates actually did, and what entities were available. Luckily, we can query for information through Orbit about ips and their design units.

Let's remember what entities we have at our disposal.

```
$ orbit info gates --units
```
``` 
and_gate                            entity        public 
nand_gate                           entity        public 
```

Okay, how did we implement the NAND gate architecture?
```
$ orbit read --ip gates nand_gate --start architecture
```
```
architecture rtl of nand_gate is
begin

  x <= a nand b;

end architecture;
```
Cool, we had used the VHDL keyword `nand` to describe that particular circuit. Sometimes it may be insightful to read code snippets and comments from external design units when trying to integrate them into a new project.

## Integrating design units across ips

Let's use the NAND gate we previously defined to construct a half adder circuit.
```
$ orbit get --ip gates nand_gate --library --signals --instance
```
```
library gates;

signal a : std_logic;
signal b : std_logic;
signal x : std_logic;

uX : entity gates.nand_gate
  port map(
    a => a,
    b => b,
    x => x
  );
```

A half adder can be constructed with 5 NAND gates. It's time to copy/paste our NAND gate instances into our new file "half_add.vhd".

Filename: half_add.vhd
``` vhdl
library ieee;
use ieee.std_logic_1164.all;

library gates;

entity half_add is
  port(
    a, b : in std_logic;
    c, s : out std_logic
  );
end entity;

architecture rtl of half_add is
  
  signal x4, x1, x2 : std_logic;

begin

  -- 1st layer: This gate creates the first NAND intermediate output.
  u4 : entity gates.nand_gate
    port map(
      a => a,
      b => b,
      x => x4
    );
  
  -- 2nd layer: Perform NAND with input 'a' and the 1st layer's output.
  u1 : entity gates.nand_gate
    port map(
      a => a,
      b => x4,
      x => x1
    );

  -- 2nd layer: Perform NAND with input 'b' and the 1st layer's output.
  u2 : entity gates.nand_gate
    port map(
      a => x4,
      b => b,
      x => x2
    );

  -- 3rd layer: This gate produces the final sum signal ('a' XOR 'b').
  u3 : entity gates.nand_gate
    port map(
      a => x1,
      b => x2,
      x => s
    );

  -- 3rd layer: This gate produces the final carry out signal ('a' AND 'b').
  u5 : entity gates.nand_gate
    port map(
      a => x4,
      b => x4,
      x => c
    );

end architecture;
```

Let's inspect the design hierarchy to make sure our circuit and its components are identified by Orbit.
```
$ orbit tree --format long
```
```
half_add (half-add:0.1.0)
└─ nand_gate (gates:0.1.0)
```

Finally, let's install this ip to the cache for future reuse as well. But before we can install any ip to our cache, an ip must have an up to date lockfile.

Lockfiles are updated whenever a user calls `orbit build` or `orbit test`, but they can also be updated with the dedicated `orbit lock` command. Let's go ahead and generate the lockfile now.
```
$ orbit lock
```

Now we can safely install the ip to our catalog.
```
$ orbit install
```

Nice, now we have successfully reused designs across ips! However, maybe we should have designed all the logic gates in the gates ip...

### Additional notes on dependencies

Before integrating a design unit from an external ip into a separate project, it's important to first update the Orbit.toml file. This manifest file has a dependencies section, which allows you to tell Orbit which ips to bring into the current project scope. Without the ips in scope, Orbit may be unable to identify where you got a reference for a particular design unit. Orbit denotes an unknown design unit with a ? when displaying the design hierarchy.
```
half_add (half-add:0.1.0)
└─ nand_gate ?
```

After introducing dependencies at the project level, it's also important to maintain an up-to-date lockfile, Orbit.lock. In most cases, Orbit will automatically generate it when it needs it, however, you as the user can also explicitly request Orbit to update the lockfile.
```
$ orbit lock
```

The lockfile saves information for Orbit to use later when needing to reconstruct the state of an ip. This includes saving information about all ip dependencies, their checksums, and potential sources of retrieval. Remember, the Orbit.lock file is automatically managed by Orbit and does not require direct user editing.