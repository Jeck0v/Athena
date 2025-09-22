# Athena - Production-Ready DevOps Toolkit

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-0.1.0-green.svg)](Cargo.toml)

Athena is a powerful CLI toolkit for back-end developers and DevOps engineers that simplifies project creation and infrastructure setup. It provides two main capabilities:

1. **Docker Compose Generator**: Transform a COBOL-inspired DSL into production-ready Docker Compose configurations with minimal effort.
2. **Project Boilerplate Generator**: Quickly scaffold full-stack back-end projects using frameworks like FastAPI, Go (Gin/Echo/Fiber), and Flask, with modern best practices and Docker integration.


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

**[📖 Learn more about Enhanced Error Handling →](docs/ERROR_HANDLING.md)**

## Quick Start

### Installation
```bash
# Install from source
git clone https://github.com/your-org/athena.git
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

### Generate Full-Stack Project
```bash
# Create FastAPI + PostgreSQL project
athena init fastapi my-api --with-postgresql

# Create Go + MongoDB microservice
athena init go my-service --framework gin --with-mongodb
```

## Key Features

### 🚨 Enhanced Error Handling System (New!)
- **Line & Column Precision** => Exact error locations with visual context
- **Intelligent Suggestions** => Automatic recommendations for common fixes
- **Advanced Validation** => Port conflicts, service references, circular dependencies
- **Fail-Fast Processing** => Immediate feedback with no partial generation

### Intelligent Defaults 2025+
- No more `version` field modern Docker Compose spec compliance
- Auto-detects service types database, Cache, WebApp, Proxy patterns
- Smart restart policies `always` for databases, `unless-stopped` for apps
- Optimized health checks different intervals per service type
- Container naming follows modern conventions (`project-service`)

### Docker-First Approach
- Dockerfile by default => No image? Uses `build.dockerfile: Dockerfile`
- Intelligent networking => Auto-configured networks with proper isolation
- Production-ready => Security, resource limits, and health monitoring
- Standards compliant => Follows Docker Compose 2025 best practices

### Performance Optimized
- Topological sorting => Services ordered by dependencies automatically
- Iterative validation => Fast circular dependency detection
- Optimized parsing => **<1ms parse time, <2ms generation**
- Memory efficient => Pre-allocated structures for large compositions

### Full-Stack Boilerplates
- FastAPI + PostgreSQL/MongoDB => Production authentication, async drivers
- Go + Gin/Echo/Fiber => Clean architecture, proper middleware
- Flask + PostgreSQL =>  Modern Python web development
- Docker ready => Multi-stage builds, Nginx reverse proxy included

## Documentation

### Core Documentation
- [Enhanced Error Handling (**New**)](docs/ERROR_HANDLING.md) - Complete guide to Athena's advanced error system.
- [Installation Guide](docs/INSTALLATION.md)
- [Docker Compose Generator Usage](docs/DSL_REFERENCE.md)
- [Boilerplate Project Generator](docs/BOILERPLATE.md)
- [Examples](docs/EXAMPLES.md)

### Development
- [Architecture Overview](docs/ARCHITECTURE.md)
- [Development Guide](docs/DEVELOPMENT.md)
- [Testing Documentation](docs/TESTING.md)

## Basic Usage

### Docker Compose Generator
```bash
athena build deploy.ath              # Generate docker-compose.yml
athena build deploy.ath -o custom.yml   # Custom output file
athena validate deploy.ath           # Validate syntax only
```

### Boilerplate Generator
```bash
# FastAPI projects
athena init fastapi my-api --with-postgresql
athena init fastapi my-api --with-mongodb

# Go projects
athena init go my-service --framework gin
athena init go my-service --framework echo --with-postgresql

# Flask projects
athena init flask my-app --with-postgresql
```

## Complete Example: Modern Web Application

The `presentation.ath` file demonstrates **all Athena features** in a production-ready web application:

```athena
DEPLOYMENT-ID MODERN_WEB_APP
VERSION-ID 1.0.0

ENVIRONMENT SECTION
NETWORK-NAME modern_app_network

SERVICES SECTION

SERVICE nginx_proxy
IMAGE-ID "nginx:alpine"
PORT-MAPPING 80 TO 80
DEPENDS-ON backend
END SERVICE

SERVICE backend
PORT-MAPPING 3000 TO 3000
ENV-VARIABLE {{NODE_ENV}}
DEPENDS-ON mongodb
END SERVICE

SERVICE mongodb
IMAGE-ID "mongo:7.0"
PORT-MAPPING 27017 TO 27017
RESTART-POLICY always
END SERVICE
```

**Generated Configuration Highlights:**

- **Automatic Dockerfile Detection**: Backend service gets `build.dockerfile: Dockerfile`
- **Service Type Detection**: MongoDB → Database type with optimized settings
- **Custom Network**: All services connected to `modern_app_network`
- **Smart Labels**: Project tracking and metadata automatically added
- **Dependency Ordering**: Services sorted automatically (mongodb → backend → nginx)
- **Health Checks**: Type-specific intervals and commands

This **65-line configuration** generates a **220+ line** production-ready Docker Compose file with all best practices included!

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

## License

This project is licensed under the MIT License see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **Pest** for powerful parsing capabilities
- **Clap** for excellent CLI framework
- **Docker Community** for container standards
- **Rust Community** for the amazing ecosystem

---

Built with ❤️ using Rust | Production-ready DevOps made simple
