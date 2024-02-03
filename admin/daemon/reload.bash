#!/usr/bin/env bash

source "${BASH_SOURCE%/*}/common.bash"

echo "reload" > "$reload_fifo"
