#!/bin/sh
echo ''
set -e  # Exit on error

# Ensure the script is running with elevated privileges
if [ "$(id -u)" -ne 0 ]; then
echo "This miru installation script must be run with 'sudo' to be able to install miru as a debian package to your system. Please prepend 'sudo' to your previous command to run this script."
exit 1
fi

# Determine the architecture
echo "Determining the architecture..."
arch="arm64"  # default to arm64
if [ "$(uname -m)" = "x86_64" ] || [ "$(uname -m)" = "x64" ]; then
	arch="amd64"
fi
echo "Architecture: $arch"

# Loop through each file in the directory matching the glob pattern
for file in ./miru*.deb; do
	# Check if the file exists
	if [ -f "$file" ]; then
		# Remove the file
		rm "$file"
		echo "Removing existing miru installation file: $file"
	fi
done

# download the debian package
printf "\nDownloading the miru debian package..."
curl -OJsSf "%s/v1/install/$arch"

# Verify a debian package was downloaded
count=$(find . -maxdepth 1 -name 'miru*.deb' | wc -l)
# Check if the count is zero
if [ "$count" -eq 0 ]; then
    echo "Failed to download the Debian package. Exiting..."
    exit 1
fi

# install dependencies
printf "\nInstalling dependencies..."
dependencies="pkg-config"
for dependency in $dependencies; do
	if ! apt-get install -y "$dependency"; then
		echo "Installing $dependency dependency failed. Attempting to fix..."
		apt-get --fix-broken install -y
		echo "Retrying installation of $dependency..."
		apt-get install -y "$dependency"
	fi
done

# remove any previous installations
printf "\nRemoving any previous miru installations..."
dpkg --purge miru

# install the debian package
printf "\nInstalling miru debian package..."
dpkg -i miru*.deb

# remove the debian package
printf "\nRemoving downloaded files..."
rm miru*.deb

# start the installation script
printf "\nStarting the installation script..."
/usr/local/miru/install-miru < /dev/tty

exit 0
