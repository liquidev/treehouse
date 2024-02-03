#!/usr/bin/env bash

cd ~/repo
git pull
"${BASH_SOURCE%/*}/reload.bash"
