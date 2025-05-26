#!/bin/bash

# Simplified Local CI Test - Tests what we can actually test locally
# Run this from the npx/ directory

set -e  # Exit on any error

echo "🧪 Testing NPM CLI package locally (simplified)..."
echo "=================================================="

# Colors for output
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

# Check we're in the right directory
if [[ ! -f "package.json" ]]; then
    echo "❌ Please run this script from the npx/ directory"
    exit 1
fi

echo ""
echo "===================="
echo "📦 NPM Package Tests"
echo "===================="

log_step "Building for current platform ($(uname -m)-$(uname -s))"
cd ..  # Go to project root
if cargo build --release; then
    log_success "Rust binary built successfully"
else
    log_warning "Rust build failed - this might affect CI"
fi

cd npx

log_step "Copying binary for current platform"
if [[ "$OSTYPE" == "darwin"* ]]; then
    if [[ -f "../target/release/llm-proxy" ]]; then
        cp ../target/release/llm-proxy dist/helicone-router-macos
        chmod +x dist/helicone-router-macos
        log_success "macOS binary copied"
    else
        echo "❌ macOS binary not found"
        exit 1
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if [[ -f "../target/release/llm-proxy" ]]; then
        cp ../target/release/llm-proxy dist/helicone-router-linux
        chmod +x dist/helicone-router-linux
        log_success "Linux binary copied"
    else
        echo "❌ Linux binary not found"
        exit 1
    fi
fi

log_step "Running NPM package tests"
if npm test; then
    log_success "NPM tests passed"
else
    echo "❌ NPM tests failed"
    exit 1
fi

log_step "Testing CLI execution"
if node bin/index.js --help > /dev/null; then
    log_success "CLI execution works"
else
    echo "❌ CLI execution failed"
    exit 1
fi

log_step "Testing npm link"
if npm link && helicone-router --help > /dev/null; then
    npm unlink -g helicone-router
    log_success "Global CLI works"
else
    npm unlink -g helicone-router || true
    echo "❌ Global CLI failed"
    exit 1
fi

log_step "Validating package for publishing"
if npm run publish:dry > /dev/null; then
    log_success "Package validation passed"
else
    echo "❌ Package validation failed"
    exit 1
fi

echo ""
echo "===================="
echo "🔒 Security Checks"
echo "===================="

log_step "NPM audit"
if [ ! -f package-lock.json ]; then
    npm install --package-lock-only
fi
npm audit --audit-level=moderate || log_warning "NPM audit found issues"

log_step "Package.json validation"
node -e "
const pkg = require('./package.json');
if (!pkg.files || pkg.files.length === 0) {
  throw new Error('No files field specified');
}
console.log('✓ Package.json is valid');
"

echo ""
echo "🎉 Local tests completed!"
echo "========================="
log_success "All testable components passed"
echo ""
echo "What this test covers:"
echo "✅ Rust compilation for current platform"
echo "✅ NPM package structure and tests"
echo "✅ CLI wrapper functionality"
echo "✅ Package publishing validation"
echo "✅ Basic security checks"
echo ""
echo "What still needs GitHub CI:"
echo "⚠️  Cross-platform builds (Linux, macOS Intel/ARM)"
echo "⚠️  Distribution testing (Ubuntu, Alpine, etc.)"
echo "⚠️  Performance analysis across platforms"
echo ""
echo "Ready to push to GitHub for full CI testing! 🚀" 