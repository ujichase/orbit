#!/usr/bin/env bash

# Set the -e option
set -e

cd sub
orbit lock --force
cd ..

cd top
orbit lock --force
# verify the ip dependency graph only has 1 aka version
STDOUT=$(orbit tree -e ip)

cd ..

# store the ideal value for later comparison
EXACT="top:0.1.0
└─ sub:0.1.0"

# compare the output with the expected value
if [ "$STDOUT" != "$EXACT" ]; then
    echo "TEST: RELATIVE_DEPENDENCY - FAIL"
    echo "--- Expected ---"
    echo "$EXACT"
    echo "--- Received ---"
    echo "$STDOUT"
    exit 101
fi

echo "TEST: RELATIVE_DEPENDENCY - PASS"
exit 0