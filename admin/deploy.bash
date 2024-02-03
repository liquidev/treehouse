#!/usr/bin/env bash

ssh "$TREEHOUSE_SERVER" -p "$TREEHOUSE_SERVER_PORT" "bash" "~/repo/admin/daemon/reload.bash"
