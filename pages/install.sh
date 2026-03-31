#!/bin/bash

set -e

INSTALL_DIR="${HOME}/.local/bin"
REPO="SamitoX4/devc"
BINARY_NAME="devc"

echo "Installing devc CLI..."

if [ ! -d "$INSTALL_DIR" ]; then
    mkdir -p "$INSTALL_DIR"
fi

ARCH=$(uname -m)
OS=$(uname -s)

case "$OS" in
    Linux*)
        case "$ARCH" in
            x86_64) TRIPLE="x86_64-unknown-linux-gnu" ;;
            aarch64|arm64) TRIPLE="aarch64-unknown-linux-gnu" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    Darwin*)
        case "$ARCH" in
            x86_64) TRIPLE="x86_64-apple-darwin" ;;
            aarch64|arm64) TRIPLE="aarch64-apple-darwin" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "Detected: $OS ($TRIPLE)"

echo "Getting latest version..."
VERSION=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
    echo "Error: Could not determine latest version"
    exit 1
fi

echo "Downloading devc $VERSION..."

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY_NAME}-${TRIPLE}.tar.gz"

TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "Downloading from: $DOWNLOAD_URL"
curl -sL "$DOWNLOAD_URL" -o "${BINARY_NAME}.tar.gz"

if [ ! -f "${BINARY_NAME}.tar.gz" ] || [ ! -s "${BINARY_NAME}.tar.gz" ]; then
    echo "Error: Download failed"
    rm -rf "$TEMP_DIR"
    exit 1
fi

tar xzf "${BINARY_NAME}.tar.gz"

chmod +x "$BINARY_NAME"
mv "$BINARY_NAME" "$INSTALL_DIR/"

cd /
rm -rf "$TEMP_DIR"

if command -v devc &> /dev/null; then
    echo ""
    echo "devc is already in your PATH"
else
    echo ""
    echo "Added to PATH: $INSTALL_DIR"
    echo "Make sure $INSTALL_DIR is in your PATH"
fi

echo ""
echo "✓ devc v${VERSION} installed successfully!"
echo ""

echo "Running initial setup..."
"$INSTALL_DIR/devc" update --force 2>/dev/null || true

echo ""
echo "Next steps:"
echo "  devc list                    - List available templates"
echo "  devc gen                     - Generate a devcontainer (interactive)"
echo "  devc gen -t nodejs -n myapp - Generate with options"
echo ""
