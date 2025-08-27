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
error() { echo "${RED}Error:${NC} $1"; }
fatal() { echo "${RED}Error:${NC} $1"; exit 1; }

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
backend_host() {
    backend_host=$(default_value "" "$@")
    for arg in "$@"; do
        case $arg in
        --backend-host=*) backend_host="${arg#*=}";;
        esac
    done
    echo "$backend_host"
}

print_backend_host() {
    backend_host=$1
    debug "Backend Host: '$backend_host'"
}

# MQTT Broker Host
mqtt_broker_host() {
    mqtt_broker_host=$(default_value "" "$@")
    for arg in "$@"; do
        case $arg in
        --mqtt-broker-host=*) mqtt_broker_host="${arg#*=}";;
        esac
    done
    echo "$mqtt_broker_host"
}

print_mqtt_broker_host() {
    mqtt_broker_host=$1
    debug "MQTT Broker Host: '$mqtt_broker_host'"
}

device_name() {
    default_device_name=$(hostname)
    device_name=$(default_value "$default_device_name" "$@")
    for arg in "$@"; do
        case $arg in
        --device-name=*) device_name="${arg#*=}";;
        esac
    done
    echo "$device_name"
}

print_device_name() {
    device_name=$1
    debug "Device Name: '$device_name'"
}

# Token
report_token_existence() {
    if [ -n "$MIRU_ACTIVATION_TOKEN" ]; then
        debug "Activation token IS provided"
    else
        debug "Activation token IS NOT provided"
    fi
}

# version flag
version_flag() {
    version_flag=$(default_value "" "$@")
    for arg in "$@"; do
        case $arg in
        --version=*) version_flag="${arg#*=}";;
        esac
    done
    echo "$version_flag"
}

print_version_flag() {
    version_flag=$1
    debug "Version flag: '$version_flag' (should be a semantic version string like 'v1.2.3')"
}

### COPIED ARGUMENT UTILITIES END ###

# CLI args
DEBUG=$(debug_flag "$@")
if [ "$DEBUG" = true ]; then
    debug "Script: deb-install.sh"
fi
PRERELEASE=$(prerelease_flag "$@")
if [ "$DEBUG" = true ]; then
    print_prerelease_flag "$PRERELEASE"
fi
VERSION=$(version_flag "$@")
if [ "$DEBUG" = true ]; then
    print_version_flag "$VERSION"
fi


# Configuration
DEB_PKG_NAME="miru-agent"
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
    command_exists "$cmd" || fatal "$cmd is required but not installed."
done

# Determine OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
DEB_ARCH=$(uname -m)

# Get the version
if [ "$VERSION" = "" ]; then
    if [ "$PRERELEASE" = true ]; then
        log "Fetching latest pre-release version..."
        VERSION=$(curl -sL "https://api.github.com/repos/${GITHUB_REPO}/releases" | 
            jq -r '.[] | select(.prerelease==true) | .tag_name' | head -n 1) || fatal "Failed to fetch latest pre-release version"
    else
        log "Fetching latest stable version..."
        VERSION=$(curl -sL "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" | 
            grep "tag_name" | cut -d '"' -f 4) || fatal "Failed to fetch latest version"
    fi
fi

[ -z "$VERSION" ] && fatal "Could not determine latest version"
log "Version: ${VERSION}"

# Convert architecture names
case $DEB_ARCH in
    x86_64|amd64) DEB_ARCH="amd64" ;;
    aarch64|arm64) DEB_ARCH="arm64" ;;
    *) fatal "Unsupported architecture: $DEB_ARCH" ;;
esac

# Set download URL based on OS
case $OS in
    linux) OS="Linux" ;;
    *) fatal "Unsupported operating system: $OS" ;;
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
    fatal "Failed to download ${DEB_PKG_NAME}"

# Download and verify checksum if available
if curl -fsSL "$CHECKSUM_URL" -o "$DOWNLOAD_DIR/checksums.txt" 2>/dev/null; then
    log "Verifying checksum..."
    EXPECTED_CHECKSUM=$(grep "${DEB_PKG_NAME}_${VERSION_WO_V}_${DEB_ARCH}.deb" "$DOWNLOAD_DIR/checksums.txt" | cut -d ' ' -f 1)
    if [ -n "$EXPECTED_CHECKSUM" ]; then
        verify_checksum "$DOWNLOAD_DIR/${DEB_PKG_NAME}.deb" "$EXPECTED_CHECKSUM" ||
            fatal "Checksum verification failed"
    else
        warn "Checksum not found in checksums.txt"
    fi
else
    warn "Checksums file not available, skipping verification"
fi

# Extract archive
log "Installing..."
sudo dpkg -i "$DOWNLOAD_DIR/${DEB_PKG_NAME}.deb" || 
    fatal "Failed to install"

# Remove the downloaded files
rm -rf "$DOWNLOAD_DIR"

exit 0
