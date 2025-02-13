# Author: Chase Ruskin
# Details:
#   A series of commands to run at convenience.

_default:
    just --list

# Install the debug build in placement of old binary for local system testing
install:
    cargo build --release
    cp ./target/release/orbit "$HOME/.cargo/bin/orbit"

# Run partial section of tests by specifying the modules in MODS
codev MODS:
    cargo watch -c -x check -x "test -- {{ MODS }}" --ignore test/data

# Synchronize documentation from TOML file to markdown and Rust
mansync:
    python ./tools/mansync.py

# Generate a summary of changelog notes for the next version release
chlog:
    python ./tools/clgen.py --verbose

# Updates the source code with the license header
lic:
    python ./tools/license.py

# Run the documentation book on the local host for testing before going live
docs:
    mdbook serve ./docs

# Run all the possible tests available for this project
fulltest:
    cargo check
    cargo test
    python -m unittest discover -s ./tools -p '*.py'

# Sort the glossary contents
sortgloss:
    echo "$(python ./tools/sort_gloss.py)" > ./docs/src/glossary.md

# Organize commands required for end-to-end system-level tests

run-sys-tests:
    just test-plan-1
    just test-plan-2
    just test-dst
    just test-pub
    just test-partv
    just test-rel-dep

# Run all system tests
test-all:
    just test-plan-1
    just test-plan-2
    just test-dst
    just test-dst-local
    just test-pub
    just test-partv
    just test-rel-dep

# Planning stage (associated files)
test-plan-1:
    chmod +x ./tests/s1/test.sh
    cd ./tests/s1; ./test.sh

# Planning stage (minimum order)
test-plan-2:
    chmod +x ./tests/s2/test.sh
    cd ./tests/s2; ./test.sh

# Dst algorithm
test-dst:
    chmod +x ./tests/s3/dst.sh
    cd ./tests/s3; ./dst.sh

# Dst test to verify library mappings work when using a simulator
test-dst-local:
    chmod +x ./tests/s3/dst-local.sh
    cd ./tests/s3; ./dst-local.sh

# Using 'public' in manifest
test-pub: 
    chmod +x ./tests/s3/pub.sh
    cd ./tests/s3; ./pub.sh

# Partial version for dependency
test-partv:
    chmod +x ./tests/s4/test.sh
    cd ./tests/s4; ./test.sh

# Using relative dependencies
test-rel-dep:
    chmod +x ./tests/s5/test.sh
    cd ./tests/s5; ./test.sh

# TODO: Ip namespace collision 
test-ip-collide:
    chmod +x ./tests/s6/test.sh
    cd ./tests/s6; ./test.sh