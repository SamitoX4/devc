#!/bin/bash

set -e

if [ -z "$BASH_VERSION" ]; then
    echo "Error: Please run with bash, not sh"
    echo "Usage: bash build-release.sh"
    exit 1
fi

export PATH="$HOME/.cargo/bin:$PATH"

check_and_install_rust() {
    if command -v cargo &> /dev/null; then
        echo "✓ Rust detected: $(cargo --version)"
        return 0
    fi
    
    echo "⚠ Rust is not installed"
    echo ""
    echo "Rust is required to compile the CLI."
    echo -n "Do you want to install Rust now? (Y/n): "
    read -r response
    response=${response:-Y}
    
    if [[ "$response" =~ ^[Nn]$ ]]; then
        echo "Error: Rust is required to continue"
        exit 1
    fi
    
    echo ""
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
    
    export PATH="$HOME/.cargo/bin:$PATH"
    
    if command -v cargo &> /dev/null; then
        echo ""
        echo "✓ Rust installed successfully: $(cargo --version)"
    else
        echo ""
        echo "Error: Failed to install Rust"
        echo "Please install Rust manually: https://rustup.rs"
        exit 1
    fi
}

REPO="SamitoX4/devc"
CLI_DIR="$(cd "$(dirname "$0")" && cd .. && pwd)/cli"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RELEASES_DIR="$(cd "$(dirname "$0")" && cd .. && pwd)/releases"

if [ ! -d "$CLI_DIR" ]; then
    echo "Error: CLI directory not found at $CLI_DIR"
    exit 1
fi

echo "=========================================="
echo "       devc Release Builder"
echo "=========================================="
echo ""

echo "=== Checking Rust ==="
check_and_install_rust
echo ""

cd "$CLI_DIR"

echo "=== Step 1: Detect platform ==="
ARCH=$(uname -m)
OS=$(uname -s)

case "$OS" in
    Linux*)
        case "$ARCH" in
            x86_64) TRIPLE="x86_64-unknown-linux-gnu" ;;
            aarch64) TRIPLE="aarch64-unknown-linux-gnu" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    Darwin*)
        case "$ARCH" in
            x86_64) TRIPLE="x86_64-apple-darwin" ;;
            arm64) TRIPLE="aarch64-apple-darwin" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "Platform: $OS ($TRIPLE)"
echo ""

echo "=== Step 2: Select version ==="
echo "Current releases:"

get_latest_version() {
    ls -1 "$RELEASES_DIR" 2>/dev/null | grep "\.tar\.gz$" | grep "${TRIPLE}" | sed 's/devc-//' | sed 's/-x86_64.*//' | sed 's/-aarch64.*//' | sort -V | tail -1
}

SUGGESTED=""
MAJOR_JUMP_SUGGESTED=""

LATEST=$(get_latest_version)

if [ -n "$LATEST" ]; then
    echo "  Latest: $LATEST"
    
    MAJOR=$(echo "$LATEST" | cut -d. -f1 | tr -d 'v')
    MINOR=$(echo "$LATEST" | cut -d. -f2)
    PATCH=$(echo "$LATEST" | cut -d. -f3)
    
    PATCH_NEXT=$((PATCH + 1))
    MINOR_NEXT=$((MINOR + 1))
    MAJOR_NEXT=$((MAJOR + 1))
    
    SUGGESTED="v${MAJOR}.${MINOR}.${PATCH_NEXT}"
    
    echo ""
    echo "Version type:"
    echo "  p - patch:     $SUGGESTED (bug fixes)"
    echo "  f - feature:   v${MAJOR}.${MINOR_NEXT}.0 (new features)"
    echo "  m - major:     v${MAJOR_NEXT}.0.0 (breaking changes)"
    
    if [ "$MINOR" -ge 1 ] && [ "$PATCH" -ge 3 ]; then
        MAJOR_JUMP_SUGGESTED="v${MAJOR_NEXT}.0.0"
        echo ""
        echo "  ★ Ready for major release: Consider v${MAJOR_NEXT}.0.0"
    fi
else
    SUGGESTED="v0.1.0"
    echo "  No releases found"
    echo ""
    echo "Version type:"
    echo "  Enter - patch: $SUGGESTED (first release)"
    echo "  m - major:     v1.0.0"
fi

echo ""
echo -n "Enter version or type (p/f/m, default: $SUGGESTED): "
read VERSION

if [ -z "$VERSION" ]; then
    VERSION="$SUGGESTED"
elif [ "$VERSION" = "p" ] || [ "$VERSION" = "P" ]; then
    VERSION="$SUGGESTED"
elif [ "$VERSION" = "f" ] || [ "$VERSION" = "F" ]; then
    if [ -n "$LATEST" ]; then
        MAJOR=$(echo "$LATEST" | cut -d. -f1 | tr -d 'v')
        MINOR=$(echo "$LATEST" | cut -d. -f2)
        VERSION="v${MAJOR}.$((MINOR + 1)).0"
    else
        VERSION="v0.1.0"
    fi
elif [ "$VERSION" = "m" ] || [ "$VERSION" = "M" ]; then
    if [ -n "$LATEST" ]; then
        MAJOR=$(echo "$LATEST" | cut -d. -f1 | tr -d 'v')
        VERSION="v$((MAJOR + 1)).0.0"
    else
        VERSION="v1.0.0"
    fi
fi

if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Invalid version format. Use vX.Y.Z (e.g., v0.1.0)"
    exit 1
fi

if [ -f "$RELEASES_DIR/devc-${VERSION}-${TRIPLE}.tar.gz" ]; then
    echo "Error: Release $VERSION for $TRIPLE already exists"
    exit 1
fi

VERSION_NOV=${VERSION#v}

echo ""
echo "=== Step 3: Update versions ==="

echo "  - Updating Cargo.toml..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i.bak "s/^version = \".*\"/version = \"$VERSION_NOV\"/" Cargo.toml
    rm -f Cargo.toml.bak
else
    sed -i "s/^version = \".*\"/version = \"$VERSION_NOV\"/" Cargo.toml
fi

echo "  - Updating docs/index.html..."
DOCS_INDEX="$CLI_DIR/../docs/index.html"
if [ -f "$DOCS_INDEX" ]; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i.bak "s/<div class=\"badge\">.*<\/div>/<div class=\"badge\">$VERSION<\/div>/" "$DOCS_INDEX"
        rm -f "$DOCS_INDEX.bak"
    else
        sed -i "s/<div class=\"badge\">.*<\/div>/<div class=\"badge\">$VERSION<\/div>/" "$DOCS_INDEX"
    fi
fi

echo "  - Updating README.md..."
README_FILE="$CLI_DIR/../README.md"
if [ -f "$README_FILE" ]; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i.bak "s|raw.githubusercontent.com/SamitoX4/devc/main/pages|raw.githubusercontent.com/SamitoX4/devc/main/docs|g" "$README_FILE"
        sed -i.bak "s|main/pages/install.sh|main/docs/install.sh|g" "$README_FILE"
        rm -f "$README_FILE.bak"
    else
        sed -i "s|raw.githubusercontent.com/SamitoX4/devc/main/pages|raw.githubusercontent.com/SamitoX4/devc/main/docs|g" "$README_FILE"
        sed -i "s|main/pages/install.sh|main/docs/install.sh|g" "$README_FILE"
    fi
fi

echo "✓ All versions updated to $VERSION"
echo ""

echo "=== Step 4: Cleaning previous builds ==="
cargo clean
echo ""

echo "=== Step 5: Building release ==="
cargo build --release

if [ ! -f "target/release/devc" ]; then
    echo "Error: Build failed"
    exit 1
fi
echo "✓ Build successful"
echo ""

echo "=== Step 6: Creating release package ==="

RELEASES_DIR="$(cd "$(dirname "$0")" && cd .. && pwd)/releases"
STAGING_DIR="$RELEASES_DIR/staging-$$"

mkdir -p "$RELEASES_DIR"
mkdir -p "$STAGING_DIR"

echo "Copying binary..."
cp target/release/devc "$STAGING_DIR/"

echo "Downloading templates from GitHub..."
TEMPLATES_URL="https://github.com/SamitoX4/devcontainers/archive/refs/heads/master.zip"
TEMP_ZIP="/tmp/devc-templates-$$.zip"

if curl -sL "$TEMPLATES_URL" -o "$TEMP_ZIP"; then
    TEMPLATES_ZIP_CONTENT=$(unzip -l "$TEMP_ZIP" 2>/dev/null | grep -c "devcontainers-master/templates" || echo "0")
    if [ "$TEMPLATES_ZIP_CONTENT" -gt 0 ]; then
        unzip -q "$TEMP_ZIP" -d /tmp/
        mv /tmp/devcontainers-master/templates "$STAGING_DIR/"
        rm -rf /tmp/devcontainers-master "$TEMP_ZIP"
        TEMPLATE_COUNT=$(ls "$STAGING_DIR/templates" 2>/dev/null | wc -l)
        echo "✓ Templates included: $TEMPLATE_COUNT templates"
    else
        echo "⚠ Warning: No templates found in archive"
    fi
else
    echo "⚠ Warning: Failed to download templates"
fi

RELEASE_FILE="devc-${VERSION}-${TRIPLE}.tar.gz"
cd "$STAGING_DIR"
tar -czvf "$RELEASES_DIR/$RELEASE_FILE" ./*
cd "$SCRIPT_DIR"

rm -rf "$STAGING_DIR"

echo "✓ Package created: releases/$RELEASE_FILE"
echo ""

echo "=========================================="
echo "         Release Summary"
echo "=========================================="
echo "Version:    $VERSION"
echo "Platform:  $TRIPLE"
echo "File:      releases/$RELEASE_FILE"
echo ""
echo "=========================================="
echo ""
echo "Next steps to publish:"
echo ""
echo "1. Go to: https://github.com/$REPO/releases/new"
echo ""
echo "2. Create release with:"
echo "   Tag:         $VERSION"
echo "   Title:       devc $VERSION"
echo "   Description: Release $VERSION"
echo ""
echo "3. Upload file:"
echo "   releases/$RELEASE_FILE"
echo ""
echo "4. Publish release"
echo ""
echo "After publishing, users can install with:"
echo "   curl -fsSL https://samitox4.github.io/devc/install.sh | bash"
echo ""
