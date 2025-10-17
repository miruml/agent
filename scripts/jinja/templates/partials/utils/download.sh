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