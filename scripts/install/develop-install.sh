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
DEBUG=$(debug_flag --default=true "$@")
if [ "$DEBUG" = true ]; then
    debug "Script: develop-install.sh"
fi
BRANCH=$(git_branch --default=dev "$@")
if [ "$DEBUG" = true ]; then
    print_git_branch "$BRANCH"
fi
PRERELEASE=$(prerelease_flag --default=true "$@")
if [ "$DEBUG" = true ]; then
    print_prerelease_flag "$PRERELEASE"
fi
BACKEND_HOST=$(backend_host --default="https://configs.dev.api.miruml.com" "$@")
if [ "$DEBUG" = true ]; then
    print_backend_host "$BACKEND_HOST"
fi
MQTT_BROKER_HOST=$(mqtt_broker_host --default="dev.mqtt.miruml.com" "$@")
if [ "$DEBUG" = true ]; then
    print_mqtt_broker_host "$MQTT_BROKER_HOST"
fi
DEVICE_NAME=$(device_name "$@")
if [ "$DEBUG" = true ]; then
    print_device_name "$DEVICE_NAME"
fi
if [ "$DEBUG" = true ]; then
    report_token_existence
fi
VERSION=$(version_flag "$@")
if [ "$DEBUG" = true ]; then
    print_version_flag "$VERSION"
fi



MIRU_ACTIVATION_TOKEN=$MIRU_ACTIVATION_TOKEN curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$BRANCH"/scripts/install/install.sh | sh -s -- \
--debug="$DEBUG" \
--git-branch="$BRANCH" \
--prerelease="$PRERELEASE" \
--backend-host="$BACKEND_HOST" \
--mqtt-broker-host="$MQTT_BROKER_HOST" \
--device-name="$DEVICE_NAME" \
--version="$VERSION"

