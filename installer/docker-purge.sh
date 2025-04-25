#!/bin/sh
# This script is used to uninstall Docker on an Ubuntu system
echo ''
set -e  # Exit on error

script_name="docker-purge.sh"

# Ensure the script is running with elevated privileges
if [ "$(id -u)" -ne 0 ]; then
echo "'$script_name' script must be run with the root user. Please use sudo or switch to the root user."
exit 1
fi

echo "Purging Docker installations"
echo "----------------------------"
sudo apt-get -y purge docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin docker-ce-rootless-extras
echo ''

echo "Removing Docker directories"
echo "---------------------------"
sudo rm -rf /var/lib/docker
sudo rm -rf /var/lib/containerd
echo ''