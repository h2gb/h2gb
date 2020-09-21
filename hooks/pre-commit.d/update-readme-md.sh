#!/bin/bash

set -euo pipefail

# Print errors in red
err() {
  echo -ne '\e[31m\e[1m' # Red + Bold
  echo -e "$@"
  echo -ne '\e[0m'
  exit 0
}

# Get the root directory
BASE=$(git rev-parse --show-toplevel)

# Update README.md
pushd $BASE > /dev/null
cargo readme > README.md || err 'Failed to run `cargo readme`!'
git add README.md
popd > /dev/null
