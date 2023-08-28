#!/usr/bin/env bash

ssh "$TREEHOUSE_SERVER" -p "$TREEHOUSE_SERVER_PORT" 'fish' '$TREEHOUSE_PATH/admin/server_sync.fish'
