#!/bin/bash

# Cross-compilation build script for Small MP4
set -e

PROJECT_NAME="small-mp4"
VERSION=$(grep version Cargo.toml | head -n1 | cut -d'"' -f2)
BUILD_DIR="target/release-builds"

echo "🔨 Building Small MP4 v$VERSION for multiple platforms..."

# Create build directory
mkdir -p "$BUILD_DIR"

# Platform configurations
declare -A PLATFORMS=(
    ["x86_64-unknown-linux-gnu"]="Linux x64"
    ["x86_64-unknown-linux-musl"]="Linux x64 (static)"
    ["aarch64-unknown-linux-gnu"]="Linux ARM64"
    ["x86_64-pc-windows-gnu"]="Windows x64"
    ["x86_64-pc-windows-msvc"]="Windows x64 (MSVC)"
    ["aarch64-pc-windows-msvc"]="Windows ARM64"
    ["x86_64-apple-darwin"]="macOS Intel"
    ["aarch64-apple-darwin"]="macOS Apple Silicon"
)

# Supported platforms (comment out problematic ones)
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-gnu" 
    "x86_64-pc-windows-gnu"
    # "x86_64-pc-windows-msvc"  # Requires Visual Studio
    # "aarch64-pc-windows-msvc" # Requires Visual Studio
    # "x86_64-apple-darwin"     # Requires macOS SDK
    # "aarch64-apple-darwin"    # Requires macOS SDK
)

# Build function
build_target() {
    local target=$1
    local name="${PLATFORMS[$target]}"
    
    echo "📦 Building for $target ($name)..."
    
    # Special handling for different targets
    case $target in
        "x86_64-pc-windows-"*)
            # Windows builds
            cargo build --release --target="$target" --features=gui
            local binary_name="${PROJECT_NAME}.exe"
            ;;
        *)
            # Unix builds
            cargo build --release --target="$target" --features=gui
            local binary_name="$PROJECT_NAME"
            ;;
    esac
    
    # Copy and rename binary
    local source_path="target/$target/release/$binary_name"
    local dest_name="${PROJECT_NAME}-${VERSION}-${target}"
    
    if [[ "$target" == *"windows"* ]]; then
        dest_name="${dest_name}.exe"
    fi
    
    if [ -f "$source_path" ]; then
        cp "$source_path" "$BUILD_DIR/$dest_name"
        echo "✅ Built: $BUILD_DIR/$dest_name"
        
        # Calculate file size
        local size=$(du -h "$BUILD_DIR/$dest_name" | cut -f1)
        echo "   Size: $size"
    else
        echo "❌ Build failed for $target"
        return 1
    fi
}

# Main build loop
failed_builds=()
successful_builds=()

for target in "${TARGETS[@]}"; do
    echo ""
    if build_target "$target"; then
        successful_builds+=("$target")
    else
        failed_builds+=("$target")
    fi
done

# Summary
echo ""
echo "🏁 Build Summary"
echo "================"
echo "✅ Successful builds: ${#successful_builds[@]}"
for target in "${successful_builds[@]}"; do
    echo "   - $target (${PLATFORMS[$target]})"
done

if [ ${#failed_builds[@]} -gt 0 ]; then
    echo "❌ Failed builds: ${#failed_builds[@]}"
    for target in "${failed_builds[@]}"; do
        echo "   - $target (${PLATFORMS[$target]})"
    done
fi

echo ""
echo "📁 Build artifacts in: $BUILD_DIR"
ls -la "$BUILD_DIR"

echo ""
echo "🎉 Cross-compilation completed!"