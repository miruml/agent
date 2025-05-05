#!/bin/sh
set -e


# Configuration
BINARY_NAME="installer"
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
ARCH=$(uname -m)

# Get latest version
log "Fetching latest version..."
VERSION=$(curl -sL "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" | 
    grep "tag_name" | cut -d '"' -f 4) || error "Failed to fetch latest version"

[ -z "$VERSION" ] && error "Could not determine latest version"
log "Latest version: ${VERSION}"

# Convert architecture names
case $ARCH in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) error "Unsupported architecture: $ARCH" ;;
esac

# Set download URL based on OS
case $OS in
    linux) OS="Linux" ;;
    *) error "Unsupported operating system: $OS" ;;
esac

VERSION_WO_V=$(echo "$VERSION" | cut -d 'v' -f 2)
URL="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/${BINARY_NAME}_${OS}_${ARCH}.tar.gz"
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
log "Downloading Miru Agent Installer ${VERSION}..."
download_with_progress "$URL" "$DOWNLOAD_DIR/${BINARY_NAME}.tar.gz" ||
    error "Failed to download ${BINARY_NAME}"

# Download and verify checksum if available
if curl -fsSL "$CHECKSUM_URL" -o "$DOWNLOAD_DIR/checksums.txt" 2>/dev/null; then
    log "Verifying checksum..."
    EXPECTED_CHECKSUM=$(grep "${BINARY_NAME}_${OS}_${ARCH}.tar.gz" "$DOWNLOAD_DIR/checksums.txt" | cut -d ' ' -f 1)
    if [ -n "$EXPECTED_CHECKSUM" ]; then
        verify_checksum "$DOWNLOAD_DIR/${BINARY_NAME}.tar.gz" "$EXPECTED_CHECKSUM" ||
            error "Checksum verification failed"
    else
        warn "Checksum not found in checksums.txt"
    fi
else
    warn "Checksums file not available, skipping verification"
fi

# Extract archive
log "Extracting..."
tar -xzf "$DOWNLOAD_DIR/${BINARY_NAME}.tar.gz" -C "$DOWNLOAD_DIR" || 
    error "Failed to extract archive"

# Execute the installer
cd "$DOWNLOAD_DIR"
BACKEND_URL=${1:-""}
if [ -n "$BACKEND_URL" ]; then
    sudo -u miru ./config-agent-installer "$BACKEND_URL"
else
    sudo -u miru ./config-agent-installer
fi
cd -

# Remove the downloaded files
rm -rf "$DOWNLOAD_DIR"

exit 0
