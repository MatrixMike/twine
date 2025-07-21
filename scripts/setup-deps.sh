#!/bin/bash

# Twine Scheme Interpreter - Dependency Setup Script
#
# This script sets up the local dependency source management infrastructure
# for AI agent access to complete, accurate source code and documentation.

set -e  # Exit on any error

echo "🔧 Setting up Twine dependency management..."

# Create directory structure if it doesn't exist
echo "📁 Creating directory structure..."
mkdir -p deps/vendor
mkdir -p deps/docs
mkdir -p deps/registry

# Download all dependency sources
echo "📦 Vendoring dependency sources..."
cargo vendor deps/vendor

# Generate comprehensive documentation
echo "📚 Generating comprehensive documentation..."
cargo doc --all-features --document-private-items --no-deps
cargo doc --all-features --document-private-items

# Copy generated docs to deps directory
echo "📋 Copying documentation to deps/docs/..."
cp -r target/doc/* deps/docs/

# Verify setup
echo "✅ Verifying setup..."
if [ -d "deps/vendor" ] && [ -d "deps/docs" ] && [ -d "deps/registry" ]; then
    echo "✨ Directory structure created successfully"
else
    echo "❌ Failed to create directory structure"
    exit 1
fi

if [ -n "$(ls -A deps/vendor)" ]; then
    echo "✨ Dependencies vendored successfully"
else
    echo "❌ Failed to vendor dependencies"
    exit 1
fi

if [ -n "$(ls -A deps/docs)" ]; then
    echo "✨ Documentation generated successfully"
else
    echo "❌ Failed to generate documentation"
    exit 1
fi

echo ""
echo "🎉 Dependency management setup complete!"
echo ""
echo "📋 Summary:"
echo "  - Vendored sources: deps/vendor/"
echo "  - Generated docs:   deps/docs/"
echo "  - Registry cache:   deps/registry/"
echo ""
echo "💡 To update dependencies, run: ./scripts/update-deps.sh"
