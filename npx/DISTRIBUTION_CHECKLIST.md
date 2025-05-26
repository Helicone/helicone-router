# Distribution Compatibility Checklist

This checklist covers important considerations for distributing the Helicone Router CLI across different platforms and Linux distributions.

## 🐧 Linux Distribution Compatibility

### Major Distributions to Test
- [ ] **Ubuntu LTS** (20.04, 22.04, 24.04)
- [ ] **Debian** (11, 12)
- [ ] **CentOS/RHEL** (8, 9)
- [ ] **Fedora** (latest 2 versions)
- [ ] **Alpine Linux** (for containers)
- [ ] **Amazon Linux 2**
- [ ] **SUSE/openSUSE**

### Architecture Support
- [ ] **x86_64** (Intel/AMD 64-bit) - Primary target
- [ ] **arm64/aarch64** (ARM 64-bit) - Growing importance (M1 Macs, AWS Graviton)
- [ ] **armv7** (32-bit ARM) - Raspberry Pi, IoT devices
- [ ] Consider which architectures are actually needed for your use case

### C Library Compatibility
- [ ] **glibc** - Most common on mainstream distributions
- [ ] **musl** - Used by Alpine Linux, smaller containers
- [ ] **Static linking** - Eliminates runtime dependencies
- [ ] Check minimum glibc version requirements

## 🔧 Build Considerations

### Cross-Compilation Setup
```bash
# Add Linux targets
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
rustup target add aarch64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-musl
```

### Docker-based Build for Consistent Environments
```dockerfile
# Example Dockerfile for building Linux binaries
FROM rust:1.70-slim AS builder
RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl
```

### Package Size Optimization
- [ ] Strip debug symbols: `strip target/release/binary`
- [ ] Use `cargo build --release` for optimized builds
- [ ] Consider UPX compression for smaller binaries
- [ ] Monitor total npm package size (100MB limit)

## 📦 Package Management Considerations

### NPM Package Limits
- [ ] Individual file size: 100MB max
- [ ] Total package size: Reasonable for quick installs
- [ ] Consider splitting large binaries into separate packages

### Alternative Distribution Methods
- [ ] **GitHub Releases** - For larger binaries
- [ ] **Docker Images** - Pre-built containers
- [ ] **Homebrew** - macOS/Linux package manager
- [ ] **APT/YUM repositories** - Native Linux packages
- [ ] **Snap/AppImage/Flatpak** - Universal Linux packages

## 🧪 Testing Strategy

### Automated Testing
- [ ] **GitHub Actions** - Multi-platform CI/CD
- [ ] **Docker containers** - Test in isolated environments
- [ ] **Matrix builds** - Multiple OS/arch combinations

### Manual Testing Checklist
- [ ] Test `npx helicone-router` on fresh systems
- [ ] Verify `--help` and basic commands work
- [ ] Check binary permissions and execution
- [ ] Test in containers (Docker, Podman)
- [ ] Validate on different shell environments (bash, zsh, fish)

### Container Testing
```bash
# Test in various containers
docker run --rm -it ubuntu:22.04 bash -c "curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash && source ~/.bashrc && nvm install 18 && npx helicone-router --help"
```

## 🔒 Security Considerations

### Binary Security
- [ ] Scan binaries for vulnerabilities
- [ ] Sign binaries (code signing certificates)
- [ ] Verify checksums in documentation
- [ ] Use official build environments

### NPM Package Security
- [ ] Enable 2FA on npm account
- [ ] Use `npm audit` to check dependencies
- [ ] Pin exact versions in package-lock.json
- [ ] Consider using `npm provenance` for transparency

## 🌐 Global Considerations

### Internationalization
- [ ] CLI help text in multiple languages
- [ ] Error messages are clear and helpful
- [ ] Support for different locale settings

### Network Requirements
- [ ] Test in environments with limited internet
- [ ] Consider offline usage scenarios
- [ ] Document any required network access

### Performance Considerations
- [ ] Binary startup time
- [ ] Memory usage across platforms
- [ ] CPU compatibility (older vs newer instruction sets)

## 📋 Pre-Release Checklist

### Documentation
- [ ] Update README with platform support
- [ ] Document known limitations
- [ ] Provide troubleshooting guide
- [ ] Include system requirements

### Version Management
- [ ] Use semantic versioning
- [ ] Tag releases in Git
- [ ] Maintain CHANGELOG.md
- [ ] Consider deprecation timeline for unsupported platforms

### Community Feedback
- [ ] Beta testing with community
- [ ] Issue templates for bug reports
- [ ] Feature request process
- [ ] Regular dependency updates

## 🚀 Deployment Pipeline

### Automated Release Process
```yaml
# Example GitHub Actions workflow
name: Release
on:
  push:
    tags: ['v*']
jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        target: [x86_64-unknown-linux-gnu, x86_64-apple-darwin]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          target: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - run: npm publish
```

### Monitoring and Analytics
- [ ] Track download statistics
- [ ] Monitor error reports
- [ ] Platform usage analytics
- [ ] Performance metrics

This checklist should be reviewed and updated regularly as the project evolves and new platforms/distributions become relevant. 