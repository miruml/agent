#!/bin/sh
set -e

this_repo_root_dir=$(git rev-parse --show-toplevel)
this_dir=$this_repo_root_dir/build
cd "$this_dir"

goreleaser release --snapshot --clean