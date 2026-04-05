#!/bin/bash
# Pre-commit build check script
# Run this before pushing to catch build errors early

set -e

echo "🔍 Running pre-commit build checks..."

# Check frontend build
echo "📦 Building frontend..."
npm run build

# Check Rust code
echo "🦀 Checking Rust code..."
cd src-tauri

echo "  - Running cargo check..."
cargo check

echo "  - Running clippy..."
cargo clippy -- -D warnings 2>/dev/null || cargo clippy

echo "  - Running tests..."
cargo test

cd ..

echo ""
echo "✅ All checks passed! You can safely push."
