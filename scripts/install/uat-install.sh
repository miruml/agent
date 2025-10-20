#!/bin/sh
set -e

# Script: uat-install.sh
# Jinja Template: install.j2
# Build Timestamp: 2025-10-19T20:44:31.182247
# Description: Install the Miru Agent in the UAT environment

# DISPLAY #
# ======= #
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NO_COLOR='\033[0m'

debug() { echo "${BLUE}==>${NO_COLOR} $1"; }
log() { echo "${GREEN}==>${NO_COLOR} $1"; }
warn() { echo "${YELLOW}Warning:${NO_COLOR} $1"; }
error() { echo "${RED}Error:${NO_COLOR} $1"; }
fatal() { echo "${RED}Error:${NO_COLOR} $1"; exit 1; }

# ARGUMENTS #
# ========= #
DEBUG=false
for arg in "$@"; do
    case $arg in
    --debug=*) DEBUG="${arg#*=}";;
    --debug) DEBUG=true;;
    esac
done

PRERELEASE=false
for arg in "$@"; do
    case $arg in
    --prerelease=*) PRERELEASE="${arg#*=}";;
    --prerelease) PRERELEASE=true;;
    esac
done
if [ "$DEBUG" = true ]; then
    debug "prerelease: '$PRERELEASE' (should be true or false)"
fi

VERSION=''
for arg in "$@"; do
    case $arg in
    --version=*) VERSION="${arg#*=}";;
    esac
done
if [ "$DEBUG" = true ]; then
    debug "version: '$VERSION' (should be a semantic version string like 'v1.2.3')"
fi

DEVICE_NAME=''
for arg in "$@"; do
    case $arg in
    --device-name=*) DEVICE_NAME="${arg#*=}";;
    esac
done
if [ "$DEBUG" = true ]; then
    debug "device-name: '$DEVICE_NAME' (should be the name of the device)"
fi

FROM_PKG=''
for arg in "$@"; do
    case $arg in
    --from-pkg=*) FROM_PKG="${arg#*=}";;
    esac
done
if [ "$DEBUG" = true ]; then
    debug "from-pkg: '$FROM_PKG' (should be the path to the agent package on this machine)"
fi

BACKEND_HOST="https://uat.api.miruml.com"
for arg in "$@"; do
    case $arg in
    --backend-host=*) BACKEND_HOST="${arg#*=}";;
    esac
done
if [ "$DEBUG" = true ]; then
    debug "backend-host: '$BACKEND_HOST' (should be the URL of the backend host)"
fi

MQTT_BROKER_HOST="uat.mqtt.miruml.com"
for arg in "$@"; do
    case $arg in
    --mqtt-broker-host=*) MQTT_BROKER_HOST="${arg#*=}";;
    esac
done
if [ "$DEBUG" = true ]; then
    debug "mqtt-broker-host: '$MQTT_BROKER_HOST' ()"
fi

# UTILITIES #
# ========= #
cmd_exists() { 
    command -v "$1" >/dev/null 2>&1
}

for cmd in curl grep cut jq; do
    cmd_exists "$cmd" || fatal "$cmd is required but not installed"
done


verify_checksum() {
    file=$1
    expected_checksum=$2

    if [ -z "$expected_checksum" ]; then
        fatal "Expected checksum is required but not provided"
    fi
    if [ -z "$file" ]; then
        fatal "File is required but not provided"
    fi

    if cmd_exists sha256sum; then
        # use printf here for precise control over the spaces since sha256sum requires exactly two spaces in between the checksum and the file
        printf "%s  %s\n" "$expected_checksum" "$file" | sha256sum -c >/dev/null 2>&1 || {
            warn "Checksum verification failed using sha256sum"
            return 1
        }
    elif cmd_exists shasum; then
        printf "%s  %s\n" "$expected_checksum" "$file" | shasum -a 256 -c >/dev/null 2>&1 || {
            warn "Checksum verification failed using shasum"
            return 1
        }
    else
        warn "Could not verify checksum: no sha256sum or shasum command found"
        return 0
    fi
}

# VARIABLES #
# ========= #
ARCH="$(uname -m)"
DOWNLOAD_DIR="$HOME/.miru/downloads"
AGENT_DEB_PKG_NAME="miru-agent"
GITHUB_REPO="miruml/agent"
CHECKSUMS_FILE="$DOWNLOAD_DIR/checksums.txt"
DEB_PKG_MIME_TYPE="application/vnd.debian.binary-package"

# MAIN LOGIC #
# ========== #
DEB_ARCH=$ARCH
case $DEB_ARCH in
    x86_64|amd64) DEB_ARCH="amd64" ;;
    aarch64|arm64) DEB_ARCH="arm64" ;;
    *) fatal "Unsupported architecture: $DEB_ARCH" ;;
esac

# USE PROVIDED PACKAGE #
# -------------------- #
if [ -n "$FROM_PKG" ]; then
    log "Installing from package on local machine: '$FROM_PKG'"
    if [ ! -f "$FROM_PKG" ]; then
        fatal "The provided package does not exist on this machine: '$FROM_PKG'"
    fi
    if [ "$(file -b --mime-type "$FROM_PKG")" != "$DEB_PKG_MIME_TYPE" ]; then
        fatal "The provided package is not a valid Debian package. Expected mimetype '$DEB_PKG_MIME_TYPE' but got '$(file -b --mime-type "$FROM_PKG")'."
    fi
    if [ "$(dpkg -f "$FROM_PKG" Package)" != "$AGENT_DEB_PKG_NAME" ]; then
        fatal "The provided package is not a valid Miru Agent package. Expected package name '$AGENT_DEB_PKG_NAME' but got '$(dpkg -f "$FROM_PKG" Package)'."
    fi
    if [ "$(dpkg -f "$FROM_PKG" Architecture)" != "$DEB_ARCH" ]; then
        fatal "The provided package architecture ($(dpkg -f "$FROM_PKG" Architecture)) does not match this machine's architecture ($DEB_ARCH)."
    fi
    AGENT_DEB_PKG=$FROM_PKG

    VERSION=$(dpkg -f "$FROM_PKG" Version)
fi

# DETERMINE THE VERSION #
# --------------------- #
if [ -z "$VERSION" ]; then
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
VERSION=$(echo "$VERSION" | cut -d 'v' -f 2)
[ -z "$VERSION" ] && fatal "Could not determine latest version"

# Validate the version is supported
MAJOR=$(echo "$VERSION" | cut -d '.' -f 1)
MINOR=$(echo "$VERSION" | cut -d '.' -f 2)
PATCH=$(echo "$VERSION" | cut -d '.' -f 3 | sed 's/[^0-9].*//')
if ! echo "$MAJOR" | grep -q '^[0-9]\+$' || ! echo "$MINOR" | grep -q '^[0-9]\+$' || ! echo "$PATCH" | grep -q '^[0-9]\+$'; then
    fatal "Could not parse version '$VERSION' to determine if it is supported"
else
    if [ "$MAJOR" -lt 0 ] || [ "$MAJOR" -eq 0 ] && [ "$MINOR" -lt 6 ]; then
        fatal "Version v$VERSION has been deprecated, please install v0.6.0 or greater"
    fi
fi
log "Version to install: ${VERSION}"

# DOWNLOAD THE AGENT #
# ------------------ #
INSTALLED_VERSION=$(dpkg-query -W -f='${Version}' "$AGENT_DEB_PKG_NAME" 2>/dev/null || echo "")
# replace '~' with '-' 
if [ -n "$INSTALLED_VERSION" ]; then
    INSTALLED_VERSION=$(echo "$INSTALLED_VERSION" | sed 's/~/-/g')
fi

if [ "$INSTALLED_VERSION" != "$VERSION" ]; then
    rm -rf "$DOWNLOAD_DIR"
    mkdir -p "$DOWNLOAD_DIR"

    # download the agent deb package if not provided locally
    if [ -z "$AGENT_DEB_PKG" ] || [ ! -f "$AGENT_DEB_PKG" ]; then
        log "Downloading version ${VERSION}"
        AGENT_DEB_PKG="$DOWNLOAD_DIR/${AGENT_DEB_PKG_NAME}.deb"
        AGENT_DEB_PKG_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${AGENT_DEB_PKG_NAME}_${VERSION}_${DEB_ARCH}.deb"
        curl -#fL "$AGENT_DEB_PKG_URL" -o "$AGENT_DEB_PKG" ||
            fatal "Failed to download ${AGENT_DEB_PKG_NAME}"
    fi

    # download the checksums file
    CHECKSUM_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/agent_${VERSION}_checksums.txt"
    curl -fsSL "$CHECKSUM_URL" -o "$CHECKSUMS_FILE" || fatal "Failed to download checksums.txt"

    EXPECTED_CHECKSUM=$(grep "${AGENT_DEB_PKG_NAME}_${VERSION}_${DEB_ARCH}.deb" "$CHECKSUMS_FILE" | cut -d ' ' -f 1)
    if [ -n "$EXPECTED_CHECKSUM" ]; then
        verify_checksum "$AGENT_DEB_PKG" "$EXPECTED_CHECKSUM" ||
            fatal "Checksum verification failed"
    else
        fatal "Checksums not found inside $CHECKSUM_URL" 
    fi

    if [ -n "$INSTALLED_VERSION" ]; then
        log "Replacing version ${INSTALLED_VERSION} with version ${VERSION}"
    else
        log "Installing version ${VERSION}"
    fi
    sudo dpkg -i "$AGENT_DEB_PKG" || fatal "Failed to install the agent"

    log "Removing downloaded files"
    rm -rf "$DOWNLOAD_DIR"
else 
    log "Version ${VERSION} is already installed"
fi

# ACTIVATE THE AGENT #
# ------------------ #
cleanup() {
    exit_code=$?

    # restart the agent
    log "Restarting the Miru Agent"
    sudo systemctl restart miru >/dev/null 2>&1

    exit $exit_code
}

trap cleanup EXIT INT TERM QUIT HUP

log "Activating the Miru Agent..."
if systemctl is-active --quiet miru; then
    log "Stopping the currently running agent"
    sudo systemctl stop miru >/dev/null 2>&1
fi

# Collect the arguments
args=""
args="$args --backend-host=$BACKEND_HOST"
args="$args --mqtt-broker-host=$MQTT_BROKER_HOST"
if [ -n "$DEVICE_NAME" ]; then
    args="$args --device-name=$DEVICE_NAME"
fi

if [ -z "$MIRU_ACTIVATION_TOKEN" ]; then
    fatal "The MIRU_ACTIVATION_TOKEN environment variable is not set"
fi

# Execute the installer
sudo -u miru -E env MIRU_ACTIVATION_TOKEN="$MIRU_ACTIVATION_TOKEN" /usr/sbin/miru-agent --install $args
exit 0