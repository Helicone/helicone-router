# Docker-Based Build System

This document explains how to build the Helicone Router CLI package using Docker containers for maximum reliability and consistency.

## 🐳 **Why Docker?**

- **Consistent environments**: Same build environment everywhere
- **No cross-compilation issues**: Build natively in each target environment
- **Reproducible builds**: Exact same results every time  
- **Easy testing**: Test in multiple Linux distributions effortlessly
- **No dependency conflicts**: Isolated build environments

## 🚀 **Quick Start**

### Build All Binaries
```bash
# From project root
./npx/docker/build-docker.sh

# Or using npm scripts (from npx/ directory)
npm run build:docker
```

### Test Everything
```bash
# From npx/ directory
npm run test:docker
```

## 📁 **Docker Files Overview**

```
npx/docker/
├── Dockerfile.linux          # Linux glibc binary (Ubuntu, Debian, etc.)
├── Dockerfile.linux-musl     # Linux musl binary (Alpine, static)
├── docker-compose.yml        # Parallel builds and tests
└── build-docker.sh          # Build script
```

## 🏗️ **Build Targets**

### 1. Linux glibc (`Dockerfile.linux`)
- **Base**: `rust:1.75-slim-bullseye`
- **Target**: `x86_64-unknown-linux-gnu`
- **Compatible with**: Ubuntu, Debian, CentOS, RHEL, etc.
- **Dependencies**: Links to system glibc

### 2. Linux musl (`Dockerfile.linux-musl`)  
- **Base**: `rust:1.75-alpine`
- **Target**: `x86_64-unknown-linux-musl`
- **Compatible with**: Alpine, or any Linux (static binary)
- **Dependencies**: Statically linked

### 3. macOS (Native)
- **Platform**: macOS runner (no Docker option)
- **Target**: `x86_64-apple-darwin` or `aarch64-apple-darwin`
- **Note**: Apple doesn't allow macOS in Docker on non-Apple hardware

## 🛠️ **Build Commands**

### Individual Builds
```bash
# Linux glibc
docker build -f npx/docker/Dockerfile.linux -t helicone-router:linux .
docker run --rm -v "$PWD/npx/dist:/dist" helicone-router:linux

# Linux musl (Alpine-compatible)
docker build -f npx/docker/Dockerfile.linux-musl -t helicone-router:linux-musl .
docker run --rm -v "$PWD/npx/dist:/dist" helicone-router:linux-musl

# macOS (if on macOS)
cargo build --release
cp target/release/llm-proxy npx/dist/helicone-router-macos
```

### Parallel Builds with Docker Compose
```bash
cd npx/docker
docker-compose up build-linux build-linux-musl
```

## 🧪 **Testing**

### Quick Test
```bash
cd npx
npm run test:docker
```

### Manual Distribution Testing
```bash
# Test Ubuntu compatibility
docker run --rm -v "$PWD/npx/dist:/test" ubuntu:20.04 bash -c "
  chmod +x /test/helicone-router-linux && 
  /test/helicone-router-linux --help
"

# Test Alpine compatibility  
docker run --rm -v "$PWD/npx/dist:/test" alpine:latest sh -c "
  chmod +x /test/helicone-router-linux-musl && 
  /test/helicone-router-linux-musl --help
"

# Test multiple Ubuntu versions
for version in 18.04 20.04 22.04; do
  echo "Testing Ubuntu $version..."
  docker run --rm -v "$PWD/npx/dist:/test" ubuntu:$version bash -c "
    chmod +x /test/helicone-router-linux && 
    /test/helicone-router-linux --help && 
    echo '✅ Ubuntu $version compatible'
  "
done
```

### Automated Distribution Testing
```bash
cd npx/docker
docker-compose up test-ubuntu test-alpine
```

## 📊 **Binary Analysis**

### Check Binary Types
```bash
file npx/dist/*
```

### Check Dependencies
```bash
# glibc binary dependencies
ldd npx/dist/helicone-router-linux

# musl binary (should show "statically linked")
ldd npx/dist/helicone-router-linux-musl
```

### Size Analysis
```bash
ls -lh npx/dist/
du -sh npx/dist/
```

## 🎯 **CI Integration**

The Docker approach is used in the CI workflow:

```yaml
# .github/workflows/npm-cli-ci-docker.yml
jobs:
  build-linux-docker:
    strategy:
      matrix:
        include:
          - dockerfile: npx/docker/Dockerfile.linux
            binary: helicone-router-linux
          - dockerfile: npx/docker/Dockerfile.linux-musl  
            binary: helicone-router-linux-musl
    
    steps:
      - name: Build binary with Docker
        run: |
          docker build -f ${{ matrix.dockerfile }} -t temp-image .
          docker run --rm -v "$PWD/dist:/dist" temp-image
```

## 🐛 **Troubleshooting**

### Docker Issues

**"No space left on device"**
```bash
docker system prune -a
npm run clean:docker
```

**"Permission denied" in containers**
```bash
# Make sure binaries are executable
chmod +x npx/dist/*
```

**Build failures**
```bash
# Check Docker logs
docker build -f npx/docker/Dockerfile.linux .

# Test step by step
docker run -it rust:1.75-slim-bullseye bash
```

### Binary Issues

**"No such file or directory" on Linux**
- Check if you're using the right binary (glibc vs musl)
- Use `ldd` to check dependencies
- Try the musl binary (statically linked)

**Binary too large**
- Check build flags in Dockerfiles
- Consider stripping symbols: `strip npx/dist/*`

### Container Compatibility

**Alpine compatibility**
- Always use the musl binary (`helicone-router-linux-musl`)
- glibc binaries won't work in Alpine

**Old Linux distributions**
- Use musl binary for maximum compatibility
- Or build in older containers for glibc compatibility

## ✅ **Best Practices**

1. **Always test locally first**:
   ```bash
   npm run build:docker && npm run test:docker
   ```

2. **Use appropriate binary for target**:
   - **Ubuntu/Debian/RHEL**: `helicone-router-linux` (glibc)
   - **Alpine/Static**: `helicone-router-linux-musl` (musl)
   - **Maximum compatibility**: `helicone-router-linux-musl`

3. **Keep Docker images clean**:
   ```bash
   npm run clean:docker  # Cleanup unused images
   ```

4. **Test in target environment**:
   ```bash
   # Test in your actual deployment environment
   docker run --rm -v "$PWD/npx/dist:/test" your-base-image:tag bash -c "
     chmod +x /test/helicone-router-linux && 
     /test/helicone-router-linux --help
   "
   ```

## 📈 **Performance Comparison**

| Method | Build Time | Reliability | Cross-platform | Setup Complexity |
|--------|-----------|-------------|----------------|------------------|
| Docker | Slower (~3-5min) | Highest ✅ | Linux only | Low |
| Native | Faster (~1-2min) | Medium | Platform-specific | Medium |
| Cross-compilation | Medium (~2-3min) | Low ❌ | Yes | High |

**Recommendation**: Use Docker for CI/production, native for development.

## 🚀 **Ready for Production**

Once all Docker tests pass:

1. **Commit the Docker setup**:
   ```bash
   git add npx/docker/ npx/DOCKER_BUILD.md
   git commit -m "Add Docker-based build system"
   ```

2. **Update CI to use Docker workflow**:
   - Use `.github/workflows/npm-cli-ci-docker.yml`
   - Disable cross-compilation workflows

3. **Test on GitHub Actions**:
   - Push changes
   - Monitor CI results
   - All builds should be ✅ green

**The Docker approach eliminates the cross-compilation failures completely!** 🎉 