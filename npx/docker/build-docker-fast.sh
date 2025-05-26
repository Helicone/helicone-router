#!/bin/bash

# Fast Docker Build Script for Helicone Router
# Uses multi-stage builds and parallel processing

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

echo "🚀 Fast Docker Build for Helicone Router"
echo "========================================"

# Clean up old images to free space
log_step "Cleaning up old Docker images"
docker system prune -f >/dev/null 2>&1 || true

# Function to build with progress
build_with_progress() {
    local dockerfile=$1
    local tag=$2
    local binary_name=$3
    
    log_step "Building $binary_name"
    
    # Build with progress and timeout
    timeout 600 docker build \
        --progress=plain \
        --no-cache \
        -f "$dockerfile" \
        -t "$tag" \
        . || {
        log_error "Build failed or timed out for $binary_name"
        return 1
    }
    
    # Extract binary quickly
    docker run --rm -v "$(pwd)/npx/dist:/dist" "$tag" && {
        log_success "$binary_name built successfully"
    } || {
        log_error "Failed to extract $binary_name"
        return 1
    }
}

# Build Linux binaries in parallel using background processes
log_step "Starting parallel builds"

# Start Linux glibc build in background
(
    build_with_progress "npx/docker/Dockerfile.linux" "helicone-router:linux" "Linux glibc binary"
) &
LINUX_PID=$!

# Start Linux musl build in background  
(
    build_with_progress "npx/docker/Dockerfile.linux-musl" "helicone-router:linux-musl" "Linux musl binary"
) &
MUSL_PID=$!

# Build macOS binary natively (if on macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    log_step "Building macOS binary (native)"
    if cargo build --release; then
        cp target/release/llm-proxy npx/dist/helicone-router-macos
        chmod +x npx/dist/helicone-router-macos
        log_success "macOS binary built"
    else
        log_error "macOS build failed"
    fi
else
    log_warning "Skipping macOS build (not on macOS system)"
fi

# Wait for parallel builds to complete
log_step "Waiting for parallel builds to complete..."

wait $LINUX_PID
LINUX_STATUS=$?

wait $MUSL_PID
MUSL_STATUS=$?

# Check results
if [ $LINUX_STATUS -eq 0 ] && [ $MUSL_STATUS -eq 0 ]; then
    log_success "All parallel builds completed successfully"
else
    log_error "Some builds failed - Linux: $LINUX_STATUS, Musl: $MUSL_STATUS"
    exit 1
fi

echo ""
echo "🎉 Fast Docker build complete!"
echo "============================="
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
echo "2. Test compatibility: npm run test:distributions" 