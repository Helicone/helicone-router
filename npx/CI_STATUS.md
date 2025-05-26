# CI/CD Status for Helicone Router CLI

## GitHub Actions Workflows

### 🔧 NPM CLI Package CI
**File**: `.github/workflows/npm-cli-ci.yml`

This comprehensive workflow validates the Node.js CLI wrapper across multiple dimensions:

#### Jobs Overview:

1. **📦 test-npm-package**
   - Tests Node.js wrapper functionality
   - Validates package structure
   - Tests CLI execution
   - Simulates npm publishing

2. **🏗️ build-cross-platform**
   - Builds for multiple targets:
     - `x86_64-unknown-linux-gnu` (Linux glibc)
     - `x86_64-unknown-linux-musl` (Linux musl - Alpine compatible)
     - `x86_64-apple-darwin` (Intel macOS)
     - `aarch64-apple-darwin` (Apple Silicon macOS)

3. **🐧 test-distributions**
   - Tests in different Linux containers:
     - Ubuntu 20.04, 22.04
     - Debian 11
     - Alpine Linux
   - Validates binary execution across distributions

4. **✅ validate-package**
   - Comprehensive package validation with all binaries
   - Tests complete package structure
   - Validates binary types and permissions
   - Package size analysis

5. **🔒 security-checks**
   - NPM audit for vulnerabilities
   - Checks for hardcoded secrets
   - File permission validation
   - Package.json validation

6. **⚡ performance-check**
   - Binary size analysis
   - Startup performance testing
   - Warns about oversized binaries

#### Triggers:
- **Push** to `main` branch (when NPX-related files change)
- **Pull Requests** to `main` branch (when NPX-related files change)
- Path filters to only run when relevant files are modified

#### Path Filters:
```yaml
paths:
  - "npx/**"           # NPM package files
  - "crates/**"        # Rust source code
  - "Cargo.toml"       # Rust dependencies
  - "Cargo.lock"       # Rust lockfile
  - ".github/workflows/npm-cli-ci.yml"  # This workflow
```

## Status Badges

Add these to your main README.md:

```markdown
[![NPM CLI CI](https://github.com/Helicone/helicone-router/actions/workflows/npm-cli-ci.yml/badge.svg)](https://github.com/Helicone/helicone-router/actions/workflows/npm-cli-ci.yml)
[![Rust CI](https://github.com/Helicone/helicone-router/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/Helicone/helicone-router/actions/workflows/rust-ci.yml)
```

## Local Development Workflow

1. **Make changes** to NPX package or Rust code
2. **Test locally**: `cd npx && npm test`
3. **Create PR** - CI automatically runs
4. **Review CI results** - All jobs must pass
5. **Merge** when green ✅

## CI Performance

- **Parallel execution**: Multiple jobs run simultaneously
- **Caching**: Rust dependencies cached between runs
- **Artifacts**: Binaries saved for cross-job testing
- **Matrix builds**: Multiple platforms tested efficiently

## Troubleshooting CI

### Common Issues:

1. **Binary not found**
   - Check Rust build step succeeded
   - Verify binary path in copy commands

2. **Permission denied**
   - Ensure `chmod +x` steps are included
   - Check binary executable permissions

3. **Distribution test failures**
   - Verify glibc/musl compatibility
   - Check Node.js installation in containers

4. **Package validation failures**
   - Run `npm test` locally first
   - Check package.json files field

### Debugging Steps:

1. Check the specific job that failed
2. Look at the step logs
3. Reproduce locally using the same commands
4. Test in the same container/OS if needed

## Future Enhancements

- [ ] Add ARM64 Linux builds
- [ ] Windows builds (if needed)
- [ ] Release automation
- [ ] Performance benchmarking
- [ ] Dependency vulnerability scanning
- [ ] Code coverage reporting 