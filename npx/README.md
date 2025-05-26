# Helicone Router CLI

A Node.js CLI wrapper for the Helicone Router - a high-performance proxy router for LLM APIs built with Rust.

## Installation

### Using npx (Recommended)
```bash
npx helicone-router --start
```

### Global Installation
```bash
npm install -g helicone-router
helicone-router --start
```

## Usage

Once installed, you can use the `helicone-router` command from anywhere:

```bash
# Start the router
helicone-router --start

# Check version
helicone-router --version

# Get help
helicone-router --help
```

All arguments are forwarded directly to the underlying Rust binary, so you can use any flags and options that the Rust CLI supports.

## System Requirements

- **Node.js**: Version 16 or higher
- **Supported Platforms**: 
  - macOS (darwin)
  - Linux (linux)

## How It Works

This package includes pre-compiled Rust binaries for different platforms:
- `helicone-router-macos` for macOS
- `helicone-router-linux` for Linux

The Node.js wrapper automatically detects your operating system and runs the appropriate binary with your provided arguments.

## Troubleshooting

### "Binary not found" Error
Make sure you're using a supported platform (macOS or Linux). If you're on a supported platform and still getting this error, the binary might be missing from the package.

### "Binary is not executable" Error
Run the following command to make the binary executable:
```bash
chmod +x ./node_modules/helicone-router/dist/helicone-router-*
```

### "Node.js version" Error
Upgrade to Node.js version 16 or higher:
```bash
# Using nvm
nvm install 16
nvm use 16

# Or download from https://nodejs.org/
```

## Contributing

This CLI wrapper is part of the [Helicone Router](https://github.com/Helicone/helicone-router) project. Please refer to the main repository for contributing guidelines.

## License

MIT 