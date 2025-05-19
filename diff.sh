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

passed_tests=0
total_bytes=0
total_time=0

pad_width=${#num_tests}

tmp_input=$(mktemp)
tmp_output=$(mktemp)
tmp_reverse_output=$(mktemp)

trap "rm -f $tmp_input $tmp_output $tmp_reverse_output" EXIT

for ((i = 1; i <= num_tests; ++i)); do
    # Random number between 1000 and 9999 bytes
    random_num=$(( RANDOM % 9000 + 1000 ))

    head -c "$random_num" /dev/urandom > "$tmp_input" || true

    # Record start time (in seconds with fractional part)
    start_time=$(perl -MTime::HiRes=time -E 'say time')

    # Overwrite file each iteration
    $binary "$tmp_input" > "$tmp_output"

    # Compare hex dump
    if diff "$tmp_output" <(xxd "$tmp_input") > /dev/null; then
        # Overwrite file each iteration
        $binary -r "$tmp_output" > "$tmp_reverse_output"

        # Compare reverse hex dump
        if diff "$tmp_reverse_output" <(xxd -r "$tmp_output") > /dev/null; then
            end_time=$(perl -MTime::HiRes=time -E 'say time')
            elapsed=$(awk "BEGIN {printf \"%.3f\", ($end_time - $start_time) * 1000}")

            printf "[%0${pad_width}d] | ${green}PASS - hex-dump/reverse${nc} | ${elapsed} ms | ${random_num} bytes\n" "${i}"

            passed_tests=$((passed_tests + 1))
            total_bytes=$((total_bytes + random_num))
            total_time=$(echo "$total_time + $elapsed" | bc)
        else
            printf "[%0${pad_width}d] | ${red}FAIL - reverse${nc}\n\n" "${i}"
            diff -y "$tmp_reverse_output" <(xxd -r "$tmp_output") >&2 || true
            break
        fi
    else
        printf "[%0${pad_width}d] | ${red}FAIL - hex-dump${nc}\n\n" "${i}"
        diff -y "$tmp_output" <(xxd "$tmp_input") >&2 || true
        break
    fi
done

avg_time=$(echo "scale=3; $total_time / $passed_tests" | bc)

echo
echo "Tests passed: $passed_tests/$num_tests"
printf "Total bytes:  %'d bytes \n" "${total_bytes}"
echo "Average time: ${avg_time} ms"
