#!/bin/bash

# Simple Docker Build Script for Helicone Router
# Builds only the working binaries (Linux glibc + macOS)

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
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

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    echo "❌ Please run this script from the project root directory (where Cargo.toml is)"
    exit 1
fi

# Create dist directory
mkdir -p npx/dist

echo "🐳 Simple Docker Build for Helicone Router"
echo "=========================================="

# Build Linux glibc binary with Docker
log_step "Building Linux glibc binary with Docker"
if docker build -f npx/docker/Dockerfile.linux -t helicone-router:linux .; then
    if docker run --rm -v "$(pwd)/npx/dist:/dist" helicone-router:linux; then
        log_success "Linux glibc binary built successfully"
    else
        log_error "Failed to extract Linux binary"
        exit 1
    fi
else
    log_error "Linux Docker build failed"
    exit 1
fi

# Build macOS binary natively (if on macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    log_step "Building macOS binary (native)"
    if cargo build --release; then
        cp target/release/llm-proxy npx/dist/helicone-router-macos
        chmod +x npx/dist/helicone-router-macos
        log_success "macOS binary built successfully"
    else
        log_error "macOS build failed"
        exit 1
    fi
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
echo "Binary sizes:"
du -h npx/dist/*

echo ""
echo "🚀 Ready for testing and deployment!"
echo "Next steps:"
echo "1. Test the binaries: cd npx && npm test"
echo "2. Test in containers: docker run --rm -v \$(pwd)/npx/dist:/test ubuntu:20.04 /test/helicone-router-linux --help" 