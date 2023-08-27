#!/usr/bin/env fish

cd $TREEHOUSE_PATH

echo
set_color white --bold; echo "* Running editor"; set_color normal
eval $EDITOR content/index.tree

fish admin/client_push.fish
