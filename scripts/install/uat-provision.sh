#!/bin/sh
set -e

# Script: uat-provision.sh
# Jinja Template: provision.j2
# Build Timestamp: 2025-10-18T14:40:04.579507
# Description: Provision a device & install the miru agent in the UAT environment

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
    esac
done

PRERELEASE=false
for arg in "$@"; do
    case $arg in
    --prerelease=*) PRERELEASE="${arg#*=}";;
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

DEVICE_NAME=$(hostname)
for arg in "$@"; do
    case $arg in
    --device-name=*) DEVICE_NAME="${arg#*=}";;
    esac
done
if [ "$DEBUG" = true ]; then
    debug "device-name: '$DEVICE_NAME' (should be the name of the device)"
fi

ALLOW_REACTIVATION=false
for arg in "$@"; do
    case $arg in
    --allow-reactivation=*) ALLOW_REACTIVATION="${arg#*=}";;
    esac
done
if [ "$DEBUG" = true ]; then
    debug "allow-reactivation: '$ALLOW_REACTIVATION' (should be true or false)"
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
CHECKSUMS_NAME="checksums.txt"

# MAIN LOGIC #
# ========== #
DEB_ARCH=$ARCH
case $DEB_ARCH in
    x86_64|amd64) DEB_ARCH="amd64" ;;
    aarch64|arm64) DEB_ARCH="arm64" ;;
    *) fatal "Unsupported architecture: $DEB_ARCH" ;;
esac

# PROVISION THE DEVICE #
# --------------------- #
if [ -z "$MIRU_API_KEY" ]; then
    echo "MIRU_API_KEY is not set"
    exit 1
fi

response_body=$(curl --request POST \
  --url "$BACKEND_HOST"/v1/devices \
  --header 'Content-Type: application/json' \
  --header "X-API-Key: $MIRU_API_KEY" \
  --data "{
  \"name\": \"$DEVICE_NAME\"
}" \
  --write-out "\n%{http_code}" \
  --silent)

# Extract HTTP status code (last line) and response body (everything else)
http_code=$(echo "$response_body" | tail -n1)
response_body=$(echo "$response_body" | head -n -1)

# Check if the request succeeded
if [ "$http_code" -eq 200 ] || [ "$http_code" -eq 201 ]; then
    log "Created device '$DEVICE_NAME'"
    device="$response_body"
elif [ "$http_code" -eq 409 ]; then
    log "Device '$DEVICE_NAME' already exists"
    # Search for the device by name
    response_body=$(curl --request GET \
    --url "$BACKEND_HOST"/v1/devices?name="$DEVICE_NAME" \
    --header "X-API-Key: $MIRU_API_KEY" \
    --write-out "\n%{http_code}" \
    --silent)

    http_code=$(echo "$response_body" | tail -n1)
    response_body=$(echo "$response_body" | head -n -1)

    # check there is only one device
    if [ "$(echo "$response_body" | jq -r '.data | length')" -ne 1 ]; then
        error "Expected exactly one device with name '$DEVICE_NAME'. Instead got:"
        fatal "$response_body"
    fi

    # Extract the first device from the array since the endpoint returns a list
    device=$(echo "$response_body" | jq -r '.data[0]')
else
    error "Device creation failed (HTTP status $http_code)"
    error "Response body:"
    fatal "$response_body"
fi

device_id=$(echo "$device" | jq -r '.id')
device_name=$(echo "$device" | jq -r '.name')


log "Creating activation token for device '$device_name'"
log "Allow reactivation: $ALLOW_REACTIVATION (must be true if the device has been activated before)"
response_body=$(curl --request POST \
  --url "$BACKEND_HOST"/v1/devices/"$device_id"/activation_token \
  --header "X-API-Key: $MIRU_API_KEY" \
  --data "{
  \"allow_reactivation\": $ALLOW_REACTIVATION
}" \
  --write-out "\n%{http_code}" \
  --silent)

# Extract HTTP status code (last line) and response body (everything else)
http_code=$(echo "$response_body" | tail -n1)
response_body=$(echo "$response_body" | head -n -1)

# Check if the request succeeded
if [ "$http_code" -eq 200 ] || [ "$http_code" -eq 201 ]; then
    log "Successfully created activation token"
    MIRU_ACTIVATION_TOKEN=$(echo "$response_body" | jq -r '.token')
else
    error "Activation token request failed (HTTP status $http_code)"
    error "Response body:"
    fatal "$response_body"
fi

# DETERMINE THE VERSION #
# --------------------- #
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
VERSION=$(echo "$VERSION" | cut -d 'v' -f 2)
[ -z "$VERSION" ] && fatal "Could not determine latest version"

# Validate the version is supported
MAJOR=$(echo "$VERSION" | cut -d '.' -f 1)
MINOR=$(echo "$VERSION" | cut -d '.' -f 2)
PATCH=$(echo "$VERSION" | cut -d '.' -f 3 | sed 's/[^0-9].*//')
if ! echo "$MAJOR" | grep -q '^[0-9]\+$' || ! echo "$MINOR" | grep -q '^[0-9]\+$' || ! echo "$PATCH" | grep -q '^[0-9]\+$'; then
    fatal "Could not parse version '$VERSION' to determine if it is supported"
else
    if [ "$MAJOR" -lt 0 ] || [ "$MAJOR" -eq 0 ] && [ "$MINOR" -lt 5 ] || [ "$MAJOR" -eq 0 ] && [ "$MINOR" -eq 5 ] && [ "$PATCH" -lt 2 ]; then
        fatal "Version v$VERSION has been deprecated, please install v0.5.2 or greater"
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
if [ -n "$INSTALLED_VERSION" ]; then
    log "Version ${INSTALLED_VERSION} is currently installed"
else
    log "miru agent is not currently installed"
fi

CHECKSUMS_FILE="$DOWNLOAD_DIR/${CHECKSUMS_NAME}"
AGENT_DEB_PKG="$DOWNLOAD_DIR/${AGENT_DEB_PKG_NAME}.deb"
if [ "$INSTALLED_VERSION" != "$VERSION" ]; then
    mkdir -p "$DOWNLOAD_DIR"

    if [ -n "$INSTALLED_VERSION" ]; then
        log "Downloading version ${VERSION} to replace version ${INSTALLED_VERSION}"
    else
        log "Downloading version ${VERSION}"
    fi

    AGENT_DEB_PKG_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${AGENT_DEB_PKG_NAME}_${VERSION}_${DEB_ARCH}.deb"
    CHECKSUM_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/agent_${VERSION}_checksums.txt"

    # download the checksums file
    curl -fsSL "$CHECKSUM_URL" -o "$CHECKSUMS_FILE" || fatal "Failed to download checksums.txt"

    curl -#fL "$AGENT_DEB_PKG_URL" -o "$AGENT_DEB_PKG" ||
        fatal "Failed to download ${AGENT_DEB_PKG_NAME}"
    EXPECTED_CHECKSUM=$(grep "${AGENT_DEB_PKG_NAME}_${VERSION}_${DEB_ARCH}.deb" "$CHECKSUMS_FILE" | cut -d ' ' -f 1)
    if [ -n "$EXPECTED_CHECKSUM" ]; then
        verify_checksum "$AGENT_DEB_PKG" "$EXPECTED_CHECKSUM" ||
            fatal "Checksum verification failed"
    else
        fatal "Checksums not found inside $CHECKSUM_URL" 
    fi

    log "Installing version ${VERSION}"
    sudo dpkg -i "$AGENT_DEB_PKG" || fatal "Failed to install the agent"

    log "Removing downloaded files"
    rm -rf "$DOWNLOAD_DIR"
else 
    log "Skipping download of version ${VERSION} because it is already installed"
fi

# ACTIVATE THE AGENT #
# ------------------ #
cleanup() {
    exit_code=$?

    # restart the agent
    log "Restarting the miru agent"
    sudo systemctl restart miru >/dev/null 2>&1

    exit $exit_code
}

trap cleanup EXIT INT TERM QUIT HUP

log "Activating the miru agent..."
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

if [ "$MIRU_ACTIVATION_TOKEN" = "" ]; then
    fatal "The MIRU_ACTIVATION_TOKEN environment variable is not set"
fi

# Execute the installer
sudo -u miru -E env MIRU_ACTIVATION_TOKEN="$MIRU_ACTIVATION_TOKEN" /usr/sbin/miru-agent --install $args
exit 0