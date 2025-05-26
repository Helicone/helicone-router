#!/bin/bash

# Docker Build Script for Helicone Router
# Run this from the project root directory

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_step() {
    echo -e "${BLUE}▶ $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Check we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    echo "❌ Please run this script from the project root directory (where Cargo.toml is)"
    exit 1
fi

# Create dist directory
mkdir -p npx/dist

echo "🐳 Building Helicone Router with Docker"
echo "======================================"

# Build Linux x86_64 binary
log_step "Building Linux x86_64 binary"
docker build -f npx/docker/Dockerfile.linux -t helicone-router:linux .
docker run --rm -v "$(pwd)/npx/dist:/dist" helicone-router:linux
log_success "Linux x86_64 binary built"

# Build Linux musl binary
log_step "Building Linux musl binary (Alpine-compatible)"
docker build -f npx/docker/Dockerfile.linux-musl -t helicone-router:linux-musl .
docker run --rm -v "$(pwd)/npx/dist:/dist" helicone-router:linux-musl
log_success "Linux musl binary built"

# Build macOS binary (if on macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    log_step "Building macOS binary (native)"
    cargo build --release
    cp target/release/llm-proxy npx/dist/helicone-router-macos
    chmod +x npx/dist/helicone-router-macos
    log_success "macOS binary built"
else
    log_warning "Skipping macOS build (not on macOS system)"
fi

echo ""
echo "🎉 Docker build complete!"
echo "========================="
echo "Built binaries:"
ls -lh npx/dist/

echo ""
echo "File types:"
file npx/dist/* 2>/dev/null || echo "file command not available"

echo ""
echo "Next steps:"
echo "1. Test the binaries: cd npx && npm test"
echo "2. Test distribution compatibility with: docker run -v \$(pwd)/npx/dist:/dist alpine:latest /dist/helicone-router-linux-musl --help" 