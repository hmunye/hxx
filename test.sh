#!/usr/bin/env bash

set -euo pipefail

binary="./target/release/hxx"

if [[ "$#" -ne 1  ]]; then
    echo "Usage: $0 num_tests" >&2
    echo "                 ^^^^^^^^^ number of comparison tests" >&2
    exit 1
fi

num_tests=$1

if [ ! -f "$binary" ]; then
    echo "ERROR: $binary not found in the current directory" >&2
    echo "       ^^^^^^^^^^^^^^^^^^^^ run 'cargo build --release'" >&2
    exit 1
fi

if ! command -v xxd &> /dev/null; then
    echo "ERROR: xxd is not installed or not in PATH" >&2
    exit 1
fi

tmp_file=$(mktemp)
trap "rm -f $tmp_file" EXIT

for ((i = 1; i <= num_tests; ++i)); do
    random_num=$(( RANDOM % (64 - 2 + 1) + 2 ))

    head -c "$random_num" /dev/urandom > "$tmp_file" || true

    echo -n "TEST $i: "

    if diff <($binary "$tmp_file") <(xxd "$tmp_file") > /dev/null; then
        echo "PASS"
    else
        echo "FAIL"
        diff <($binary "$tmp_file") <(xxd "$tmp_file") >&2
        exit 1
    fi
done
