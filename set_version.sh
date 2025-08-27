#!/usr/bin/env bash

# Version update script for BZMM
# Usage: ./set_version.sh <new_version>

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <new_version>"
    echo ""
    echo "Example:"
    echo "  $0 0.3.2"
    exit 1
fi

NEW_VERSION="$1"

# Get current version from package.json
OLD_VERSION=$(grep '"version":' package.json | sed 's/.*"version": "\([^"]*\)".*/\1/')

if [ -z "$OLD_VERSION" ]; then
    echo "Error: Could not detect current version from package.json"
    exit 1
fi

# Validate version format (basic semver check)
if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "Error: Version must be in format X.Y.Z (e.g., 0.3.2)"
    exit 1
fi

echo "Updating version from $OLD_VERSION to $NEW_VERSION"
echo "Updating files..."

# Update package.json
echo "  Updating package.json..."
sed -i "s/\"version\": \"$OLD_VERSION\"/\"version\": \"$NEW_VERSION\"/" package.json

# Update src-tauri/Cargo.toml
echo "  Updating src-tauri/Cargo.toml..."
sed -i "s/^version = \"$OLD_VERSION\"/version = \"$NEW_VERSION\"/" src-tauri/Cargo.toml

# Update src-tauri/tauri.conf.json
echo "  Updating src-tauri/tauri.conf.json..."
sed -i "s/\"version\": \"$OLD_VERSION\"/\"version\": \"$NEW_VERSION\"/" src-tauri/tauri.conf.json

echo "Version update complete!"

# Verify the changes
echo ""
echo "Verification:"
echo "  package.json:           $(grep '"version":' package.json | tr -d ' ' | cut -d'"' -f4)"
echo "  src-tauri/Cargo.toml:   $(grep '^version =' src-tauri/Cargo.toml | cut -d'"' -f2)"
echo "  src-tauri/tauri.conf.json: $(grep '"version":' src-tauri/tauri.conf.json | tr -d ' ' | cut -d'"' -f4)"