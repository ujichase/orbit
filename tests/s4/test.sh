#!/usr/bin/env bash

# Set the -e option
set -e

# install dependencies
cd aka1
orbit lock --force
orbit install --force
cd ..

cd ao
orbit lock --force
orbit install --force
cd ..

cd aka2
orbit lock --force
orbit install --force
cd ..

cd kuro
orbit lock --force
# verify the ip dependency graph only has 1 aka version
STDOUT=$(orbit tree -e ip)

cd ..

# store the ideal value for later comparison
EXACT="kuro:0.1.0
├─ ao:2.0.0
│  └─ aka:1.0.2
└─ aka:1.0.2"

# compare the output with the expected value
if [ "$STDOUT" != "$EXACT" ]; then
    orbit remove aka --force
    orbit remove aka --force
    orbit remove ao --force

    echo "TEST: PARTIAL_VERSION - FAIL"
    echo "--- Expected ---"
    echo "$EXACT"
    echo "--- Received ---"
    echo "$STDOUT"
    exit 101
fi

orbit remove aka --force
orbit remove aka --force
orbit remove ao --force

echo "TEST: PARTIAL_VERSION - PASS"
exit 0