#!/bin/sh
# This script is used to add the primary user to the docker group
set -e  # Exit on error

# add the docker group
sudo groupadd docker >/dev/null 2>&1 || true

# add the user to the docker group
sudo usermod -aG docker $USER

# activate the new group
newgrp docker

# ensure the ownership of the docker directory is correct
sudo chown "$USER":"$USER" /home/"$USER"/.docker -R >/dev/null 2>&1 || true
sudo chmod g+rwx "$HOME/.docker" -R >/dev/null 2>&1 || true
