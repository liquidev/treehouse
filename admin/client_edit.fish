#!/usr/bin/env fish

cd $TREEHOUSE_PATH

echo
set_color white --bold; echo "* Running editor"; set_color normal
eval $EDITOR content/index.tree

echo
set_color white --bold; echo "* Fixing the tree"; set_color normal
cargo run -p treehouse fix-all --apply

echo
set_color white --bold; echo "* Committing changes"; set_color normal
git add content
git commit
git push

echo
set_color white --bold; echo "* Uploading to server"; set_color normal
fish admin/client_sync.fish
