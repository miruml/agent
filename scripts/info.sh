#!/bin/sh
set -e  # Exit on error

# view the dependency tree
cargo tree

# view contributions to the binary size
cargo bloat