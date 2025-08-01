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
backend_base_url() {
    backend_base_url=$(default_value "https://configs.api.miruml.com/v1" "$@")
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

# Token
report_token_existence() {
    if [ -n "$MIRU_ACTIVATION_TOKEN" ]; then
        debug "Activation token provided"
    else
        debug "No activation token provided"
    fi
}

### COPIED ARGUMENT UTILITIES END ###


# check the the $MIRU_API_KEY is set
if [ -z "$MIRU_API_KEY" ]; then
    echo "MIRU_API_KEY is not set"
    exit 1
fi

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

install_script_file() {
    install_script_file=$(default_value "install.sh" "$@")
    for arg in "$@"; do
        case $arg in
        --install-script-file=*) install_script_file="${arg#*=}";;
        esac
    done
    echo "$install_script_file"
}

BACKEND_BASE_URL=$(backend_base_url "$@")
DEVICE_NAME=$(device_name "$@")

# create the device
echo "Provisioning Device"
echo "==================="

log "Creating device: $DEVICE_NAME"
response_body=$(curl --request POST \
  --url $BACKEND_BASE_URL/devices \
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
    log "Device creation request succeeded (HTTP $http_code)"
    device="$response_body"
else
    error "Device creation failed with HTTP status: $http_code"
    error "Response body:"
    fatal "$response_body"
fi

device_id=$(echo $device | jq -r '.id')
device_name=$(echo $device | jq -r '.name')
log "Successfully created device: $device_name"

# fetch the activation token
log "Fetching activation token"
response_body=$(curl --request POST \
  --url $BACKEND_BASE_URL/devices/$device_id/activation_token \
  --header "X-API-Key: $MIRU_API_KEY" \
  --write-out "\n%{http_code}" \
  --silent)

# Extract HTTP status code (last line) and response body (everything else)
http_code=$(echo "$response_body" | tail -n1)
response_body=$(echo "$response_body" | head -n -1)

# Check if the request succeeded
if [ "$http_code" -eq 200 ] || [ "$http_code" -eq 201 ]; then
    log "Successfully fetched activation token (HTTP status: $http_code)"
    activation_token=$(echo "$response_body" | jq -r '.token')
else
    error "Activation token request failed with HTTP status: $http_code"
    error "Response body:"
    fatal "$response_body"
fi

GIT_BRANCH=$(git_branch "$@")
INSTALL_SCRIPT_FILE=$(install_script_file "$@")

# install the agent onto the device
export MIRU_ACTIVATION_TOKEN=$activation_token
curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$GIT_BRANCH"/scripts/install/"$INSTALL_SCRIPT_FILE" | sh
