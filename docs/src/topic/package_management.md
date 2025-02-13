# Agile Package Management

Orbit is an agile package manager for HDLs. Orbit supports a wide range of commands to the user to automate codebase management processes for installing ip, referencing ip, and removing ip.

## Installing

`orbit install`

Before you can reference an ip in your current project, you must first make sure the ip exists on your local filesystem. Orbit manages ip through its _ip catalog_. The catalog consists of multiple file paths that Orbit maintains for ips at varying states of accessibility.

To make an ip accessible by another project, it must first be installed to your ip catalog.

## Referencing

`orbit info`, `orbit get`

Once an ip is installed to your ip catalog, the design units of that ip are available to be referenced in the current design.

1. Tell Orbit which installed ip you wish to use by providing the name and version under the `[dependencies]` table in the Orbit.toml file.

2. Instantiate one of the available design units from the dependency in your source code. At this point, any build that uses this HDL source code will be correctly sorted during the planning stage in topological order.

## Removing

`orbit remove`

An ip can be removed from the catalog when it is no longer supported or needed to be used again. By removing the ip, Orbit deletes the ip's contents stored in the catalog, effectively forgetting that it existed.