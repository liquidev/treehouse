#!/usr/bin/env fish

# This script runs on the server.

cd $TREEHOUSE_PATH

git pull
cargo run --release -p treehouse generate
