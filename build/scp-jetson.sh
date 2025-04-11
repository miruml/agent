#!/bin/bash
echo ''
set -e # Exit on error

source "$(pwd)/vars.sh"

# Variables
echo "version: $version"
prod_env="prod"
dev_env="dev"

build_dir=$(pwd)
# jetson_build_dir=/home/miru/miru/device/build

# copy over the dev files
read -rp "Enter <device-user-name>@<ip-address> for the device: " device 
read -rp "Enter the build directory path on the jetson: " jetson_build_dir
scp "$device":"$jetson_build_dir"/"$dev_env"/miru_"$version"_arm64.deb "$build_dir"/"$dev_env"/miru_"$version"_arm64.deb
scp "$device":"$jetson_build_dir"/"$dev_env"/miru_"$version"_arm64/usr/local/miru/miru-"$version" "$build_dir"/"$dev_env"/miru_"$version"_arm64

# rename the dev folder to allow the binary to be copied to the dev folder root
if [ -d "$build_dir"/"$dev_env"/miru_"$version"_amd64 ]; then
    mv "$build_dir"/"$dev_env"/miru_"$version"_amd64 "$build_dir"/"$dev_env"/deb_pkg
fi
cp "$build_dir"/"$dev_env"/deb_pkg/usr/local/miru/miru-"$version" "$build_dir"/"$dev_env"/miru_"$version"_amd64

# copy over the prod files
scp "$device":"$jetson_build_dir"/"$prod_env"/miru_"$version"_arm64.deb "$build_dir"/"$prod_env"/miru_"$version"_arm64.deb
scp "$device":"$jetson_build_dir"/"$prod_env"/miru_"$version"_arm64/usr/local/miru/miru-"$version" "$build_dir"/"$prod_env"/miru_"$version"_arm64

# rename the dev folder to allow the binary to be copied to the dev folder root
if [ -d "$build_dir"/"$prod_env"/miru_"$version"_amd64 ]; then
    mv "$build_dir"/"$prod_env"/miru_"$version"_amd64 "$build_dir"/"$prod_env"/deb_pkg
fi
cp "$build_dir"/"$prod_env"/deb_pkg/usr/local/miru/miru-"$version" "$build_dir"/"$prod_env"/miru_"$version"_amd64
# rename the prod folder to allow the binary to be copied to the prod folder root