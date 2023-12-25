#!/bin/bash

mode="$1"
branch="$2"

# TODO repo name
git pull origin $branch

# Build server
cd server/
cargo build --quiet --release --feature $mode

# Launch server
cd ../../
./ eos/server/target/release/server

# Update one more time in case this script changed
cd eos/
git pull origin $branch

# Repeat
sleep 2
exec "$0" $mode $branch
