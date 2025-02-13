# First Project: Gates

In this tutorial, you will learn how to:

[1.](#creating-an-ip) Create an ip from scratch  
[2.](#integrating-design-units) Use Orbit to integrate an entity into a larger design  
[3.](#building-an-ip-for-a-scripted-workflow) Build a design using a simple target  
[4.](#making-an-ip-and-its-design-units-reusable) Release a version of an ip

## Creating an ip

First, navigate to a directory in your file system where you would like to store the project. From there, let's issue our first `orbit` command:
```
$ orbit new gates
```

A directory called "gates" should now exist and look like the following tree structure:
```
gates/
├─ Orbit.lock
└─ Orbit.toml
```

Let's create our first design unit for describing a NAND gate. Feel free to copy the following code into a file called "nand_gate.vhd" that exists in our project directory "/gates".

Filename: nand_gate.vhd
``` vhdl
library ieee;
use ieee.std_logic_1164.all;

entity nand_gate is
  port(
    a, b : in std_logic;
    x : out std_logic
  );
end entity;

architecture rtl of nand_gate is
begin

  x <= a nand b;

end architecture;
```

## Integrating design units

Consider for an instant that our HDL only supports the `nand` keyword and is missing the other logic gates such as `or`, `and`, and `xor`.

Recalling our basic knowledge of digital circuits, we know a NAND gate is a universal gate, so let's compose other gates using our newly created `nand_gate` entity. Create a new file for our next design unit to describe an AND gate. 

Filename: and_gate.vhd
``` vhdl
library ieee;
use ieee.std_logic_1164.all;

entity and_gate is
  port(
    a, b : in std_logic;
    y : out std_logic
  );
end entity;

architecture rtl of and_gate is
begin

    -- What to put here?

end architecture;
```

After some thinking, we realize we can use two NAND gates together to construct an AND gate. Let's use Orbit to help us integrate our `nand_gate` entity into the `and_gate`'s architecture.

```
$ orbit get nand_gate --component --signals --instance
```
```
component nand_gate
  port(
    a : in std_logic;
    b : in std_logic;
    x : out std_logic
  );
end component;

signal a : std_logic;
signal b : std_logic;
signal x : std_logic;

uX : nand_gate
  port map(
    a => a,
    b => b,
    x => x
  );
```

With this single command, Orbit provided us with:
- the component declaration
- signals for the port interface
- an instantiation template

Sweet! After some quick copy/pasting and signal renaming, we have our architecture described for an AND gate.

Filename: and_gate.vhd
``` vhdl
library ieee;
use ieee.std_logic_1164.all;

entity and_gate is
  port(
    a, b : in std_logic;
    y : out std_logic
  );
end entity;

architecture rtl of and_gate is
  
  component nand_gate
    port(
      a : in std_logic;
      b : in std_logic;
      x : out std_logic
    );
  end component;

  signal x : std_logic;

begin

  u1 : nand_gate
    port map(
      a => a,
      b => b,
      x => x
    );

  u2 : nand_gate
    port map(
      a => x,
      b => x,
      x => y
    );

end architecture;
```

Let's make a quick check to verify our `and_gate` is using our `nand_gate`.

```
$ orbit tree
```
```
and_gate
└─ nand_gate
```

Cool! We got a hierarchical view of our top-most design unit.

## Building an ip for a scripted workflow

After all of our hard work, we are excited to show off our latest design on the newest Yilinx FPGA that just arrived in the mail. You realize you need a way to get your HDL code to the Yilinx synthesis tool in order to generate the final bitstream for your FPGA.

To make this possible, Orbit builds a project through two stages: planning and execution. Although both stages occur together, users must define their own targets to be invoked during execution. This explicit separation of layers between planning and execution enable the user to tailor the build process to their specific requirements.

### Creating a target

A target is a command invoked by Orbit for execution during the build process. In this example, we will write a script and have our target's command call our script to execute our process. In other words, you could say we are _targeting_ the Yilinx tool. Let's make a simple target for our Yilinx synthesis tool using the Python programming language.

Filename: .orbit/yilinx.py
``` python
file_order = []
# Read and parse the blueprint file
with open('blueprint.tsv') as blueprint:
    rules = blueprint.readlines()
    for r in rules:
        fileset, lib, path = r.strip().split('\t')
        if fileset == 'VHDL':
            file_order += [(lib, path)]
    pass

# Use the Yilinx tool to perform synthesize on the HDL files
for (lib, path) in file_order:
    print('YILINX:', 'Synthesizing file ' + str(path) +' into ' + str(lib) + '...')

# Use the Yilinx tool to perform placement and routing
print('YILINX:','Performing place-and-route...')

# Use the Yilinx tool to generate the bitstream
print('YILINX:', 'Generating bitstream...')
with open('fpga.bit', 'w') as bitstream:
    bitstream.write('011010101101' * 2)

print('YILINX:','Bitstream saved at: target/yilinx/fpga.bit')

```

For Orbit to know about our target, we need to give information to Orbit about the target. This is done in a configuration file. For this example, we edit the project-level configurations.

Filename: .orbit/config.toml
``` toml
[[target]]
name = "yilinx"
description = "Generate bitstreams for Yilinx FPGAs"
command = "python"
args = ["yilinx.py"]
```

### Calling a target

```
$ orbit build --target yilinx
```
```
YILINX: Synthesizing file /Users/chase/tutorials/gates/nand_gate.vhd into gates...
YILINX: Synthesizing file /Users/chase/tutorials/gates/and_gate.vhd into gates...
YILINX: Performing place-and-route...
YILINX: Generating bitstream...
YILINX: Bitstream saved at: target/yilinx/fpga.bit
```

Typically, we create targets to interface with EDA tools which will in turn produce desired output files, called artifacts. We see Yilinx saved our bitstream artifact for us to program our FPGA. Cool!

Filename: target/yilinx/fpga.bit
``` text
011010101101011010101101
```

## Making an ip and its design units reusable

Now we are ready to move on to more advanced topics, so let's go ahead and store an immutable reference to this project to use in other projects in our developer journey. 

```
$ orbit install
```

This command ran a series of steps that packaged our project and placed it into our _cache_. Internally, Orbit knows where our cache is and can reference designs from our cache when we request them. Let's make sure our project was properly installed by viewing our entire ip catalog.

```
$ orbit search
```
```
gates                       0.1.0       install
```

And there it is! Let's continue to the next tutorial, where we introduce dependencies across ips.

### Additional notes on project structure

Our final project structure looks like the following:
```
gates/
├─ .orbit/
│  ├─ config.toml
│  └─ yilinx.py
├─ target/
│  ├─ CACHEDIR.TAG
|  └─ yilinx/
│     ├─ .env
│     ├─ blueprint.tsv
│     └─ fpga.bit
├─ Orbit.toml
├─ Orbit.lock
├─ and_gate.vhd
└─ nand_gate.vhd
```

- The configurations stored in "/.orbit" exist only for this project; to store configurations that persist across projects make changes to the $ORBIT_HOME directory.

- Orbit creates an output directory to store the blueprint and any tool output files during a build. These files should reside in "/target" and may change often during development (probably don't check this directory into version control).

- Orbit creates a lock file "Orbit.lock" to store all the information required to manage and recreate the exact state of this project. It is a good idea to always keep it and to not manually edit it (probably be sure to check this file into version control).