#!/bin/bash

# Release script for oxibase
# Usage: ./scripts/release.sh [VERSION]

VERSION="$1"

if [ -z "$VERSION" ]; then
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    MAJOR=$(echo $CURRENT_VERSION | cut -d. -f1)
    MINOR=$(echo $CURRENT_VERSION | cut -d. -f2)
    PATCH=$(echo $CURRENT_VERSION | cut -d. -f3)
    NEW_PATCH=$((PATCH + 1))
    VERSION="$MAJOR.$MINOR.$NEW_PATCH"
    echo "No VERSION provided, bumping patch to $VERSION"
fi

echo "Updating version to $VERSION"
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
cargo check
git commit -a -m "Bump version to $VERSION"
git tag -a v$VERSION -m "Release version $VERSION"
git push origin HEAD
git push origin v$VERSION
cargo publish
echo "DEBUG: VERSION=$VERSION"
gh release create v$VERSION --generate-notes
echo "Release completed."