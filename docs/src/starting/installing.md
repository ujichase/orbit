# Installing

There are two main methods for getting Orbit running on your computer: downloading a precompiled binary or by using Cargo.

> __Reminder:__ By installing and using Orbit, you accept usage under its GPL-3.0 license.

## 1. Using a precompiled binary

1.  Visit Orbit's [releases](https://github.com/chaseruskin/orbit/releases) page on Github to find all of its official releases. 
2. Download the binary for your computer's architecture and operating system.
3. Install Orbit. Either run the provided `install` executable or follow the manual instructions for placing Orbit's executable (`orbit` for Unix and `orbit.exe` for Windows) in a location recognized by the PATH environment variable.

There are multiple solutions to accomplish step 3. The following outlines one way to manually install Orbit depending on the user's operating system. 

### Linux
1. Download the latest prebuilt package.
```
curl -LO https://github.com/chaseruskin/orbit/releases/download/CARGO_CRATE_VERSION/orbit-CARGO_CRATE_VERSION-x86_64-linux.zip
```

2. Unzip the prebuilt package.
```
unzip orbit-CARGO_CRATE_VERSION-x86_64-linux.zip
```

3. Move the executable to a location already set in the PATH environment variable. 
```
mv ./orbit-CARGO_CRATE_VERSION-x86_64-linux/bin/orbit /usr/local/bin/orbit
```

### macOS
1. Download the latest prebuilt package.
```
curl -LO https://github.com/chaseruskin/orbit/releases/download/CARGO_CRATE_VERSION/orbit-CARGO_CRATE_VERSION-x86_64-macos.zip
```

2. Unzip the prebuilt package.
```
unzip orbit-CARGO_CRATE_VERSION-x86_64-macos.zip
```

3. Move the executable to a location already set in the PATH environment variable. 
```
mv ./orbit-CARGO_CRATE_VERSION-x86_64-macos/bin/orbit /usr/local/bin/orbit
```

### Windows
1. Open a new terminal (Powershell) to where Orbit was downloaded.

2. Unzip the prebuilt package.
```
$ expand-archive "./orbit-CARGO_CRATE_VERSION-x86_64-windows.zip"
```

3. Make a new directory to store this package.
```
$ new-item -path "$env:LOCALAPPDATA/Programs/orbit" -itemtype directory
```

4. Move the package to the new directory.
```
$ copy-item "./orbit-CARGO_CRATE_VERSION-x86_64-windows/*" -destination "$env:LOCALAPPDATA/Programs/orbit" -recurse
```

5. Edit the user-level PATH environment variable in ___Control Panel___ by adding __%LOCALAPPDATA%\Programs\orbit\bin__.

## 2. Installing with Cargo

To install the latest version through Cargo:
```
$ cargo install --git https://github.com/chaseruskin/orbit.git --bin orbit --tag CARGO_CRATE_VERSION
```

This will build the `orbit` binary and place it a path already set in the PATH environment variable.

## Checking if Orbit is installed correctly

To verify Orbit is working correctly on your system, let's open a new terminal session and print it's current version.
```
$ orbit --version
```
```
orbit CARGO_CRATE_VERSION
```
This should print out your version of Orbit you installed. Congratulations! You are now ready to begin using Orbit.