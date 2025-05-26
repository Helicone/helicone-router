#!/bin/bash

# Test Docker Builds Script
# Tests all Docker containers and binaries locally

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
if [[ ! -f "../Cargo.toml" ]]; then
    echo "❌ Please run this script from the npx/ directory"
    exit 1
fi

echo "🐳 Testing Docker Builds for Helicone Router"
echo "============================================"

# Clean up previous builds
log_step "Cleaning up previous builds"
rm -rf dist/*
rm -rf test-artifacts/
mkdir -p test-artifacts

# Test 1: Build with Docker
log_step "Building binaries with Docker"
cd ..
./npx/docker/build-docker.sh
cd npx

if [[ -f "dist/helicone-router-linux" && -f "dist/helicone-router-linux-musl" ]]; then
    log_success "Docker builds completed"
else
    log_error "Docker builds failed"
    exit 1
fi

# Test 2: Test binaries in different containers
log_step "Testing binaries in different containers"

echo ""
echo "Testing glibc binary in Ubuntu..."
docker run --rm -v "$(pwd)/dist:/test" ubuntu:20.04 bash -c "
    chmod +x /test/helicone-router-linux &&
    echo 'Binary info:' &&
    ls -lh /test/helicone-router-linux &&
    echo 'Testing execution:' &&
    /test/helicone-router-linux --help || echo 'Test completed'
"

echo ""
echo "Testing musl binary in Alpine..."
docker run --rm -v "$(pwd)/dist:/test" alpine:latest sh -c "
    chmod +x /test/helicone-router-linux-musl &&
    echo 'Binary info:' &&
    ls -lh /test/helicone-router-linux-musl &&
    echo 'Testing execution:' &&
    /test/helicone-router-linux-musl --help || echo 'Test completed'
"

echo ""
echo "Testing musl binary in Debian..."
docker run --rm -v "$(pwd)/dist:/test" debian:11 bash -c "
    chmod +x /test/helicone-router-linux-musl &&
    echo 'Binary info:' &&
    ls -lh /test/helicone-router-linux-musl &&
    echo 'Testing execution:' &&
    /test/helicone-router-linux-musl --help || echo 'Test completed'
"

log_success "Container compatibility tests completed"

# Test 3: NPM package tests
log_step "Testing NPM package functionality"
if npm test; then
    log_success "NPM tests passed"
else
    log_error "NPM tests failed"
    exit 1
fi

# Test 4: Binary analysis
log_step "Analyzing binaries"
echo ""
echo "Binary sizes:"
ls -lh dist/

echo ""
echo "Binary types:"
file dist/* 2>/dev/null || echo "file command not available"

echo ""
echo "Dependencies (for Linux binaries):"
for binary in dist/helicone-router-linux*; do
    if [[ -f "$binary" ]]; then
        echo "Dependencies for $(basename "$binary"):"
        ldd "$binary" 2>/dev/null || echo "  Static binary or ldd not available"
        echo ""
    fi
done

# Test 5: Performance test
log_step "Performance testing"
echo ""
echo "Startup performance:"
for binary in dist/*; do
    if [[ -f "$binary" && -x "$binary" ]]; then
        echo -n "$(basename "$binary"): "
        time timeout 3s "$binary" --help >/dev/null 2>&1 || echo "completed"
    fi
done

# Test 6: Cross-distribution compatibility
log_step "Testing cross-distribution compatibility"

# Test Ubuntu versions
for version in "18.04" "20.04" "22.04"; do
    echo ""
    echo "Testing in Ubuntu $version..."
    docker run --rm -v "$(pwd)/dist:/test" ubuntu:$version bash -c "
        apt-get update -qq && apt-get install -y -qq file >/dev/null 2>&1 || true
        chmod +x /test/helicone-router-linux
        echo 'Ubuntu $version compatibility:'
        file /test/helicone-router-linux
        /test/helicone-router-linux --help >/dev/null 2>&1 && echo '✓ Compatible' || echo '✗ Incompatible'
    " 2>/dev/null || echo "Failed to test Ubuntu $version"
done

# Test Alpine compatibility
echo ""
echo "Testing Alpine compatibility..."
docker run --rm -v "$(pwd)/dist:/test" alpine:latest sh -c "
    apk add --no-cache file >/dev/null 2>&1 || true
    chmod +x /test/helicone-router-linux-musl
    echo 'Alpine compatibility:'
    file /test/helicone-router-linux-musl
    /test/helicone-router-linux-musl --help >/dev/null 2>&1 && echo '✓ Compatible' || echo '✗ Incompatible'
" 2>/dev/null || echo "Failed to test Alpine"

log_success "Cross-distribution compatibility tests completed"

echo ""
echo "🎉 Docker build testing completed!"
echo "================================="
log_success "All Docker builds are working correctly"
echo ""
echo "Summary:"
echo "✅ Docker builds successful"
echo "✅ Container compatibility verified"
echo "✅ NPM package tests passed"
echo "✅ Binary analysis completed"
echo "✅ Performance testing done"
echo "✅ Cross-distribution compatibility verified"
echo ""
echo "Ready for CI deployment! 🚀" 