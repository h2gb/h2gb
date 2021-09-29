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
pushd $BASE/h2gb > /dev/null

# Do the main README.md
cargo readme -o $BASE/README.md

# Append other paths to README.md
echo -ne "\n# Other Documentation\n\n" >> $BASE/README.md

# Do any subdirectories
for i in $(find ../ -type f -name mod.rs -or -name lib.rs -not -wholename ../h2gb/src/lib.rs | sort); do
  if (head -n1 $i | grep '^\/\/\!' > /dev/null); then
    DIR=$(dirname "$i")
    echo "***Note: This file was automatically generated from lib.rs or mod.rs***" > "$DIR/README.md"
    echo "" >> "$DIR/README.md"
    cargo readme --no-title -i "$i" >> "$DIR/README.md"
    git add "$DIR/README.md"

    # Remove the leading ../, since we're adding it to a top-level README
    RELATIVE_DIR=$(echo "$DIR" | cut -c3-)
    echo -ne "* [$RELATIVE_DIR]($RELATIVE_DIR/README.md) - $( head -n1 $i | cut -c5- )\n\n" >> $BASE/README.md
  fi
done

popd > /dev/null
