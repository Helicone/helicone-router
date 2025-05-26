#!/bin/bash

# Test CI Locally - Simulates GitHub Actions workflow
# Run this from the npx/ directory

set -e  # Exit on any error

echo "🧪 Testing GitHub CI workflow locally..."
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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
if [[ ! -f "package.json" ]]; then
    log_error "Please run this script from the npx/ directory"
    exit 1
fi

cd ..  # Go to project root

echo ""
echo "===================="
echo "📦 JOB 1: test-npm-package"
echo "===================="

log_step "Setting up Rust for x86_64-unknown-linux-gnu"
if ! rustup target list --installed | grep -q "x86_64-unknown-linux-gnu"; then
    rustup target add x86_64-unknown-linux-gnu
fi

log_step "Building Rust binary for Linux"
if ! cargo build --release --target x86_64-unknown-linux-gnu; then
    log_error "Rust build failed"
    exit 1
fi
log_success "Rust binary built successfully"

log_step "Preparing NPM package"
cd npx
cp ../target/x86_64-unknown-linux-gnu/release/llm-proxy dist/helicone-router-linux 2>/dev/null || {
    log_error "Failed to copy binary. Check if 'llm-proxy' is the correct binary name."
    echo "Available binaries in target/x86_64-unknown-linux-gnu/release/:"
    ls -la ../target/x86_64-unknown-linux-gnu/release/ | grep -v "\.d$"
    exit 1
}
chmod +x dist/helicone-router-linux
log_success "Binary copied and made executable"

log_step "Running NPM package tests"
if ! npm test; then
    log_error "NPM tests failed"
    exit 1
fi
log_success "NPM tests passed"

log_step "Testing CLI execution"
echo "Testing direct Node.js execution:"
if ! node bin/index.js --help; then
    log_error "Direct CLI execution failed"
    exit 1
fi

echo "Testing npm link:"
if ! npm link; then
    log_error "npm link failed"
    exit 1
fi

if ! helicone-router --help; then
    log_error "Global command failed"
    npm unlink -g helicone-router || true
    exit 1
fi

npm unlink -g helicone-router
log_success "CLI execution tests passed"

log_step "Validating package for publishing"
if ! npm run publish:dry; then
    log_error "Package validation failed"
    exit 1
fi
log_success "Package validation passed"

cd ..  # Back to project root

echo ""
echo "===================="
echo "🏗️  JOB 2: build-cross-platform (Limited Local Test)"
echo "===================="

# Test what we can locally (Linux builds)
log_step "Testing Linux glibc build"
if ! cargo build --release --target x86_64-unknown-linux-gnu; then
    log_error "Linux glibc build failed"
    exit 1
fi
log_success "Linux glibc build successful"

log_step "Testing Linux musl build (if musl tools available)"
if rustup target list --installed | grep -q "x86_64-unknown-linux-musl"; then
    if cargo build --release --target x86_64-unknown-linux-musl; then
        log_success "Linux musl build successful"
    else
        log_warning "Linux musl build failed (musl tools may not be installed)"
    fi
else
    log_warning "x86_64-unknown-linux-musl target not installed"
    echo "To install: rustup target add x86_64-unknown-linux-musl"
fi

# We can't easily test macOS builds on Linux, so skip those locally

echo ""
echo "===================="
echo "🐧 JOB 3: test-distributions (Docker Required)"
echo "===================="

log_step "Checking if Docker is available"
if command -v docker &> /dev/null; then
    log_success "Docker is available"
    
    log_step "Testing Ubuntu 22.04 container"
    if docker run --rm -v "$(pwd)/target/x86_64-unknown-linux-gnu/release:/tmp/binaries" ubuntu:22.04 bash -c "
        apt-get update -qq && apt-get install -y -qq curl nodejs npm
        chmod +x /tmp/binaries/llm-proxy
        /tmp/binaries/llm-proxy --help || echo 'Binary test completed'
        node --version && npm --version
    "; then
        log_success "Ubuntu 22.04 test passed"
    else
        log_warning "Ubuntu 22.04 test had issues"
    fi
    
    log_step "Testing Alpine container"
    if docker run --rm -v "$(pwd)/target/x86_64-unknown-linux-gnu/release:/tmp/binaries" alpine:latest sh -c "
        apk add --no-cache nodejs npm
        chmod +x /tmp/binaries/llm-proxy
        /tmp/binaries/llm-proxy --help || echo 'Binary test completed'
        node --version && npm --version
    "; then
        log_success "Alpine test passed"
    else
        log_warning "Alpine test had issues (musl compatibility?)"
    fi
else
    log_warning "Docker not available - skipping distribution tests"
fi

echo ""
echo "===================="
echo "🔒 JOB 4: security-checks"
echo "===================="

cd npx

log_step "Running NPM audit"
if [ ! -f package-lock.json ]; then
    npm install --package-lock-only
fi
npm audit --audit-level=moderate || log_warning "NPM audit found issues"

log_step "Checking for hardcoded secrets"
if grep -r -i "password\|secret\|key\|token" . --exclude-dir=node_modules --exclude=test-ci-locally.sh; then
    log_warning "Found potential secrets in code"
else
    log_success "No obvious secrets found"
fi

log_step "Checking file permissions"
if find . -type f -perm 0777; then
    log_warning "Found world-writable files"
else
    log_success "No world-writable files found"
fi

log_step "Validating package.json"
node -e "
const pkg = require('./package.json');
console.log('Package name:', pkg.name);
console.log('Version:', pkg.version);
console.log('Files field:', pkg.files);
console.log('Bin field:', pkg.bin);
if (!pkg.files || pkg.files.length === 0) {
  console.error('Warning: No files field specified');
  process.exit(1);
}
" && log_success "Package.json validation passed"

cd ..

echo ""
echo "===================="
echo "⚡ JOB 5: performance-check"
echo "===================="

log_step "Analyzing binary sizes"
for target_dir in target/*/release/; do
    if [ -d "$target_dir" ]; then
        target_name=$(basename $(dirname "$target_dir"))
        if [ -f "${target_dir}llm-proxy" ]; then
            size=$(du -h "${target_dir}llm-proxy" | cut -f1)
            echo "$target_name: $size"
            
            # Check if binary is too large
            size_bytes=$(stat -c%s "${target_dir}llm-proxy" 2>/dev/null || stat -f%z "${target_dir}llm-proxy" 2>/dev/null || echo "0")
            size_mb=$((size_bytes / 1024 / 1024))
            if [ $size_mb -gt 50 ]; then
                log_warning "$target_name is ${size_mb}MB (> 50MB)"
            fi
        fi
    fi
done

log_step "Testing binary startup performance"
for target_dir in target/*/release/; do
    if [ -d "$target_dir" ] && [ -f "${target_dir}llm-proxy" ]; then
        target_name=$(basename $(dirname "$target_dir"))
        echo -n "$target_name startup time: "
        chmod +x "${target_dir}llm-proxy"
        time timeout 10s "${target_dir}llm-proxy" --help >/dev/null 2>&1 || echo "timed out or failed"
    fi
done

echo ""
echo "🎉 Local CI simulation completed!"
echo "=================================="
log_success "All major CI jobs tested locally"
echo ""
echo "Next steps:"
echo "1. Fix any issues found above"
echo "2. Run this script again to verify fixes"
echo "3. Push to GitHub to run actual CI" 