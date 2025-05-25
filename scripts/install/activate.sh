#!/bin/sh
set -e

### COPIED ARGUMENT UTILITIES BEGIN ###

### COPIED DISPLAY UTILITIES BEGIN ###

# Colors for output
RED='[0;31m'
GREEN='[0;32m'
YELLOW='[1;33m'
BLUE='[0;34m'
NC='[0m' # No Color

# Helper functions
debug() { echo "${BLUE}==>${NC} $1"; }
log() { echo "${GREEN}==>${NC} $1"; }
warn() { echo "${YELLOW}Warning:${NC} $1"; }
error() { echo "${RED}Error:${NC} $1"; exit 1; }

### COPIED DISPLAY UTILITIES END ###

default_value() {
    default_value=${1:-}
    for arg in "$@"; do
        case $arg in
        --default=*) default_value="${arg#*=}";;
        esac
    done
    echo "$default_value"
}

# Debug flag
debug_flag() {
    debug_flag=$(default_value false "$@")
    for arg in "$@"; do
        case $arg in
        --debug) debug_flag=true;;
        --debug=*) debug_flag="${arg#*=}";;
        esac
    done
    echo "$debug_flag"
}

# Git branch
git_branch() {
    branch=$(default_value "main" "$@")
    for arg in "$@"; do
        case $arg in
        --git-branch=*) branch="${arg#*=}";;
        esac
    done
    echo "$branch"
}

print_git_branch() {
    git_branch=$1
    debug "Git Branch: '$git_branch'"
}

# Prerelease flag
prerelease_flag() {
    prerelease_flag=$(default_value false "$@")
    for arg in "$@"; do
        case $arg in
            --prerelease) prerelease_flag=true;;
            --prerelease=*) prerelease_flag="${arg#*=}";;
        esac
    done
    echo "$prerelease_flag"
}

print_prerelease_flag() {
    prerelease_flag=$1
    debug "Prerelease flag: '$prerelease_flag' (should be true or false)"
}

# Backend URL
backend_base_url() {
    backend_base_url=$(default_value "" "$@")
    for arg in "$@"; do
        case $arg in
        --backend-base-url=*) backend_base_url="${arg#*=}";;
        esac
    done
    echo "$backend_base_url"
}

print_backend_base_url() {
    backend_base_url=$1
    debug "Backend Base URL: '$backend_base_url'"
}

### COPIED ARGUMENT UTILITIES END ###

# CLI args
DEBUG=$(debug_flag --default=false "$@")
if [ "$DEBUG" = true ]; then
    debug "Script: activate.sh"
fi
PRERELEASE=$(prerelease_flag --default=false "$@")
if [ "$DEBUG" = true ]; then
    print_prerelease_flag "$PRERELEASE"
fi
BACKEND_BASE_URL=$(backend_base_url --default="" "$@")
if [ "$DEBUG" = true ]; then
    print_backend_base_url "$BACKEND_BASE_URL"
fi

# Configuration
BINARY_NAME="installer"
GITHUB_REPO="miruml/agent"

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
if [ -n "$BACKEND_BASE_URL" ]; then
    sudo -u miru ./config-agent-installer "$BACKEND_BASE_URL"
else
    sudo -u miru ./config-agent-installer
fi
cd -

# Remove the downloaded files
rm -rf "$DOWNLOAD_DIR"

exit 0
