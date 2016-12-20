#!/bin/sh

set -e

DIR=$(dirname "$0")
DAYS=$(find "$DIR" -maxdepth 1 -type d -name 'Day [0-9]*')

echo "$DAYS" | while read DAY; do
    echo "===>" $(basename "$DAY")
    (cd "$DAY" && cargo test --verbose)
done
