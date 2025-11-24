# Athena - Production-Ready DevOps Toolkit

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-0.1.0-green.svg)](Cargo.toml)

Athena is a powerful CLI tool for back-end developers and DevOps engineers that transforms a COBOL-inspired DSL into production-ready Docker Compose configurations with minimal effort.

Built with performance and maintainability in mind, Athena uses intelligent defaults and modern Docker standards to generate optimized configurations with minimal configuration.

## Why Athena DSL?

Writing infrastructure in plain YAML often leads to:

- **Repetition**: ports, env vars, healthchecks duplicated across files
- **Verbosity**: even small projects need hundreds of lines of config
- **Errors**: indentation, misplaced keys, and subtle schema mistakes
- **Low readability**: hard for newcomers to understand what's happening

Athena introduces a **COBOL-inspired DSL** designed for clarity and speed:

### Advantages over plain YAML
- **Declarative & explicit**: easy to read and understand at a glance
- **Minimal boilerplate**: no need to repeat Docker defaults
- **Error-resistant**: parser catches common mistakes early
- **Smart defaults**: healthchecks, restart policies, and networks added automatically
- **Composable**: same DSL can currently generate Docker Compose, and in the future Kubernetes and Terraform

### Example
Instead of writing verbose YAML:
```yaml
services:
  backend:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "8000:8000"
    environment:
      - DATABASE_URL=${DATABASE_URL}
  database:
    image: postgres:15
    restart: always
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      retries: 5
```

You just write:
```athena
DEPLOYMENT-ID MY_APP

SERVICES SECTION

SERVICE backend
PORT-MAPPING 8000 TO 8000
ENV-VARIABLE {{DATABASE_URL}}
END SERVICE

SERVICE database
IMAGE-ID postgres:15
END SERVICE
```

Athena expands this into **production-ready Docker Compose** with all the right defaults.

## Enhanced Error Handling

Athena now features **revolutionary error handling** with precise location information and intelligent suggestions:

### Before (Cryptic Errors)
```
Error: Parse error: Expected athena_file rule
```

### After (Enhanced Errors)
```
Error: Parse error at line 8, column 1: Missing 'END SERVICE' statement
   |
 8 | # Missing END SERVICE for demonstration
   | ^ Error here

Suggestion: Each SERVICE block must be closed with 'END SERVICE'
```

### Smart Validation with Suggestions
```
Error: Port conflict detected! Host port 8080 is used by multiple services: app1, app2
Affected services: app1, app2

Suggestion: Use different host ports, e.g., 8080, 8081
```

**[Learn more about Enhanced Error Handling →](docs/ERROR_HANDLING.md)**

## Quick Start

### Installation
```bash
# Install from source
git clone https://github.com/Jeck0v/Athena
cd athena
cargo install --path .

# Verify installation
athena --version
```

### Generate Docker Compose
```bash
# Create a simple deploy.ath file
echo 'DEPLOYMENT-ID MY_APP

SERVICES SECTION

SERVICE backend
PORT-MAPPING 8000 TO 8000
ENV-VARIABLE {{DATABASE_URL}}
END SERVICE

SERVICE database
IMAGE-ID postgres:15
END SERVICE' > deploy.ath

# Generate production-ready docker-compose.yml
athena build deploy.ath
```

## Key Features

### Enhanced Error Handling System (New!)
- **Line & Column Precision** => Exact error locations with visual context
- **Intelligent Suggestions** => Automatic recommendations for common fixes
- **Advanced Validation** => Port conflicts, service references, circular dependencies
- **Fail-Fast Processing** => Immediate feedback with no partial generation

### Intelligent Defaults 2025+
- Auto check for the Dockerfile
- Auto-detects service types database, Cache, WebApp, Proxy patterns
- Smart restart policies `always` for databases, `unless-stopped` for apps
- Optimized health checks different intervals per service type
- Container naming follows modern conventions (`project-service`)

### Docker-First Approach
- Dockerfile by default => No image? Just dont configure it and athena will check for your Dockerfile nativement
- Intelligent networking => Auto-configured networks with proper isolation
- Production-ready => Security, resource limits, and health monitoring
- Standards compliant => Follows Docker Compose 2025 best practices

### Performance Optimized
- Topological sorting => Services ordered by dependencies automatically
- Iterative validation => Fast circular dependency detection
- Optimized parsing => **<1ms parse time, <2ms generation**
- Memory efficient => Pre-allocated structures for large compositions

### Syntax Highlighting (SOON)
- **Beautiful DSL highlighting** for `.ath` files with customizable colors
- **Zed editor extension** ready to install in `syntax-highlighting/`
- **Smart color coding** for keywords, directives, template variables, and more
- **Easy customization** via `colors.json` make it your own!

## Documentation

### Core Documentation
- [Enhanced Error Handling (**New**)](docs/ERROR_HANDLING.md) - Complete guide to Athena's advanced error system.
- [Syntax Highlighting (**New**)](syntax-highlighting/README.md) - Beautiful colors for `.ath` files in Zed editor.
- [Installation Guide](docs/INSTALLATION.md)
- [Docker Compose Generator Usage](docs/DSL_REFERENCE.md)
- [Examples](docs/EXAMPLES.md)

### Development
- [Architecture Overview](docs/ARCHITECTURE.md)
- [Development Guide](docs/DEVELOPMENT.md)
- [Testing Documentation](docs/TESTING.md)

## Basic Usage

```bash
athena build deploy.ath              # Generate docker-compose.yml
athena build deploy.ath -o custom.yml   # Custom output file
athena validate deploy.ath           # Validate syntax only
athena info                          # Show DSL information
athena info --examples               # Show usage examples
athena info --directives             # Show all directives
```

## What Athena Adds Automatically

- Smart service detection (Database, Cache, WebApp, Proxy)
- Optimized health checks with service-specific intervals
- Production restart policies based on service type
- Modern container naming (`project-service`)
- Metadata labels for tracking and management
- Resource management with deploy sections
- Network isolation with custom networks
- Dockerfile integration when no image specified
- Dependency ordering with topological sort

## Acknowledgments

- **Pest** for powerful parsing capabilities
- **Clap** for excellent CLI framework
- **Docker Community** for container standards
- **Rust Community** for the amazing ecosystem

## License

This project is licensed under the MIT License see the [LICENSE](LICENSE) file for details.

---

Built with ❤️ using Rust | Make DevOps great again.
