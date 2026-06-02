#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=========================================="
echo "           devc Release Script"
echo "=========================================="
echo ""

read -rp "Enter version (e.g. 0.2.3): " VERSION

if [ -z "$VERSION" ]; then
    echo "Error: version is required"
    exit 1
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: invalid version format. Expected X.Y.Z (e.g. 0.2.3)"
    exit 1
fi

VERSION_WITH_V="v${VERSION}"

echo ""
echo "This will:"
echo "  1. Bump version to $VERSION_WITH_V"
echo "  2. Commit changes"
echo "  3. Create tag $VERSION_WITH_V"
echo "  4. Push to origin main + tag"
echo ""

read -rp "Continue? (y/N): " CONFIRM
CONFIRM=${CONFIRM:-N}

if [[ ! "$CONFIRM" =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "→ Bumping version..."
"$SCRIPT_DIR/bump-version.sh" "$VERSION"

echo ""
echo "→ Committing..."
git add -A
git commit -m "chore: bump version to $VERSION_WITH_V"

echo ""
echo "→ Tagging..."
git tag "$VERSION_WITH_V"

echo ""
echo "→ Pushing..."
git push origin main "$VERSION_WITH_V"

echo ""
echo "=========================================="
echo "✓ Release v$VERSION initiated!"
echo "=========================================="
echo ""
echo "GitHub Actions is now building the release."
echo "Monitor progress at:"
echo "  https://github.com/SamitoX4/devc/actions"
echo ""
echo "Once the release is published, publish to npm with:"
echo "  cd npm && npm publish"
echo ""
