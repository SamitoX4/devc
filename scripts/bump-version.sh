#!/bin/bash

set -e

if [ -z "$1" ]; then
    echo "Usage: ./scripts/bump-version.sh <version>"
    echo "Example: ./scripts/bump-version.sh 0.3.0"
    exit 1
fi

VERSION=$1
VERSION_WITH_V="v${VERSION}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Bumping version to $VERSION_WITH_V..."

# Update Cargo.toml
echo "  → cli/Cargo.toml"
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" "$ROOT_DIR/cli/Cargo.toml"

# Update npm/package.json
echo "  → npm/package.json"
sed -i "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" "$ROOT_DIR/npm/package.json"

# Update docs/index.html badge
echo "  → docs/index.html"
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i.bak "s/<div class=\"badge\">.*<\/div>/<div class=\"badge\">$VERSION_WITH_V<\/div>/" "$ROOT_DIR/docs/index.html"
    rm -f "$ROOT_DIR/docs/index.html.bak"
else
    sed -i "s/<div class=\"badge\">.*<\/div>/<div class=\"badge\">$VERSION_WITH_V<\/div>/" "$ROOT_DIR/docs/index.html"
fi

echo ""
echo "✓ Version bumped to $VERSION_WITH_V"
echo ""
echo "Next steps:"
echo "  1. Review the changes (git diff)"
echo "  2. git add -A && git commit -m \"chore: bump version to $VERSION_WITH_V\""
echo "  3. git tag $VERSION_WITH_V"
echo "  4. git push origin main $VERSION_WITH_V"
echo ""
echo "GitHub Actions will build the release automatically."
echo "Once the release is published, run: cd npm && npm publish"
