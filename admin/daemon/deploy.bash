#!/usr/bin/env bash

cd ~/repo
git pull
bash "${BASH_SOURCE%/*}/reload.bash"
