# Environment Variables

Orbit's configuration can be customized with the setting of specific environment variables. These variables can be accessed anytime Orbit is executed.

- `ORBIT_HOME` - Location where Orbit stores its data. By default Orbit reads and writes to "$HOME/.orbit" on Unix systems and "%USERPROFILE%/.orbit" on Windows systems.

- `NO_COLOR` - If set, do not print colorized output to the terminal.

- `ORBIT_WIN_LITERAL_CMD` - If set, disables the default behavior of checking for programs ending with ".exe" then ".bat" when a program name without extension is not found on Windows systems.

## Runtime environment variables

Orbit also sets environment variables during runtime such that any subprocesses within Orbit, such as targets, can access necessary information.

- `ORBIT_MANIFEST_DIR` - The full path for the directory that contains the current ip's manifest.

- `ORBIT_IP_NAME` - The name of the current ip.

- `ORBIT_IP_UUID` - The uuid of the current ip.

- `ORBIT_IP_LIBRARY` - The interpretated HDL library of the current ip.

- `ORBIT_IP_VERSION` - The version of the current ip.

- `ORBIT_IP_CHECKSUM` - The first 10 characters from the latest checksum of the current ip.

- `ORBIT_TARGET` - The name of the target selected for the latest build process.

- `ORBIT_TOP_NAME` - The top level design's identifier for the latest build process, only if the build process was a build.

- `ORBIT_TOP_FILE` - The file system path that contains the top level design for the latest build process, only if the build process was a build.

- `ORBIT_TB_NAME` - The testbench's identifier for the latest build process, only if the build process was a test.

- `ORBIT_TB_FILE` - The file system path that contains the testbench for the latest build process, only if the build process was a test.

- `ORBIT_DUT_NAME` - The design under test's identifier for the latest build process, only if the build process was a test.

- `ORBIT_DUT_FILE` - The file system path that contains the design under test for the latest build process, only if the build process was a test.

- `ORBIT_BLUEPRINT` - The file name for the blueprint created from the planning stage of the latest build process. The file name includes the file's extension.

- `ORBIT_TARGET_DIR` - Directory where all generated artifacts from any targets will be stored, relative to the current ip's directory. Default is "target".
  
- `ORBIT_OUT_DIR` - The folder where all generated artifacts for the current target will be stored. This folder is inside the target directory for the current ip, and is unique for each selected target. Default is the target's name.

- `ORBIT_CHAN_INDEX` - The full path for the directory where the current ip's manifest will be placed for the current channel in the publishing process.

## Checking the environment

See [`orbit env`](./../commands/env.md) for checking environment variables on the command-line. Not all environment variables, especially runtime environment variables, may be available.

<!--
Note about environment variables vs. settings file vs. arguments

precedence:
1. config file
2. env vars
3. command-line
-->