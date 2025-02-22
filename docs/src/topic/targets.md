# Targets

Orbit operates at the _front end_ of hardware development. At the _back end_ of hardware development is where the true "processing" occurs. A particular "process" that occurs in the back end and produces some result is called a _target_.

Orbit has no built-in targets. Since hardware development varies widely with the tools available, the environments in which development occurs, and the processes that occur within those environments, Orbit has not built-in targets. This design choice gives users flexibility in configuring the types of workflows that are most important to them.

At the _front end_, Orbit frequently interacts with the user to handle operations and run processes within their hardware development workflow. The main role of Orbit is to organize, reference, and prepare HDL source code for the _back end_.

Targets typically take in as input the _blueprint_, which is the final output file from Orbit that has prepared the list of HDL files for input to the back end.

## Defining Targets

Users can setup a target in the configuration file `config.toml`. For all the available parameters to define a target, see [[[target]]](./../reference/configuration.md#the-target-array).
