#! /bin/bash

set -euo pipefail



cargo build --release

./target/release/verilot generate /tmp/verilot_secret.txt | tee /tmp/verilot_commitment.txt

printf "foo\nbar\nbaz" | ./target/release/verilot lottery --secret /tmp/verilot_secret.txt | tee /tmp/verilot_result.json

cat /tmp/verilot_result.json | ./target/release/verilot verify --commitment "$(cat /tmp/verilot_commitment.txt)"
