#!/usr/bin/env bash

# Build script for ccusage Rust implementation
# Builds optimized binaries for multiple platforms

set -e

echo "🦀 Building ccusage Rust implementation..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo is not installed.${NC}"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Parse arguments
BUILD_ALL=false
BUILD_RELEASE=true
TARGET_PLATFORM=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            BUILD_ALL=true
            shift
            ;;
        --debug)
            BUILD_RELEASE=false
            shift
            ;;
        --target)
            TARGET_PLATFORM="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --all           Build for all supported platforms"
            echo "  --debug         Build debug version instead of release"
            echo "  --target <target> Build for specific target (e.g., x86_64-apple-darwin)"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Build function
build_target() {
    local target=$1
    local output_name=$2

    echo -e "${YELLOW}Building for $target...${NC}"

    if $BUILD_RELEASE; then
        if cargo build --release --target "$target" 2>/dev/null; then
            echo -e "${GREEN}✓ Successfully built for $target${NC}"

            # Copy to output directory
            mkdir -p dist
            if [[ "$target" == *"windows"* ]]; then
                cp "target/$target/release/ccusage.exe" "dist/$output_name.exe" 2>/dev/null || true
            else
                cp "target/$target/release/ccusage" "dist/$output_name" 2>/dev/null || true
            fi
        else
            echo -e "${YELLOW}⚠ Skipping $target (not configured)${NC}"
        fi
    else
        cargo build --target "$target"
    fi
}

# Install targets if building for all platforms
if $BUILD_ALL; then
    echo "Installing cross-compilation targets..."
    rustup target add x86_64-apple-darwin 2>/dev/null || true
    rustup target add aarch64-apple-darwin 2>/dev/null || true
    rustup target add x86_64-unknown-linux-gnu 2>/dev/null || true
    rustup target add x86_64-pc-windows-gnu 2>/dev/null || true
    rustup target add aarch64-unknown-linux-gnu 2>/dev/null || true
fi

# Build based on options
if $BUILD_ALL; then
    echo -e "${GREEN}Building for all platforms...${NC}"
    build_target "x86_64-apple-darwin" "ccusage-darwin-x64"
    build_target "aarch64-apple-darwin" "ccusage-darwin-arm64"
    build_target "x86_64-unknown-linux-gnu" "ccusage-linux-x64"
    build_target "x86_64-pc-windows-gnu" "ccusage-windows-x64"
    build_target "aarch64-unknown-linux-gnu" "ccusage-linux-arm64"
elif [ -n "$TARGET_PLATFORM" ]; then
    echo -e "${GREEN}Building for $TARGET_PLATFORM...${NC}"
    build_target "$TARGET_PLATFORM" "ccusage-$TARGET_PLATFORM"
else
    echo -e "${GREEN}Building for current platform...${NC}"
    if $BUILD_RELEASE; then
        cargo build --release
        echo -e "${GREEN}✓ Build complete!${NC}"
        echo ""
        echo "Binary location: target/release/ccusage"
    else
        cargo build
        echo -e "${GREEN}✓ Debug build complete!${NC}"
        echo ""
        echo "Binary location: target/debug/ccusage"
    fi
fi

# Show output directory contents if dist was created
if [ -d "dist" ]; then
    echo ""
    echo -e "${GREEN}Built binaries:${NC}"
    ls -lh dist/
fi

echo ""
echo -e "${GREEN}🎉 Build complete!${NC}"
echo ""
echo "To run the program:"
if $BUILD_RELEASE; then
    echo "  ./target/release/ccusage --help"
else
    echo "  ./target/debug/ccusage --help"
fi
echo ""
echo "To install system-wide:"
echo "  cargo install --path ."