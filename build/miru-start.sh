#!/bin/sh
set -e # Exit on error

version="0.6.5"
deb_pkg_name="miru"

miru_binaries_dir="/usr/local/miru"

agent_bin_name="miru-$version"
agent_bin_path="$miru_binaries_dir/$agent_bin_name"
agent_bin_symlink_name="miru"
agent_bin_symlink_path="$miru_binaries_dir/$agent_bin_symlink_name"

install_bin_name="install-miru"
install_bin_path="$miru_binaries_dir/$install_bin_name"

systemd_dir="/etc/systemd/system"
systemd_service_name="miru.service"
systemd_service_path="$systemd_dir/$systemd_service_name"

miru_dir="/var/lib/miru"
miru_log_dir="/var/log/miru"

if ! "$agent_bin_symlink_path"; then

    echo "ERROR: Running miru agent symlink $agent_bin_symlink_path failed."
    echo ""

    # Find all the miru binaries in the /usr/local/miru directory which are files
    # (as opposed to directories, symlinks, etc.) matching the pattern 'miru*' and
    # sort them alphabetically in reverse order. If the directory contains {0.4.7, 0.5.0, 0.5.1, 0.6.0, 1.0.0, 10.0.0, etc.}, then the expected output is:
    # 10.0.0
    # 1.0.0
    # 0.6.0
    # 0.5.1
    # 0.5.0
    # 0.4.7
    sorted_binaries=$(find "$miru_binaries_dir" -type f -name 'miru-[0-9]*.[0-9]*.[0-9]*' ! -name "$(basename "$0")" | sort -Vr)
    echo "Found potential alternative miru binaries:"
    echo "$sorted_binaries"
    echo ""
    
    # Try to execute binaries in version order
    for bin in $sorted_binaries; do
        if [ -x "$bin" ]; then
            echo "Attempting to run miru binaries $bin"
            if "$bin"; then
                return 0
            else
                echo "Failed to run $bin, trying older version"
            fi
        else
            echo "Skipping non-executable file: $bin"
        fi
    done
    
    echo "All versions failed"
    return 1
fi
