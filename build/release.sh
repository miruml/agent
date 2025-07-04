#!/bin/sh
set -e

this_repo_root_dir=$(git rev-parse --show-toplevel)
this_dir=$this_repo_root_dir/build
cd "$this_dir"

# shellcheck source=git-tags.sh
. "$this_dir/git-tags.sh"
previous_tag=$(previous_tag)
echo "Previous tag: $previous_tag"
export GORELEASER_PREVIOUS_TAG="$previous_tag"

goreleaser release --clean