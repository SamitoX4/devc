#!/bin/bash

set -e

if [ -z "$BASH_VERSION" ]; then
    echo "Error: Please run with bash, not sh"
    echo "Usage: bash build-release.sh"
    exit 1
fi

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

cd "$CLI_DIR"

echo "=== Step 1: Cleaning previous builds ==="
cargo clean
echo ""

echo "=== Step 2: Building release ==="
cargo build --release

if [ ! -f "target/release/devc" ]; then
    echo "Error: Build failed"
    exit 1
fi
echo "✓ Build successful"
echo ""

echo "=== Step 3: Detect platform ==="
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

echo "=== Step 4: Version ==="
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

echo ""
echo "=== Step 5: Creating release package ==="

mkdir -p "$RELEASES_DIR"

RELEASE_FILE="devc-${VERSION}-${TRIPLE}.tar.gz"
cd target/release
tar -czvf "$RELEASES_DIR/$RELEASE_FILE" devc
cd "$SCRIPT_DIR"

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
