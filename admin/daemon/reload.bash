#!/usr/bin/env bash

source "${BASH_SOURCE%/*}/common.bash"

echo "Reloading"
echo "reload" > "$reload_fifo"
