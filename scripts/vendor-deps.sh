#!/bin/bash

# Twine Scheme Interpreter - Dependency Management Script
#
# This script manages local dependency sources and documentation for AI agent access.
# It can be used for both initial setup and updates when dependencies change.
# It automatically removes old vendored dependencies that are no longer needed.

set -e  # Exit on any error

# Auto-detect setup vs update mode
MODE="update"
if [[ ! -d "deps/vendor" ]] || [[ -z "$(ls -A deps/vendor 2>/dev/null)" ]] || [[ ! -L "deps/doc" ]] || [[ ! -d "target/doc" ]]; then
    MODE="setup"
fi

if [[ "$MODE" == "setup" ]]; then
    echo "🔧 Setting up Twine dependency management..."
else
    echo "🔧 Updating Twine dependency management..."
fi

# Create directory structure
echo "📁 Ensuring directory structure..."
mkdir -p deps/vendor

# Clean up old vendored dependencies in update mode
if [[ "$MODE" == "update" ]] && [[ -f "Cargo.lock" ]]; then
    echo "🧹 Cleaning up old vendored dependencies..."

    # Get list of current dependencies from Cargo.lock
    if command -v awk >/dev/null 2>&1; then
        # Extract package names from Cargo.lock using awk
        CURRENT_DEPS=$(awk '/^\[\[package\]\]/{getline; if(/^name = /) {gsub(/^name = "|"$/, ""); print}}' Cargo.lock | sort)
    else
        # Fallback using grep and sed if awk is not available
        CURRENT_DEPS=$(grep -A1 '^\[\[package\]\]' Cargo.lock | grep '^name = ' | sed 's/^name = "\(.*\)"$/\1/' | sort)
    fi

    # Get list of currently vendored directories
    if [[ -d "deps/vendor" ]] && [[ -n "$(ls -A deps/vendor 2>/dev/null)" ]]; then
        VENDORED_DIRS=$(ls deps/vendor | sort)

        # Find directories to remove (vendored but not in current deps)
        DIRS_TO_REMOVE=""
        for dir in $VENDORED_DIRS; do
            if ! echo "$CURRENT_DEPS" | grep -q "^${dir}$"; then
                DIRS_TO_REMOVE="$DIRS_TO_REMOVE $dir"
            fi
        done

        # Remove old dependency directories
        if [[ -n "$DIRS_TO_REMOVE" ]]; then
            echo "🗑️  Removing old dependencies:$DIRS_TO_REMOVE"
            for dir in $DIRS_TO_REMOVE; do
                rm -rf "deps/vendor/$dir"
                echo "   - Removed: $dir"
            done
        else
            echo "✨ No old dependencies to remove"
        fi
    fi
fi

# Update/vendor dependency sources
if [[ "$MODE" == "setup" ]]; then
    echo "📦 Vendoring dependency sources..."
    cargo vendor deps/vendor
else
    echo "📦 Updating vendored sources..."
    cargo vendor deps/vendor --sync Cargo.toml
fi

# Clean old documentation if updating
if [[ "$MODE" == "update" ]]; then
    echo "🧹 Cleaning old documentation..."
    cargo clean --doc
fi

# Generate comprehensive documentation
echo "📚 Generating documentation..."
if [[ "$MODE" == "update" ]]; then
    cargo clean --doc
fi
cargo doc --all-features --document-private-items --no-deps
cargo doc --all-features --document-private-items

# Create symlink to documentation
echo "📋 Linking documentation to deps/doc/..."
if [[ ! -L "deps/doc" ]]; then
    rm -rf deps/doc
    ln -s ../target/doc deps/doc
fi

# Verify operation
echo "✅ Verifying $MODE..."
if [ -n "$(ls -A deps/vendor)" ]; then
    if [[ "$MODE" == "setup" ]]; then
        echo "✨ Dependencies set up successfully"
    else
        echo "✨ Dependencies updated successfully"
    fi
else
    echo "❌ Failed to $MODE dependencies"
    exit 1
fi

if [ -L "deps/doc" ] && [ -d "target/doc" ] && [ -n "$(ls -A target/doc)" ]; then
    echo "✨ Documentation linked successfully"
else
    echo "❌ Failed to link documentation"
    exit 1
fi

echo ""
echo "🎉 Dependency management $MODE complete!"
echo ""
echo "📋 Summary:"
echo "  - Vendored sources: deps/vendor/"
echo "  - Generated docs:   deps/doc/"
echo ""
if [[ "$MODE" == "setup" ]]; then
    echo "💡 To update dependencies later, run: ./scripts/vendor-deps.sh"
else
    echo "💡 Dependencies are now synchronized with Cargo.toml"
fi
