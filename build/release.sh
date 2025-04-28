#!/bin/bash
set -e

# Build the deb package
goreleaser release --snapshot --clean

# Build the deb package
./build-apt-repo.sh