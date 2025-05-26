# Building and Testing Helicone Router CLI

This guide explains how to build and test the Node.js CLI wrapper locally before publishing.

## Prerequisites

- Node.js 16 or higher
- Rust toolchain installed
- Access to macOS and Linux for cross-compilation (optional)

## Building

### 1. Build for macOS (current platform)
```bash
npm run build
```

This will:
- Build the Rust project in release mode
- Copy the binary to `dist/helicone-router-macos`
- Make it executable

### 2. Build for Linux (cross-compilation)
```bash
# First, add the Linux target (one-time setup)
rustup target add x86_64-unknown-linux-gnu

# Then build for Linux
npm run build:linux
```

## Testing Locally

### 1. Run the comprehensive test suite
```bash
npm test
```

This runs `test-local.js` which validates:
- Package structure
- Binary existence and permissions
- CLI execution
- All expected files are present

### 2. Test with npm link (simulates global installation)
```bash
# Link the package locally
npm link

# Test the global command
helicone-router --help

# Test with npx from any directory
cd /tmp
npx helicone-router --help

# Unlink when done testing
npm unlink -g helicone-router
```

### 3. Test what will be published
```bash
npm run publish:dry
```

This shows exactly what files would be included in the published package.

## Publishing Process

### 1. Pre-flight checks
```bash
# Ensure you're logged in to npm
npm whoami

# Run all tests
npm test

# Check what will be published
npm run publish:dry
```

### 2. Publish to npm
```bash
npm publish
```

The `prepublishOnly` script will automatically run tests before publishing.

## File Structure After Build

```
/npx
├── bin/
│   └── index.js                  # Node CLI entry point
├── dist/
│   ├── helicone-router-macos     # macOS binary
│   └── helicone-router-linux     # Linux binary (if cross-compiled)
├── package.json
├── README.md
├── BUILD.md                      # This file
└── test-local.js                 # Test script
```

## Troubleshooting

### Binary not found
Make sure you've run `npm run build` to copy the Rust binary to the dist folder.

### Permission denied
The build script should automatically make binaries executable, but if needed:
```bash
chmod +x dist/helicone-router-*
```

### Cross-compilation issues
For Linux builds on macOS, you might need additional tools:
```bash
# Install cross-compilation tools
brew install FiloSottile/musl-cross/musl-cross
```

### Testing on different platforms
The package detects the platform automatically:
- macOS: Uses `helicone-router-macos`
- Linux: Uses `helicone-router-linux`

Make sure to test on the target platforms before publishing. 