#!/bin/bash
# 🐶 PugVault Installer

REPO="tannht/pug-vault"
VERSION="v1.1.0"
BINARY_NAME="pug-vault"
INSTALL_DIR="/usr/local/bin"

echo "🐶 Downloading PugVault $VERSION..."

# Determine OS and Arch (Simplified for Linux x64 for now)
# In a real scenario, we'd have multiple binaries in the release.
URL="https://github.com/$REPO/releases/download/$VERSION/$BINARY_NAME"

curl -L "$URL" -o "/tmp/$BINARY_NAME"
chmod +x "/tmp/$BINARY_NAME"
sudo mv "/tmp/$BINARY_NAME" "$INSTALL_DIR/"

echo "✅ PugVault installed successfully! Try running: PUG_MASTER_PASSWORD=test pug-vault list"
