#!/bin/bash

mode="$1"

git pull

# Build server
cd server/
cargo build --quiet --release --feature $mode

# Launch server
cd ../../
./ eos/server/target/release/server

# Update one more time in case this script changes
cd eos/
git pull

# Repeat
sleep 2
exec "$0" $mode
