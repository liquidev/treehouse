#!/usr/bin/env bash

cd "$TREEHOUSE_PATH"

echo
echo "* Fixing the tree"
cargo run -p treehouse fix-all --apply

echo
echo "* Committing changes"
git add \
    content static template \
    treehouse.toml
git commit
git push

echo
echo "* Uploading to server"
bash admin/client_sync.bash
