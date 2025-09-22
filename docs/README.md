# 📚 Athena Documentation

Welcome to the comprehensive documentation for Athena, the production-ready DevOps toolkit that simplifies Docker Compose generation and project scaffolding.

## 🚀 Quick Navigation

### Core Documentation
- [🚨 **Enhanced Error Handling**](ERROR_HANDLING.md) - Complete guide to Athena's advanced error system
- [📖 DSL Reference](DSL_REFERENCE.md) - Complete syntax reference for .ath files
- [🐳 Docker Compose Generator](DOCKER_COMPOSE.md) - Generate production-ready Docker configurations
- [🏗️ Project Boilerplates](BOILERPLATE.md) - FastAPI, Flask, and Go project templates
- [💡 Examples](EXAMPLES.md) - Real-world usage examples

### Development & Advanced
- [🏗️ Architecture Overview](ARCHITECTURE.md) - Internal architecture and design
- [🧪 Testing Documentation](TESTING.md) - Testing strategies and guidelines
- [⚙️ Development Guide](DEVELOPMENT.md) - Contributing and development setup

## ✨ What's New

### 🚨 Enhanced Error Handling System (Latest)

Athena now features a **revolutionary error handling system** with:

- **📍 Line & Column Precision**: Exact error locations with visual context
- **💡 Intelligent Suggestions**: Automatic recommendations for common fixes
- **🔍 Advanced Validation**: Port conflicts, service references, circular dependencies
- **⚡ Fail-Fast Processing**: Immediate feedback with no partial generation

**Example Enhanced Error:**
```
Error: Parse error at line 8, column 1: Missing 'END SERVICE' statement
   |
 8 | # Missing END SERVICE statement
   | ^ Error here

Suggestion: Each SERVICE block must be closed with 'END SERVICE'
```

**[📖 Read the complete Error Handling guide →](ERROR_HANDLING.md)**

### 🎯 Key Features

#### 🧠 Intelligent Defaults Engine
- **Service Type Detection**: Automatically detects databases, caches, web apps, proxies
- **Smart Restart Policies**: Service-specific restart strategies
- **Optimized Health Checks**: Type-aware health check intervals

#### ⚡ Performance Optimizations
- **Parse time**: <1ms for typical files
- **Generation time**: <2ms for 20+ service compositions
- **Topological Service Sorting**: Automatic dependency ordering
- **Memory Efficient**: ~2MB for large compositions

#### 🐳 Docker Compose 2025+ Features
- **Modern Specification**: No deprecated fields, latest best practices
- **Container Naming**: Follows kebab-case conventions
- **Enhanced Labels**: Comprehensive metadata for tracking
- **Dockerfile-First**: Automatic build configuration when no image specified

#### 🌐 Network & Security
- **Automatic Network Configuration**: Smart network naming and isolation
- **Metadata Labels**: Docker operation management labels
- **Production Restart Policies**: Service-specific restart strategies

## 🎯 Getting Started

### 1. Installation
```bash
git clone https://github.com/your-org/athena.git
cd athena
cargo install --path .
```

### 2. Basic Usage
```bash
# Generate Docker Compose from .ath file
athena build deploy.ath

# Validate syntax only
athena validate deploy.ath

# Generate FastAPI project
athena init fastapi my-api --with-postgresql
```

### 3. Your First .ath File
```athena
DEPLOYMENT-ID MY_FIRST_APP

SERVICES SECTION

SERVICE web
IMAGE-ID "nginx:alpine"
PORT-MAPPING 8080 TO 80
DEPENDS-ON api
END SERVICE

SERVICE api
IMAGE-ID "node:18-alpine"
PORT-MAPPING 3000 TO 3000
DEPENDS-ON database
END SERVICE

SERVICE database
IMAGE-ID "postgres:15"
PORT-MAPPING 5432 TO 5432
END SERVICE
```

## 🛠️ Troubleshooting

### Common Issues

1. **Parse Errors**: Check the [Error Handling guide](ERROR_HANDLING.md) for detailed examples
2. **Port Conflicts**: Athena will suggest alternative ports automatically
3. **Service References**: Make sure all DEPENDS-ON services exist
4. **Missing END SERVICE**: Each SERVICE block must be properly closed

### Getting Help

- 📖 Check the [Error Handling documentation](ERROR_HANDLING.md) for specific error solutions
- 💡 Look at [Examples](EXAMPLES.md) for reference implementations
- 🔧 Review the [DSL Reference](DSL_REFERENCE.md) for syntax details

## 🚀 What Makes Athena Special

### Developer Experience First
- **Clear Error Messages**: No more cryptic parsing errors
- **Intelligent Suggestions**: Learn the DSL through helpful error messages
- **Visual Error Context**: See exactly where issues occur
- **Fail-Fast Validation**: Catch errors before generation

### Production Ready
- **Modern Docker Standards**: 2025+ best practices built-in
- **Performance Optimized**: Sub-millisecond parsing and generation
- **Security Focused**: Safe defaults and proper isolation
- **Scalable Architecture**: Handles large microservice compositions

### Comprehensive Toolkit
- **Docker Compose Generation**: From simple to complex multi-service applications
- **Project Scaffolding**: FastAPI, Flask, Go with authentication and databases
- **Intelligent Defaults**: Service-type aware configuration
- **Advanced Validation**: Prevents common deployment issues

---

**Ready to get started?** 

📖 **[Begin with the Error Handling guide](ERROR_HANDLING.md)** to understand Athena's powerful error system, then explore the [DSL Reference](DSL_REFERENCE.md) for complete syntax documentation.

Built with ❤️ using Rust | Production-ready DevOps made simple