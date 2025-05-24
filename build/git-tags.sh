#!/bin/sh
set -e

# determine the previous tag by tags which follow 'vX.y.z' but have no characters after
# the 'z'
previous_tag () {
    git tag -l 'v[0-9]*.[0-9]*.[0-9]*' | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -n 1
}

latest_tag () {
    git tag -l 'v[0-9]*.[0-9]*.[0-9]*' | sort -V | tail -n 1
}

current_commit_tag () {
    git describe --exact-match --tags HEAD
}
