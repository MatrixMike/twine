#!/bin/bash

# Twine Scheme Interpreter - Dependency Update Script
#
# This script updates the local dependency source management when dependencies change.
# It synchronizes vendored sources with Cargo.toml and regenerates documentation.

set -e  # Exit on any error

echo "🔄 Updating Twine dependency management..."

# Ensure directory structure exists
echo "📁 Ensuring directory structure..."
mkdir -p deps/vendor
mkdir -p deps/docs
mkdir -p deps/registry

# Update vendored sources when dependencies change
echo "📦 Updating vendored sources..."
cargo vendor deps/vendor --sync Cargo.toml

# Clean old documentation
echo "🧹 Cleaning old documentation..."
rm -rf deps/docs/*

# Regenerate documentation after dependency updates
echo "📚 Regenerating documentation..."
cargo clean --doc
cargo doc --all-features --document-private-items --no-deps
cargo doc --all-features --document-private-items

# Copy updated docs to deps directory
echo "📋 Copying updated documentation..."
cp -r target/doc/* deps/docs/

# Verify update
echo "✅ Verifying update..."
if [ -n "$(ls -A deps/vendor)" ]; then
    echo "✨ Dependencies updated successfully"
else
    echo "❌ Failed to update dependencies"
    exit 1
fi

if [ -n "$(ls -A deps/docs)" ]; then
    echo "✨ Documentation updated successfully"
else
    echo "❌ Failed to update documentation"
    exit 1
fi

echo ""
echo "🎉 Dependency management updated successfully!"
echo ""
echo "📋 Updated:"
echo "  - Vendored sources: deps/vendor/"
echo "  - Generated docs:   deps/docs/"
echo ""
echo "💡 Dependencies are now synchronized with Cargo.toml"
