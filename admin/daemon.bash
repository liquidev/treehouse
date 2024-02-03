#!/usr/bin/env bash

source "${BASH_SOURCE%/*}/daemon/common.bash"
build_log="/tmp/treehouse_build.log"

trap 'trap - SIGTERM && kill 0' SIGTERM SIGINT EXIT

rm -f $reload_fifo
mkfifo $reload_fifo

reload() {
    # This just kind of assumes regeneration doesn't take too long.
    kill "$treehouse_pid"
    cargo run --release -- serve --port 8082 > "$build_log" &
    treehouse_pid="$!"
}

reload

while true; do
    read command < "$reload_fifo"
    case "$command" in
        reload)
            echo "Reloading"
            reload;;
    esac
done
