#!/bin/bash
# 🐶 PugVault Installer

REPO="tannht/pug-vault"
BINARY_NAME="pug-vault"
INSTALL_DIR="/usr/local/bin"

echo "🐶 Downloading PugVault..."
# Trong thực tế, mình sẽ tải binary từ GitHub Release. 
# Nhưng vì chưa có release, mình sẽ build từ source nếu máy có Rust, 
# hoặc giả lập việc cài đặt binary.
if command -v cargo &> /dev/null; then
    cargo build --release
    sudo cp target/release/$BINARY_NAME $INSTALL_DIR/
    echo "✅ PugVault installed successfully via Cargo!"
else
    echo "❌ Rust/Cargo not found. Please install Rust first or download binary from releases."
    exit 1
fi
