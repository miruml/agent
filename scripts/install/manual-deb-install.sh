#!/bin/sh
set -e

PRERELEASE=${1:-false}

# Configuration
DEB_PKG_NAME="miru-agent"
GITHUB_REPO="miruml/agent"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
log() { echo "${GREEN}==>${NC} $1"; }
warn() { echo "${YELLOW}Warning:${NC} $1"; }
error() { echo "${RED}Error:${NC} $1"; exit 1; }

# Check if command exists
command_exists() { 
    command -v "$1" >/dev/null 2>&1
}

# Verify SHA256 checksum
verify_checksum() {
    file=$1
    expected_checksum=$2

    if command_exists sha256sum; then
        # use printf here for precise control over the spaces since sha256sum requires exactly two spaces in between the checksum and the file
        printf "%s  %s\n" "$expected_checksum" "$file" | sha256sum -c >/dev/null 2>&1 || {
            warn "Checksum verification failed using sha256sum"
            return 1
        }
    elif command_exists shasum; then
        printf "%s  %s\n" "$expected_checksum" "$file" | shasum -a 256 -c >/dev/null 2>&1 || {
            warn "Checksum verification failed using shasum"
            return 1
        }
    else
        warn "Could not verify checksum: no sha256sum or shasum command found"
        return 0
    fi
}

# Check for required commands
for cmd in curl tar grep cut; do
    command_exists "$cmd" || error "$cmd is required but not installed."
done

# Determine OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
DEB_ARCH=$(uname -m)

# Get latest version
if [ "$PRERELEASE" = true ]; then
    log "Fetching latest pre-release version..."
    VERSION=$(curl -sL "https://api.github.com/repos/${GITHUB_REPO}/releases" | 
        jq -r '.[] | select(.prerelease==true) | .tag_name' | head -n 1) || error "Failed to fetch latest pre-release version"
else
    log "Fetching latest stable version..."
    VERSION=$(curl -sL "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" | 
        grep "tag_name" | cut -d '"' -f 4) || error "Failed to fetch latest version"
fi

[ -z "$VERSION" ] && error "Could not determine latest version"
log "Latest version: ${VERSION}"

# Convert architecture names
case $DEB_ARCH in
    x86_64|amd64) DEB_ARCH="amd64" ;;
    aarch64|arm64) DEB_ARCH="arm64" ;;
    *) error "Unsupported architecture: $DEB_ARCH" ;;
esac

# Set download URL based on OS
case $OS in
    linux) OS="Linux" ;;
    *) error "Unsupported operating system: $OS" ;;
esac

VERSION_WO_V=$(echo "$VERSION" | cut -d 'v' -f 2)
URL="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/${DEB_PKG_NAME}_${VERSION_WO_V}_${DEB_ARCH}.deb"
CHECKSUM_URL="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/agent_${VERSION_WO_V}_checksums.txt"

# Create temporary directory
DOWNLOAD_DIR=~/.miru/downloads
rm -rf "$DOWNLOAD_DIR"
mkdir -p "$DOWNLOAD_DIR"

# Add as helper function
download_with_progress() {
    url="$1"
    output="$2"
    curl -#fL "$url" -o "$output"
}

# Download files
log "Downloading Miru Agent ${VERSION}..."
download_with_progress "$URL" "$DOWNLOAD_DIR/${DEB_PKG_NAME}.deb" ||
    error "Failed to download ${DEB_PKG_NAME}"

# Download and verify checksum if available
if curl -fsSL "$CHECKSUM_URL" -o "$DOWNLOAD_DIR/checksums.txt" 2>/dev/null; then
    log "Verifying checksum..."
    EXPECTED_CHECKSUM=$(grep "${DEB_PKG_NAME}_${VERSION_WO_V}_${DEB_ARCH}.deb" "$DOWNLOAD_DIR/checksums.txt" | cut -d ' ' -f 1)
    if [ -n "$EXPECTED_CHECKSUM" ]; then
        verify_checksum "$DOWNLOAD_DIR/${DEB_PKG_NAME}.deb" "$EXPECTED_CHECKSUM" ||
            error "Checksum verification failed"
    else
        warn "Checksum not found in checksums.txt"
    fi
else
    warn "Checksums file not available, skipping verification"
fi

# Extract archive
log "Installing..."
sudo dpkg -i "$DOWNLOAD_DIR/${DEB_PKG_NAME}.deb" || 
    error "Failed to install"

# Remove the downloaded files
rm -rf "$DOWNLOAD_DIR"

exit 0
