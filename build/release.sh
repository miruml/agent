#!/bin/bash
set -e

# Build the deb package
goreleaser release

# Build the deb package
aptly repo add miru-agent ./dist/miru-agent-*.deb
aptly publish repo -architectures=amd64,arm64 miru-agent