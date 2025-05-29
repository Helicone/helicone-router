#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

// Check Node.js version
function checkNodeVersion() {
    const nodeVersion = process.version;
    const majorVersion = parseInt(nodeVersion.slice(1).split('.')[0]);

    if (majorVersion < 16) {
        console.error('❌ Error: Node.js version 16 or higher is required.');
        console.error(`Current version: ${nodeVersion}`);
        process.exit(1);
    }
}

// Show Rust installation instructions
function showRustInstallInstructions() {
    console.log('\n🦀 Rust Installation Required');
    console.log('═══════════════════════════════════════════════════════════════');
    console.log('');
    console.log('To use Helicone Router on your platform, you\'ll need to install it via Rust.');
    console.log('');
    console.log('🚀 Option 1: Use cargo-dist (Recommended)');
    console.log('If you\'re a maintainer, you can use cargo-dist to build cross-platform binaries:');
    console.log('');
    console.log('  # Install cargo-dist');
    console.log('  cargo install cargo-dist');
    console.log('');
    console.log('  # Initialize cargo-dist in your project');
    console.log('  cargo dist init');
    console.log('');
    console.log('  # Build for multiple platforms');
    console.log('  cargo dist build');
    console.log('');
    console.log('📦 Option 2: Direct Installation');
    console.log('For end users, install directly from source:');
    console.log('');
    console.log('📋 Step 1: Install Rust');
    console.log('Run the following command to install Rust:');
    console.log('');
    console.log('  curl --proto \'=https\' --tlsv1.2 -sSf https://sh.rustup.rs | sh');
    console.log('');
    console.log('📋 Step 2: Restart your terminal or run:');
    console.log('');
    console.log('  source ~/.cargo/env');
    console.log('');
    console.log('📋 Step 3: Install Helicone Router');
    console.log('');
    console.log('  cargo install helicone-router');
    console.log('');
    console.log('📋 Step 4: Run Helicone Router');
    console.log('');
    console.log('  helicone-router');
    console.log('');
    console.log('═══════════════════════════════════════════════════════════════');
    console.log('');
    console.log('💡 Learn more:');
    console.log('  • Rust installation: https://rustup.rs/');
    console.log('  • cargo-dist: https://github.com/astral-sh/cargo-dist');
    console.log('');
}

// Get the appropriate binary name based on platform
function getBinaryName() {
    const platform = os.platform();
    const arch = os.arch();

    switch (platform) {
        case 'darwin':
            if (arch === 'arm64') {
                return 'helicone-router-macos';
            } else {
                console.log(`⚠️  Warning: macOS ${arch} architecture detected.`);
                console.log('Pre-built binaries are only available for macOS ARM64 (Apple Silicon).');
                showRustInstallInstructions();
                process.exit(0);
            }
        case 'linux':
            if (arch === 'x64') {
                return 'helicone-router-linux';
            } else {
                console.log(`⚠️  Warning: Linux ${arch} architecture detected.`);
                console.log('Pre-built binaries are only available for Linux x86_64.');
                showRustInstallInstructions();
                process.exit(0);
            }
        default:
            console.log(`⚠️  Warning: Unsupported platform: ${platform} ${arch}`);
            console.log('Pre-built binaries are only available for:');
            console.log('  • macOS ARM64 (Apple Silicon)');
            console.log('  • Linux x86_64');
            showRustInstallInstructions();
            process.exit(0);
    }
}

// Main execution function
function main() {
    // Check Node.js version first
    checkNodeVersion();

    // Get binary name and path
    const binaryName = getBinaryName();
    const binaryPath = path.join(__dirname, '..', 'dist', binaryName);

    // Check if binary exists
    if (!fs.existsSync(binaryPath)) {
        console.log(`⚠️  Warning: Pre-built binary not found at ${binaryPath}`);
        console.log('This could mean the binary wasn\'t included in the package or your platform isn\'t supported.');
        showRustInstallInstructions();
        process.exit(0);
    }

    // Check if binary is executable
    try {
        fs.accessSync(binaryPath, fs.constants.X_OK);
    } catch (error) {
        console.error(`❌ Error: Binary at ${binaryPath} is not executable.`);
        console.error('Run: chmod +x ' + binaryPath);
        process.exit(1);
    }

    // Show friendly startup message
    console.log('🚀 Starting Helicone Router...');

    // Get CLI arguments (excluding node and script name)
    const args = process.argv.slice(2);

    // Spawn the binary with forwarded arguments
    const child = spawn(binaryPath, args, {
        stdio: 'inherit',
        shell: false
    });

    // Handle child process events
    child.on('error', (error) => {
        console.error(`❌ Error executing binary: ${error.message}`);
        process.exit(1);
    });

    child.on('close', (code) => {
        process.exit(code || 0);
    });

    // Handle process termination signals
    process.on('SIGINT', () => {
        child.kill('SIGINT');
    });

    process.on('SIGTERM', () => {
        child.kill('SIGTERM');
    });
}

// Run the main function
if (require.main === module) {
    main();
}

module.exports = { main, checkNodeVersion, getBinaryName, showRustInstallInstructions }; 