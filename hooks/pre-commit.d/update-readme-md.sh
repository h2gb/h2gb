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

# Do the main README.md
cargo readme -o README.md

# Append other paths to README.md
echo -ne "\n# Other Documentation\n\n" >> README.md

# Do any subdirectories
for i in $(find . -type f -name mod.rs); do
  if (head -n1 $i | grep '^\/\/\!' > /dev/null); then
    DIR=$(dirname "$i")
    cargo readme -i "$i" -o "$DIR/README.md"
    git add "$DIR/README.md"

    echo -ne "* [$DIR]($DIR/README.md) - $( head -n1 $i | cut -c5- )\n\n" >> README.md
  fi
done


popd > /dev/null
