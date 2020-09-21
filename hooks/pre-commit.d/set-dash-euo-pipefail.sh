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
BASE=$(git rev-parse --show-toplevel)/src

# Get a list of shellscripts (don't stop for an error)
SCRIPTS=$(find "$BASE" -name '*.sh' -readable 2>/dev/null || true)

# Blacklist folders we don't care about (these need to be relative to the base
# of the repository)
BLACKLIST=""

BAD=
for script in $SCRIPTS; do
  # Ensure the path isn't blacklisted
  for blacklist in $BLACKLIST; do
    if [ $(echo $script | fgrep "$BASE/$blacklist") ]; then
      continue 2
    fi
  done

  if ! head $script | fgrep 'set -euo pipefail' 2>&1>/dev/null; then
    err "$script does not contain the mandatory command: \`set -euo pipefile\`!"
    BAD=1
  fi
done

if [ $BAD ]; then
  err "Please fix your bash scripts!"
fi
