#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('🧪 Testing Helicone Router CLI package locally...\n');

// Test 1: Check package.json exists and is valid
console.log('✅ Checking package.json...');
const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
console.log(`   - Name: ${packageJson.name}`);
console.log(`   - Version: ${packageJson.version}`);
console.log(`   - Node requirement: ${packageJson.engines.node}`);
console.log(`   - Supported OS: ${packageJson.os.join(', ')}`);

// Test 2: Check bin file exists and is executable
console.log('\n✅ Checking Node.js wrapper...');
const binPath = path.join('bin', 'index.js');
if (!fs.existsSync(binPath)) {
    console.error('❌ bin/index.js not found');
    process.exit(1);
}
console.log(`   - bin/index.js exists`);

// Check if bin file has shebang
const binContent = fs.readFileSync(binPath, 'utf8');
if (!binContent.startsWith('#!/usr/bin/env node')) {
    console.error('❌ bin/index.js missing Node.js shebang');
    process.exit(1);
}
console.log(`   - bin/index.js has proper shebang`);

// Test 3: Check dist directory and ALL expected platform binaries
console.log('\n✅ Checking dist directory and binaries...');
if (!fs.existsSync('dist')) {
    console.error('❌ dist directory not found');
    process.exit(1);
}

const expectedBinaries = [
    { name: 'helicone-router-macos', platform: 'darwin' },
    { name: 'helicone-router-linux', platform: 'linux' }
];

const distFiles = fs.readdirSync('dist');
console.log(`   - Dist contains: ${distFiles.join(', ')}`);

for (const binary of expectedBinaries) {
    const binaryPath = path.join('dist', binary.name);

    if (fs.existsSync(binaryPath)) {
        console.log(`   ✅ ${binary.name} exists`);

        // Check file permissions
        try {
            fs.accessSync(binaryPath, fs.constants.X_OK);
            console.log(`   ✅ ${binary.name} is executable`);
        } catch (error) {
            console.error(`   ❌ ${binary.name} is not executable`);
            process.exit(1);
        }

        // Check file size (warn if too large)
        const stats = fs.statSync(binaryPath);
        const sizeMB = (stats.size / 1024 / 1024).toFixed(2);
        console.log(`   - ${binary.name} size: ${sizeMB}MB`);

        if (stats.size > 50 * 1024 * 1024) { // 50MB warning
            console.warn(`   ⚠️  ${binary.name} is quite large (${sizeMB}MB) - consider optimization`);
        }

        // Check if it's actually a binary (not a text file)
        const firstBytes = fs.readFileSync(binaryPath, { encoding: null }).subarray(0, 4);
        const isELF = firstBytes[0] === 0x7f && firstBytes[1] === 0x45 && firstBytes[2] === 0x4c && firstBytes[3] === 0x46;
        const isMachO = firstBytes[0] === 0xcf || firstBytes[0] === 0xce; // Mach-O magic numbers

        if (binary.platform === 'linux' && !isELF) {
            console.warn(`   ⚠️  ${binary.name} doesn't appear to be a Linux ELF binary`);
        }
        if (binary.platform === 'darwin' && !isMachO) {
            console.warn(`   ⚠️  ${binary.name} doesn't appear to be a macOS Mach-O binary`);
        }

    } else {
        console.log(`   ⚠️  ${binary.name} missing (will only work on ${binary.platform === 'darwin' ? 'macOS' : 'Linux'})`);
    }
}

// Test 4: Test CLI execution on current platform
const currentPlatform = process.platform;
const currentBinary = currentPlatform === 'darwin' ? 'helicone-router-macos' : 'helicone-router-linux';
const currentBinaryPath = path.join('dist', currentBinary);

console.log('\n✅ Testing CLI execution...');
if (fs.existsSync(currentBinaryPath)) {
    try {
        const output = execSync('node bin/index.js --help', { encoding: 'utf8' });
        if (output.includes('🚀 Starting Helicone Router...')) {
            console.log('   - CLI wrapper executes successfully');
            console.log('   - Friendly startup message shown');
            console.log(`   - Successfully runs ${currentBinary} on ${currentPlatform}`);
        } else {
            console.error('❌ Unexpected CLI output');
            process.exit(1);
        }
    } catch (error) {
        console.error('❌ CLI execution failed:', error.message);
        process.exit(1);
    }
} else {
    console.warn(`   ⚠️  Cannot test CLI execution - ${currentBinary} not found for current platform (${currentPlatform})`);
}

// Test 5: Platform detection logic
console.log('\n✅ Testing platform detection...');
try {
    const { getBinaryName } = require('./bin/index.js');
    const detectedBinary = getBinaryName();
    console.log(`   - Detected binary for current platform: ${detectedBinary}`);

    if (!fs.existsSync(path.join('dist', detectedBinary))) {
        console.error(`   ❌ Detected binary ${detectedBinary} not found in dist/`);
        process.exit(1);
    }
} catch (error) {
    console.error('❌ Platform detection failed:', error.message);
    process.exit(1);
}

// Test 6: Validate package files
console.log('\n✅ Validating package structure...');
const expectedFiles = ['bin', 'dist', 'package.json', 'README.md'];
for (const file of expectedFiles) {
    if (!fs.existsSync(file)) {
        console.error(`❌ Expected file/directory ${file} not found`);
        process.exit(1);
    }
}
console.log('   - All expected files present');

// Test 7: Check package.json files field matches actual files
console.log('\n✅ Validating package.json files field...');
const actualDirs = ['bin', 'dist'].filter(dir => fs.existsSync(dir));
const specifiedFiles = packageJson.files || [];

for (const dir of actualDirs) {
    if (!specifiedFiles.includes(dir)) {
        console.warn(`   ⚠️  Directory ${dir} exists but not in package.json files field`);
    }
}

// Test 8: Calculate total package size
console.log('\n✅ Checking package size...');
const binSize = fs.statSync('bin/index.js').size;
const distSize = distFiles.reduce((total, file) => {
    return total + fs.statSync(path.join('dist', file)).size;
}, 0);

const totalSizeMB = ((binSize + distSize) / 1024 / 1024).toFixed(2);
console.log(`   - Total package size: ${totalSizeMB}MB`);

if (totalSizeMB > 100) {
    console.warn(`   ⚠️  Package is quite large (${totalSizeMB}MB) - npm has a 100MB limit per package`);
}

console.log('\n🎉 All tests passed! Package validation complete.');

// Test 9: Distribution-specific recommendations
console.log('\n📋 Distribution Compatibility Notes:');
console.log('   - Linux binary should be built with glibc compatibility in mind');
console.log('   - Consider statically linking for broader Linux compatibility');
console.log('   - Test on major distributions: Ubuntu, CentOS/RHEL, Alpine');
console.log('   - Consider providing musl-based builds for Alpine/containers');

console.log('\nNext steps:');
console.log('1. npm run publish:dry   (to see what would be published)');
console.log('2. npm publish          (to actually publish)');
console.log('3. Test on actual Linux systems before final publish');
console.log('4. npm unlink           (to remove local symlink)'); 