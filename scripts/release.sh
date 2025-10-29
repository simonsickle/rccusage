#!/bin/bash

# Release script for rccusage
# Usage: ./scripts/release.sh [patch|minor|major]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if version bump type is provided
if [ -z "$1" ]; then
    echo -e "${RED}Error: Version bump type not specified${NC}"
    echo "Usage: $0 [patch|minor|major]"
    exit 1
fi

VERSION_BUMP=$1

# Ensure we're on main/master branch
BRANCH=$(git branch --show-current)
if [[ "$BRANCH" != "main" && "$BRANCH" != "master" ]]; then
    echo -e "${RED}Error: Must be on main or master branch to release${NC}"
    echo "Current branch: $BRANCH"
    exit 1
fi

# Ensure working directory is clean
if [[ -n $(git status -s) ]]; then
    echo -e "${RED}Error: Working directory is not clean${NC}"
    git status -s
    exit 1
fi

echo -e "${GREEN}Starting release process...${NC}"

# Run tests
echo -e "${YELLOW}Running tests...${NC}"
cargo test

# Run clippy
echo -e "${YELLOW}Running clippy...${NC}"
cargo clippy -- -D warnings

# Check formatting
echo -e "${YELLOW}Checking formatting...${NC}"
cargo fmt -- --check

# Build release binary to ensure it compiles
echo -e "${YELLOW}Building release binary...${NC}"
cargo build --release

# Get current version
CURRENT_VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
echo -e "${GREEN}Current version: $CURRENT_VERSION${NC}"

# Calculate new version
IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR=${VERSION_PARTS[0]}
MINOR=${VERSION_PARTS[1]}
PATCH=${VERSION_PARTS[2]}

case $VERSION_BUMP in
    patch)
        PATCH=$((PATCH + 1))
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    *)
        echo -e "${RED}Error: Invalid version bump type: $VERSION_BUMP${NC}"
        echo "Must be one of: patch, minor, major"
        exit 1
        ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"
echo -e "${GREEN}New version: $NEW_VERSION${NC}"

# Update version in Cargo.toml
echo -e "${YELLOW}Updating Cargo.toml...${NC}"
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
else
    # Linux
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
fi

# Update Cargo.lock
echo -e "${YELLOW}Updating Cargo.lock...${NC}"
cargo build --quiet

# Commit version bump
echo -e "${YELLOW}Committing version bump...${NC}"
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to $NEW_VERSION"

# Create tag
echo -e "${YELLOW}Creating tag v$NEW_VERSION...${NC}"
git tag -a "v$NEW_VERSION" -m "Release version $NEW_VERSION"

echo -e "${GREEN}✓ Release preparation complete!${NC}"
echo ""
echo "To complete the release:"
echo "  1. Review the changes: git show"
echo "  2. Push to remote: git push origin main --tags"
echo "  3. GitHub Actions will automatically:"
echo "     - Build binaries for all platforms"
echo "     - Create a GitHub release"
echo "     - Upload artifacts"
echo ""
echo "To publish to crates.io (after GitHub release):"
echo "  cargo publish"