# Installation Guide

## Prerequisites
- Rust 1.70+
- Docker & Docker Compose (for testing generated files)

## Quick Installation

### Using Makefile (Linux/macOS)
```bash
# Clone and install in one step
git clone https://github.com/Jeck0v/PUBLIC-Athena-Tools.git
cd PUBLIC-Athena-Tools
make install
```

### Manual Installation
```bash
git https://github.com/Jeck0v/PUBLIC-Athena-Tools.git
cd PUBLIC-Athena-Tools
cargo install --path .
```

### Windows Installation
For Windows users, see [README-Windows.md](../README-Windows.md) for PowerShell installation instructions:
```powershell
.\build.ps1 install
```

## Verify Installation
```bash
athena --version        # Check version
athena info --examples # View DSL examples
which athena           # Should show: ~/.cargo/bin/athena

# Or use the makefile helper
make check-install
```

## Development Installation

### Using Makefile Commands
```bash
# Show all available commands
make help

# Build the project
make build

# Run tests
make test

# Development mode (build + test + info)
make dev

# Install locally
make install

# System-wide installation (requires sudo)
make install-system

# Run demo
make demo

# Clean build artifacts
make clean

# Uninstall
make uninstall
```

## Building from Source

### Debug Build
```bash
cargo build
```

### Release Build (Optimized)
```bash
cargo build --release
# Or use: make build
```

### Install Locally
```bash
cargo install --path .
# Or use: make install
```

## Platform-Specific Instructions

### Linux/macOS
Use the Makefile commands above for the best experience.

### Windows
See [README-Windows.md](../README-Windows.md) for complete Windows installation guide with PowerShell scripts.
