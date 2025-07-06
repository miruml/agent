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

### COPIED ARGUMENT UTILITIES END ###

# CLI args
DEBUG=$(debug_flag --default=false "$@")
if [ "$DEBUG" = true ]; then
    debug "Script: prerelease-install.sh"
fi
BRANCH=$(git_branch --default=main "$@")
if [ "$DEBUG" = true ]; then
    print_git_branch "$BRANCH"
fi
PRERELEASE=true
if [ "$DEBUG" = true ]; then
    print_prerelease_flag "$PRERELEASE"
fi
BACKEND_BASE_URL=$(backend_base_url --default="" "$@")
if [ "$DEBUG" = true ]; then
    print_backend_base_url "$BACKEND_BASE_URL"
fi
MQTT_BROKER_HOST=$(mqtt_broker_host --default="" "$@")
if [ "$DEBUG" = true ]; then
    print_mqtt_broker_host "$MQTT_BROKER_HOST"
fi

curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$BRANCH"/scripts/install/install.sh | sh -s -- \
--debug="$DEBUG" \
--prerelease="$PRERELEASE" \
--git-branch="$BRANCH" \
--backend-base-url="$BACKEND_BASE_URL" \
--mqtt-broker-host="$MQTT_BROKER_HOST"
