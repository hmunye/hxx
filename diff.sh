#!/usr/bin/env bash

set -euo pipefail

green='\x1b[1;32m'
red='\x1b[1;31m'
nc='\x1b[0m' 

binary="./target/release/hxx"

if ! command -v xxd &> /dev/null; then
    echo "Error: 'xxd' missing or unavailable in PATH. This script requires 'xxd' to compare outputs with $binary" >&2
    exit 1
fi

if [[ "$#" -ne 1  ]]; then
    echo "Usage:" >&2 
    echo "       $0 <num_tests>" >&2
    echo "                  ^^^^^^^^^ number of output comparison tests" >&2
    exit 1
fi

num_tests=$1

if [ ! -f "$binary" ]; then
    echo "Error: $binary not found in the current directory" >&2
    echo "       ^^^^^^^^^^^^^^^^^^^^ run 'cargo build --release'" >&2
    exit 1
fi

pad_width=${#num_tests} 

tmp_file=$(mktemp)
trap "rm -f $tmp_file" EXIT

for ((i = 1; i <= num_tests; ++i)); do
    # Random number between 100 and 10000 bytes
    random_num=$(( RANDOM % 9901 + 100 ))

    head -c "$random_num" /dev/urandom > "$tmp_file" || true

    printf "[Test %0${pad_width}d] Status: " "$i"

    if diff <($binary "$tmp_file") <(xxd "$tmp_file") > /dev/null; then
        echo -e "${green}PASS${nc}"
    else
        echo -e "${red}FAIL${nc}"
        diff -y <($binary "$tmp_file") <(xxd "$tmp_file") >&2
        exit 1
    fi
done
