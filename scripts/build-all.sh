#!/bin/bash

# Cross-platform build script for file-search

set -e

echo "Building file-search for multiple platforms..."

# Define targets
TARGETS=(
    "x86_64-pc-windows-msvc"
    "x86_64-unknown-linux-gnu" 
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
)

# Create output directory
mkdir -p dist

for target in "${TARGETS[@]}"; do
    echo "Building for $target..."
    
    # Add target if not already installed
    rustup target add "$target" 2>/dev/null || true
    
    # Build with CLI features
    cargo build --release --target "$target" --features=cli
    
    # Copy binary to dist directory
    if [[ "$target" == *"windows"* ]]; then
        cp "target/$target/release/file-search.exe" "dist/file-search-$target.exe"
    else
        cp "target/$target/release/file-search" "dist/file-search-$target"
    fi
    
    echo "✓ Built for $target"
done

echo "All builds completed! Binaries are in the dist/ directory."
echo ""
echo "Available binaries:"
ls -la dist/