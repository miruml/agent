#!/bin/sh
# This script is used to install (or reinstall) Docker
echo ''
set -e  # Exit on error

os_type="ubuntu" # debian

# Variables
# script_name="docker-install.sh"

# Ensure the script is running with elevated privileges
# if [ "$(id -u)" -ne 0 ]; then
# echo "'$script_name' script must be run with the root user. Please use sudo or switch to the root user."
# exit 1
# fi

echo "Removing conflicting Docker installations"
echo "-----------------------------------------"
for pkg in docker.io docker-doc docker-compose docker-compose-v2 podman-docker containerd runc; do sudo apt-get remove -y $pkg || true; done
echo ''

echo "Adding Docker's official GPG key"
echo "--------------------------------"
sudo apt-get update
sudo apt-get -y install ca-certificates curl
sudo install -m 0755 -d /etc/apt/keyrings
sudo curl -fsSL https://download.docker.com/linux/$os_type/gpg -o /etc/apt/keyrings/docker.asc
sudo chmod a+r /etc/apt/keyrings/docker.asc
echo ''

echo "Adding Docker's official repository"
echo "-----------------------------------"
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/$os_type \
  $(. /etc/os-release && echo "${UBUNTU_CODENAME:-$VERSION_CODENAME}") stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt-get update
echo ''

echo "Installing Docker"
echo "-----------------"
sudo apt-get -y --allow-downgrades install docker-ce=5:27.5.1-1* docker-ce-cli=5:27.5.1-1* containerd.io docker-buildx-plugin docker-compose-plugin
echo ''
