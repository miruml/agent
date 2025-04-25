#!/bin/sh
set -e  # Exit on error

version="0.6.5"
deb_pkg_name="miru"

miru_binaries_dir="/usr/local/miru"

agent_bin_name="miru-$version"
agent_bin_path="$miru_binaries_dir/$agent_bin_name"
agent_bin_symlink_name="miru"
agent_bin_symlink_path="$miru_binaries_dir/$agent_bin_symlink_name"
agent_start_script_name="miru-start.sh"

install_bin_name="install-miru"
install_bin_path="$miru_binaries_dir/$install_bin_name"

systemd_dir="/etc/systemd/system"
systemd_service_name="miru.service"
systemd_service_path="$systemd_dir/$systemd_service_name"

miru_dir="/var/lib/miru"
miru_log_dir="/var/log/miru"

# Variables
script="postinstallation"
user="miru"

# Ensure the script is running with elevated privileges
if [ "$(id -u)" -ne 0 ]; then
echo "'$script' script must be run as the root user or with sudo"
exit 1
fi

# Check if the service file is present
if [ ! -f $systemd_service_path ]; then
    echo "systemd service file not found!"
    exit 1
fi

# Check if the user exists, create if it doesn't
if ! id -u "$user" > /dev/null 2>&1; then
    useradd -r -s /bin/false "$user"
fi

# Grant miru agent directory permissions
if [ ! -d "$miru_dir" ]; then
    echo "App directory not found!"
    exit 1
fi
chown -R "$user:$user" "$miru_dir"

# Grant miru log directory permissions
mkdir -p "$miru_log_dir"
chown -R "$user:$user" "$miru_log_dir"

# Grant executable permissions to the miru binary
if [ ! -d "$miru_binaries_dir" ]; then
    echo "Binary directory not found!"
    exit 1
fi
chown -R "$user:$user" "$miru_binaries_dir"

# Start the systemd service
systemctl daemon-reload
systemctl enable $systemd_service_name
systemctl restart $systemd_service_name

# Exit successfully
exit 0