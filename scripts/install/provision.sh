#!/bin/bash
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
    device_name=$(default_value "" "$@")
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
version() {
    version=$(default_value "" "$@")
    for arg in "$@"; do
        case $arg in
        --version=*) version="${arg#*=}";;
        esac
    done
    echo "$version"
}

print_version() {
    version=$1
    debug "Version: '$version' (should be a semantic version string like 'v1.2.3')"
}

# allow reactivation flag
allow_reactivation_flag() {
    allow_reactivation_flag=$(default_value false "$@")
    for arg in "$@"; do
        case $arg in
        --allow-reactivation) allow_reactivation_flag=true;;
        --allow-reactivation=*) allow_reactivation_flag="${arg#*=}";;
        esac
    done
    echo "$allow_reactivation_flag"
}

print_allow_reactivation_flag() {
    allow_reactivation_flag=$1
    debug "Allow reactivation: '$allow_reactivation_flag' (should be true or false)"
}

### COPIED ARGUMENT UTILITIES END ###


# check the the $MIRU_API_KEY is set
if [ -z "$MIRU_API_KEY" ]; then
    echo "MIRU_API_KEY is not set"
    exit 1
fi

install_script_file() {
    install_script_file=$(default_value "install.sh" "$@")
    for arg in "$@"; do
        case $arg in
        --install-script-file=*) install_script_file="${arg#*=}";;
        esac
    done
    echo "$install_script_file"
}

BACKEND_HOST=$(backend_host --default="https://configs.api.miruml.com" "$@")
DEVICE_NAME=$(device_name --default="$(hostname)" "$@")
ALLOW_REACTIVATION=$(allow_reactivation_flag --default=false "$@")

# create the device
echo "Provisioning Device"
echo "==================="

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
    activation_token=$(echo "$response_body" | jq -r '.token')
else
    error "Activation token request failed (HTTP status $http_code)"
    error "Response body:"
    fatal "$response_body"
fi

DEBUG=$(debug_flag "$@")
BRANCH=$(git_branch "$@")
PRERELEASE=$(prerelease_flag "$@")
INSTALL_SCRIPT_FILE=$(install_script_file "$@")
VERSION=$(version "$@")

# install the agent onto the device
export MIRU_ACTIVATION_TOKEN=$activation_token
curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$BRANCH"/scripts/install/"$INSTALL_SCRIPT_FILE" | sh -s -- \
--debug="$DEBUG" \
--git-branch="$BRANCH" \
--prerelease="$PRERELEASE" \
--backend-host="$BACKEND_HOST" \
--device-name="$DEVICE_NAME" \
--version="$VERSION"
