#!/usr/bin/env bash

cd ~/repo

source "${BASH_SOURCE%/*}/common.bash"

git pull
bash "${BASH_SOURCE%/*}/reload.bash"

echo "^C to exit build log ($build_log)"
tail --retry -f "$build_log"
