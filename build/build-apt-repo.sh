#!/bin/bash
set -euo pipefail

# Configuration
ARCHS=("amd64" "arm64") # Add more architectures if needed
DIST="stable"
REPO_DIR="apt-repo"
DEB_DIR="dist" # where GoReleaser puts .deb files

# Ensure version is set
if [[ -z "${GORELEASER_CURRENT_TAG:-}" ]]; then
  echo "Error: GORELEASER_CURRENT_TAG is not set. Are you running outside GoReleaser?"
  exit 1
fi
VERSION="$GORELEASER_CURRENT_TAG"

# Clean previous repo
rm -rf "$REPO_DIR"

# Process each architecture
for ARCH in "${ARCHS[@]}"; do
  echo "Processing architecture: $ARCH"

  mkdir -p "$REPO_DIR/binary-$ARCH"

  # Copy only .deb files matching architecture
  find "$DEB_DIR" -type f -name "*_${ARCH}.deb" -exec cp {} "$REPO_DIR/binary-$ARCH/" \;

  # Generate Packages.gz for this architecture
  cd "$REPO_DIR/binary-$ARCH"
  dpkg-scanpackages . /dev/null | gzip -9c > Packages.gz
  cd - >/dev/null
done

# Create the Release file (listing all architectures)
ARCHITECTURES=$(IFS=' '; echo "${ARCHS[*]}")

cat > "$REPO_DIR/Release" <<EOF
Origin: miru
Label: miru
Suite: $DIST
Version: $VERSION
Codename: $DIST
Architectures: $ARCHITECTURES
Components: main
Description: miru agent repository
EOF

# Optionally: GPG-sign the Release file
# gpg --default-key "your@email.com" -abs -o "$REPO_DIR/Release.gpg" "$REPO_DIR/Release"

echo "APT repository successfully built in $REPO_DIR/ with version $VERSION"
