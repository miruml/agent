#!/bin/sh
set -e

current_tag () {
    git describe --exact-match --tags HEAD
}

current_tags () {
    git tag --points-at HEAD
}

# determine the previous tag by tags which follow 'vX.y.z' but have no characters after
# the 'z' AND are not the current tag
previous_tag () {
    git tag -l 'v[0-9]*.[0-9]*.[0-9]*' \
        | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' \
        | grep -v -F -x "$(current_tags)" \
        | sort -V \
        | tail -n 1
}

latest_tag () {
    git tag -l 'v[0-9]*.[0-9]*.[0-9]*' | sort -V | tail -n 1
}

echo "Current tag: $(current_tag)"
echo "Current tags:"
for tag in $(current_tags); do
    echo "  $tag"
done
echo "Previous tag: $(previous_tag)"
echo "Latest tag: $(latest_tag)"

