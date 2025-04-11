#!/bin/sh
# This script is used to build the deb package installation locally on the system.
echo ''
set -e  # Exit on error

. "$(pwd)/vars.sh"

# Move to the build directory for linting and testing
cd ..

# Lint the code
# ./lint.sh

# Check for any changes (staged, unstaged, or untracked)
if [ -z "$(git status --porcelain)" ]; then
    echo "There are no changes to commit. Proceeding to build the binaries."
else
    echo "To keep track of what code is on the device, we link the GitHub commit to the filename. For this to be accurate, all current changes (which will be included in the zip) must be commited. Please remove or commit your changes before zipping the server deployment file."
    exit 1
fi

# Run the tests
# ./test.sh

# Move back to the build directory
cd build

# Variables
echo "version: $version"
src_agent_bin_name="miru"  
src_install_bin_name="miru-install"  
prod_env="prod"
dev_env="dev"
local_env="local"

package_name="miru"
build_dir=$(pwd)
deb_dir="$build_dir/deb"
device_dir=$(dirname "$build_dir")

# Get the current Git commit
current_commit=$(git rev-parse HEAD)
echo "Current commit: $current_commit"

# Clean up any previous builds
echo "Cleaning up any previous builds"
echo "-------------------------------"
rm -rf "${build_dir:?}/$prod_env" "${build_dir:?}/$dev_env" "${build_dir:?}/$local_env"
echo ""

# Determine the architecture
echo "Determining the architecture"
echo "----------------------------"
arch=$(uname -m)
if [ "$arch" = "x86_64" ]; then
    arch="amd64"
elif [ "$arch" = "aarch64" ]; then
    arch="arm64" 
fi
echo ""

# define the package creation function
create_debian_package() {
    environment=$1
    # export the environment variable so Rust compiler can use it for the build
    export ENV="$environment"

    # validate the build directory
    cur_dir=$(pwd)
    cur_dir_name=$(basename "$cur_dir")
    if [ "$cur_dir_name" != "build" ]; then
        echo "This script must be run from the 'build' directory but was run from '$cur_dir' directory for the '$environment' environment"
        exit 1
    fi

    # Create the debian package directory
    echo "Creating the debian package directory"
    echo "-------------------------------------"
    package_dir="$build_dir/$environment/$package_name"_"$version"_"$arch"
    if [ -d "$package_dir" ]; then
        rm -rf "$package_dir"
    fi
    mkdir -p "$package_dir"
    echo ""

    # Change the architecture and version in the control file
    echo "Updating the control file"
    echo "-------------------------"
    src_debian_dir=$deb_dir/DEBIAN
    src_control_file=$src_debian_dir/control
    echo "Adding the architecture to the control file..."
    if ! sed -i "/^Architecture:/c\Architecture: $arch" "$src_control_file"; then
        echo "Failed to update the architecture to the control file"
        exit 1
    fi
    echo "Adding the version to the control file..."
    if ! sed -i "/^Version:/c\Version: $version" "$src_control_file"; then
        echo "Failed to update the version to the control file"
        exit 1
    fi

    # Update the version and git commit SHA in Cargo.toml file
    cargo_toml_file=$device_dir/Cargo.toml
    echo "Updating $cargo_toml_file"
    if ! sed -i "s/^version = \".*\"/version = \"$version\"/" "$cargo_toml_file"; then
        echo "Failed to update the version in the Cargo.toml file"
        exit 1
    fi
    if ! sed -i "s/^git = \".*\"/git = \"$current_commit\"/" "$cargo_toml_file"; then
        echo "Failed to update the git commit in the Cargo.toml file"
        exit 1
    fi
    echo ""

    # Set rust constants
    echo "Setting the environment in the Rust constants"
    echo "---------------------------------------------"
    rust_env_file=$device_dir/miru/src/env.rs
    if ! sed -i "/pub const ENV: &str = /c\pub const ENV: &str = \"$environment\";" "$rust_env_file"; then
        echo "Failed to update the environment in the Rust constants"
        exit 1
    fi
    if ! sed -i "/pub const GIT_COMMIT: &str = /c\pub const GIT_COMMIT: &str = \"$current_commit\";" "$rust_env_file"; then
        echo "Failed to update the git commit in the Rust constants"
        exit 1
    fi
    echo ""

    # Copy the debian, etc, and var directories to the package directory
    echo "Copying the debian and etc directories to the package directory"
    echo "---------------------------------------------------------------"
    src_etc_dir=$deb_dir/etc
    dest_debian_dir=$package_dir/DEBIAN
    dest_etc_dir=$package_dir/etc
    cp -r "$src_debian_dir" "$dest_debian_dir"
    cp -r "$src_etc_dir" "$dest_etc_dir"

    # Add the version, environment, and Git commit to the systemd service file
    dest_systemd_service_file=$dest_etc_dir/systemd/system/miru.service
    if ! sed -i "/^# Version:/c\# Version: $version" "$dest_systemd_service_file"; then
        echo "Failed to update the version in the systemd service file"
        exit 1
    fi
    if ! sed -i "/^# Environment:/c\# Environment: $environment" "$dest_systemd_service_file"; then
        echo "Failed to update the environment in the systemd service file"
        exit 1
    fi
    if ! sed -i "/^# Git Commit:/c\# Git Commit: $current_commit" "$dest_systemd_service_file"; then
        echo "Failed to update the git commit in the systemd service file"
        exit 1
    fi
    echo ""

    # Build the latest binaries
    echo "Building the latest binaries"
    echo "----------------------------"
    cargo build --release
    echo ""

    # Move the latest built binaries to the binaries directory in the debian package
    echo "Moving the latest built binaries"
    echo "--------------------------------"
    src_dir=$device_dir/target/release
    dest_dir="$package_dir""$miru_binaries_dir"
    src_install_bin_file=$src_dir/$src_install_bin_name
    dest_install_bin_file=$dest_dir/$install_bin_name
    src_agent_bin_file=$src_dir/$src_agent_bin_name
    dest_agent_bin_file=$dest_dir/$agent_bin_name
    src_agent_start_script=$build_dir/$agent_start_script_name
    dest_agent_start_script=$dest_dir/$agent_start_script_name

    mkdir -p "$dest_dir"
    cp "$src_agent_start_script" "$dest_agent_start_script"
    cp "$src_install_bin_file" "$dest_install_bin_file"
    cp "$src_agent_bin_file" "$dest_agent_bin_file"
    echo ""

    # Build the debian package
    echo "Building the debian package"
    echo "---------------------------"
    dpkg-deb --build "$package_dir"

    echo "BUILT BINARIES FOR THE '$environment' environment"
    echo ""
}

environments="$local_env $prod_env $dev_env"
for environment in $environments; do
    create_debian_package "$environment"
done

cd ..
git restore .

