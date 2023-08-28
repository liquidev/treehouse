#!/usr/bin/env bash

cd "$TREEHOUSE_PATH"

echo
echo "* Running editor"
"$EDITOR" content/index.tree

bash admin/client_push.bash
