--------------------------------------------------------------
:: GETTING STARTED WITH ORBIT                               ::
--------------------------------------------------------------

Orbit is a package manager and build tool for hardware
description languages. It is a single binary executable
interfaced through the command-line.

-----------------------------
:: AUTOMATIC INSTALLATION  ::
-----------------------------

Run the provided `install` executable. It will guide you 
through the necessary configurations and ask a final prompt 
if the user wishes to proceed with the installation.

NOTE: If the requested installation path requires elevated 
    privileges, then the install executable must be ran 
    with those privileges.

NOTE: For unix, only the Orbit binary is installed. For
    windows, the entire Orbit directory is installed.

-----------------------------
:: MANUAL INSTALLATION     ::
-----------------------------

Follow the instructions at: 
https://chaseruskin.github.io/orbit/starting/installing.html.

-----------------------------
:: DOCUMENTATION           ::
-----------------------------

The documentation is available online at:

https://chaseruskin.github.io/orbit

as well as bundled together with this package (see ./docs).
The documentation website can be self-hosted for your convenience.
Any localhost command should work, such as:

$ php -t docs -S localhost:8080

$ python -m http.server 8080 -d docs

-----------------------------
:: RESOURCES               ::
-----------------------------

Repository: https://github.com/chaseruskin/orbit

Documentation: https://chaseruskin.github.io/orbit/

-----------------------------
:: LICENSE                 ::
-----------------------------

A copy of the license is provided with this installation
which is named LICENSE. This project is licensed under the 
open-source copyleft GNU GPL-3.0 license.