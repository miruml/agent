#!/bin/sh
set -e

THIS_REPO_ROOT_DIR=$(git rev-parse --show-toplevel)
INSTALL_DIR="$THIS_REPO_ROOT_DIR/scripts/install"
ARG_UTILS_SH="$INSTALL_DIR/arg-utils.sh"

refresh_copied_args_utils() {
    dest_file="$1"

    tmpfile=$(mktemp)

    # Extract the display utilities block from the input file
    awk '/### COPIED ARGUMENT UTILITIES BEGIN ###/{flag=1} flag; /### COPIED ARGUMENT UTILITIES END ###/{flag=0}' "$ARG_UTILS_SH" > "$tmpfile"

    # Replace the block in the output file with the one from the input file
    awk -v repl="$(cat "$tmpfile")" '
        BEGIN {inblock=0}
        /### COPIED ARGUMENT UTILITIES BEGIN ###/ {print repl; inblock=1; next}
        /### COPIED ARGUMENT UTILITIES END ###/ {inblock=0; next}
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

echo "Refreshing Copied Argument Utilities"
echo "===================================="

ACTIVATE_SH="$INSTALL_DIR/activate.sh"
assert_file_exists "$ACTIVATE_SH"
echo "Refreshing $ACTIVATE_SH"
refresh_copied_args_utils "$ACTIVATE_SH"

DEB_INSTALL_SH="$INSTALL_DIR/deb-install.sh"
assert_file_exists "$DEB_INSTALL_SH"
echo "Refreshing $DEB_INSTALL_SH"
refresh_copied_args_utils "$DEB_INSTALL_SH"

DEVELOP_INSTALL_SH="$INSTALL_DIR/develop-install.sh"
assert_file_exists "$DEVELOP_INSTALL_SH"
echo "Refreshing $DEVELOP_INSTALL_SH"
refresh_copied_args_utils "$DEVELOP_INSTALL_SH"

INSTALL_SH="$INSTALL_DIR/install.sh"
assert_file_exists "$INSTALL_SH"
echo "Refreshing $INSTALL_SH"
refresh_copied_args_utils "$INSTALL_SH"