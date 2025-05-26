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

// Get the appropriate binary name based on platform
function getBinaryName() {
    const platform = os.platform();

    switch (platform) {
        case 'darwin':
            return 'helicone-router-macos';
        case 'linux':
            return 'helicone-router-linux';
        default:
            console.error(`❌ Error: Unsupported platform: ${platform}`);
            console.error('Supported platforms: macOS (darwin), Linux (linux)');
            process.exit(1);
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
        console.error(`❌ Error: Binary not found at ${binaryPath}`);
        console.error('Please ensure the Rust binary is compiled and placed in the dist/ directory.');
        process.exit(1);
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

module.exports = { main, checkNodeVersion, getBinaryName }; 