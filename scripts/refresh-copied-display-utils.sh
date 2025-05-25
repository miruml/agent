#!/bin/sh
set -e

THIS_REPO_ROOT_DIR=$(git rev-parse --show-toplevel)
DISPLAY_UTILS_SH="$THIS_REPO_ROOT_DIR/scripts/display-utils.sh"

refresh_copied_display_utils() {
    dest_file="$1"

    tmpfile=$(mktemp)

    # Extract the display utilities block from the input file
    awk '/### COPIED DISPLAY UTILITIES BEGIN ###/{flag=1} flag; /### COPIED DISPLAY UTILITIES END ###/{flag=0}' "$DISPLAY_UTILS_SH" > "$tmpfile"

    # Replace the block in the output file with the one from the input file
    awk -v repl="$(cat "$tmpfile")" '
        BEGIN {inblock=0}
        /### COPIED DISPLAY UTILITIES BEGIN ###/ {print repl; inblock=1; next}
        /### COPIED DISPLAY UTILITIES END ###/ {inblock=0; next}
        !inblock {print}
    ' "$dest_file" > "${dest_file}.new" && mv "${dest_file}.new" "$dest_file"

    rm "$tmpfile"
    chmod +x "$dest_file"
}

assert_file_exists() {
    file="$1"
    if [ ! -f "$file" ]; then
        echo "File $file does not exist"
        exit 1
    fi
}

echo "Refreshing Copied Display Utilities"
echo "==================================="

INSTALL_ARG_UTILS_SH="$THIS_REPO_ROOT_DIR/scripts/install/arg-utils.sh"
assert_file_exists "$INSTALL_ARG_UTILS_SH"
echo "Refreshing $INSTALL_ARG_UTILS_SH"
refresh_copied_display_utils "$INSTALL_ARG_UTILS_SH"