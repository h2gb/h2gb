#!/bin/bash

set -euo pipefail

# Print errors in red
err() {
  echo -ne '\e[31m\e[1m' # Red + Bold
  echo -e "$@"
  echo -ne '\e[0m'
  exit 1
}

# Get the root directory
BASE=$(git rev-parse --show-toplevel)

pushd $BASE > /dev/null
cargo test --all-features -q 2>&1 > /dev/null || err "One or more tests failed!"
popd > /dev/null
