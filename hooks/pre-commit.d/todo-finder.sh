#!/bin/bash

set -euo pipefail

# Get the root directory
BASE=$(git rev-parse --show-toplevel)/src

# Turn on yellow output + grep for TODO
echo -ne '\e[33m' # Yellow
egrep -r 'TODO' $BASE/ || true # Don't die if there are none
echo -ne '\e[0m' # No formatting

# Turn on red output + grep for XXX
echo -ne '\e[31m' # Yellow
egrep -r 'XXX' $BASE/ || true # Don't die if there are none
echo -ne '\e[0m' # No formatting

# Never block
exit 0
