#!/bin/sh
set -e  # Exit on error

git_repo_root_dir=$(git rev-parse --show-toplevel)
cd "$git_repo_root_dir"

# view the dependency tree
cargo tree

# view contributions to the binary size
cargo bloat