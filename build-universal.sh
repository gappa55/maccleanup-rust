#!/bin/bash
# Build universal binary for both Intel and Apple Silicon Macs

set -e

echo "🔨 Building maccleanup-rust for both architectures..."

# Build for Apple Silicon (ARM64)
echo "Building for Apple Silicon (ARM64)..."
cargo build --release --target aarch64-apple-darwin

# Build for Intel (x86_64)
echo "Building for Intel (x86_64)..."
cargo build --release --target x86_64-apple-darwin

# Create universal binary directory
mkdir -p target/universal

# Create universal binary using lipo
echo "Creating universal binary..."
lipo -create \
    target/aarch64-apple-darwin/release/maccleanup-rust \
    target/x86_64-apple-darwin/release/maccleanup-rust \
    -output target/universal/maccleanup-rust

# Verify the universal binary
echo "Verifying universal binary..."
file target/universal/maccleanup-rust
lipo -info target/universal/maccleanup-rust

echo "✅ Universal binary created at: target/universal/maccleanup-rust"
echo "📦 This binary will run on both Intel and Apple Silicon Macs"