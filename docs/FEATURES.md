# 🚀 Athena Features Documentation

This document details the advanced features and capabilities of Athena, especially the recent performance and intelligence improvements.

## 🧠 Intelligent Defaults Engine

### Service Type Detection

Athena automatically detects service types based on image patterns and applies appropriate defaults:

| Pattern | Type | Restart Policy | Health Check Interval | Special Features |
|---------|------|----------------|---------------------|------------------|
| `postgres`, `mysql`, `mongodb`, `mariadb` | Database | `always` | `10s` | Longer startup period (60s) |
| `redis`, `memcached` | Cache | `always` | `15s` | Fast startup (20s) |
| `nginx`, `apache`, `traefik`, `haproxy` | Proxy | `always` | `20s` | Standard web timeouts |
| `node`, `python`, `php`, `ruby`, `java`, `go` | WebApp | `unless-stopped` | `30s` | Extended startup (40s) |
| Custom Dockerfile or unknown | Generic | `unless-stopped` | `30s` | Balanced defaults |

### Example: Database Service
```yaml
# Input: IMAGE-ID postgres:15
# Output: Auto-detected as Database type
database:
  image: postgres:15
  restart: always           # ← Smart default for databases
  networks:
    - project_network       # ← Auto-configured
  labels:
    athena.type: database   # ← Auto-detected type
  pull_policy: missing      # ← Efficient default
```

### Example: WebApp Service (Dockerfile)
```yaml
# Input: No IMAGE-ID specified  
# Output: Auto-configured for Dockerfile
webapp:
  build:                    # ← Dockerfile detected automatically
    context: .
    dockerfile: Dockerfile
  restart: unless-stopped   # ← Smart default for apps
  labels:
    athena.type: generic    # ← Generic type for Dockerfile services
```

## ⚡ Performance Optimizations

### Topological Service Sorting

Services are automatically sorted by dependencies for optimal startup order:

**Input Order:**
```cobol
SERVICE frontend
DEPENDS-ON backend
END SERVICE

SERVICE backend  
DEPENDS-ON database
END SERVICE

SERVICE database
IMAGE-ID postgres:15
END SERVICE
```

**Output Order:**
```yaml
services:
  database:    # ← Sorted first (no dependencies)
    # ...
  backend:     # ← Sorted second (depends on database)
    # ...  
  frontend:    # ← Sorted last (depends on backend)
    # ...
```

### Optimized Parsing Pipeline

1. **Pre-validation** (fail-fast for common errors)
2. **Grammar parsing** using Pest (<1ms)
3. **AST optimization** (deduplication, sorting)
4. **Intelligent defaults** application
5. **Enhanced validation** with circular dependency detection
6. **YAML generation** with pre-allocated memory

### Performance Metrics
- **Parse time**: <1ms for typical files
- **Generation time**: <2ms for 20+ service compositions  
- **Validation time**: <5ms with full circular dependency detection
- **Memory usage**: ~2MB for large compositions (vs ~8MB before optimization)

## 🐳 Docker Compose 2025+ Features

### Modern Compose Specification Compliance

**Removed Deprecated Fields:**
```yaml
# ❌ Old format (deprecated)
version: '3.8'
services: ...

# ✅ New format (Athena generates)
services: ...
name: PROJECT_NAME
```

**Enhanced Service Configuration:**
```yaml
services:
  app:
    # Modern container naming
    container_name: project-name-app     # kebab-case convention
    
    # Optimized pull policy
    pull_policy: missing                 # Efficient default
    
    # Enhanced health checks  
    healthcheck:
      test: [CMD-SHELL, "curl -f http://localhost/health"]
      interval: 30s                     # Service-type optimized
      timeout: 10s
      retries: 3
      start_period: 40s                 # Extended for complex apps
    
    # Production restart policies
    deploy:
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
        window: 120s
        
    # Metadata labels for tracking
    labels:
      athena.project: PROJECT_NAME
      athena.service: app
      athena.type: webapp               # Auto-detected
      athena.generated: 2025-09-13     # Generation timestamp
```

### Dockerfile-First Approach

When no `IMAGE-ID` is specified, Athena automatically configures Docker build:

**DSL Input:**
```cobol
SERVICE api
PORT-MAPPING 8000 TO 8000
ENV-VARIABLE {{API_KEY}}
END SERVICE
```

**Generated Output:**
```yaml
api:
  build:
    context: .              # Current directory
    dockerfile: Dockerfile  # Standard Dockerfile name
    # args: {}             # Future: build arguments support
  # ... rest of configuration
```

**Advanced Build Configuration (Future):**
```yaml
# Future enhancement - build arguments
api:
  build:
    context: .
    dockerfile: Dockerfile
    args:
      BUILD_ENV: production
      NODE_VERSION: 18
```

## 🔍 Enhanced Validation System

### Circular Dependency Detection

**Optimized Algorithm:**
- Iterative DFS (no stack overflow risk)
- O(V + E) complexity for dependency graph
- Early termination on cycle detection
- Detailed error messages with service names

**Example Error:**
```
❌ Error: Circular dependency detected involving service 'api'
💡 Check the DEPENDS-ON declarations in your .ath file.
```

### Port Conflict Detection
```yaml
# Athena detects and warns about port conflicts
services:
  app1:
    ports: ["8080:80"]
  app2:
    ports: ["8080:8000"]  # ← Conflict detected!
```

### Enhanced Error Messages
```
❌ Error: Service 'backend' depends on 'database' which doesn't exist
💡 Available services: api, cache, frontend
```

## 🚨 Advanced Error Handling System

### Line & Column Precision with Visual Context

Athena provides **exact error locations** with line and column numbers, plus visual context showing the problematic code:

**Parse Error Example:**
```
Error: Parse error at line 8, column 1: Missing 'END SERVICE' statement
   |
 8 | # Missing END SERVICE statement  
   | ^ Error here

Suggestion: Each SERVICE block must be closed with 'END SERVICE'
```

**Port Mapping Error Example:**
```
Error: Parse error at line 7, column 20: Invalid port mapping format
   |
 7 | PORT-MAPPING 8080 INVALID_FORMAT 80
   |                    ^ Error here

Suggestion: Use PORT-MAPPING <host_port> TO <container_port> format, e.g., PORT-MAPPING 8080 TO 80
```

### Intelligent Error Categories

**1. Syntax Errors (Parse Errors):**
- Missing keywords (END SERVICE, DEPLOYMENT-ID)
- Invalid formats (port mappings, environment variables)  
- Malformed structures (unclosed blocks, missing sections)

**2. Validation Errors:**
- Port conflicts between services
- Invalid service references in dependencies
- Circular dependencies in service chains
- Missing required configurations

**3. Configuration Errors:**
- Invalid restart policies
- Malformed resource limits
- Incorrect volume mappings

### Enhanced Port Conflict Detection

**Input with Conflicts:**
```athena
SERVICE frontend
PORT-MAPPING 8080 TO 80
END SERVICE

SERVICE backend  
PORT-MAPPING 8080 TO 3000  # Conflict!
END SERVICE

SERVICE api
PORT-MAPPING 8080 TO 8000  # Another conflict!
END SERVICE
```

**Enhanced Error Output:**
```
Error: Validation error: Port conflict detected! Host port 8080 is used by multiple services: frontend, backend, api
Affected services: frontend, backend, api

Suggestion: Use different host ports, e.g., 8080, 8081, 8082
```

### Smart Service Reference Validation

**Input with Invalid Reference:**
```athena
SERVICE frontend
DEPENDS-ON nonexistent_backend  # Invalid reference!
END SERVICE

SERVICE database
IMAGE-ID "postgres:15"
END SERVICE
```

**Enhanced Error Output:**
```
Error: Validation error: Service 'frontend' depends on 'nonexistent_backend' which doesn't exist
Affected services: frontend, nonexistent_backend

Suggestion: Available services: database, frontend. Check the service name in your DEPENDS-ON declaration
```

### Fail-Fast Error Processing

- **Immediate validation**: Errors stop processing before attempting generation
- **No partial generation**: Docker Compose files are only created when validation passes completely
- **Clear error flow**: Users see exactly what needs to be fixed before proceeding

## 🏷️ Metadata and Labels

### Automatic Label Generation

Every service gets comprehensive metadata labels:

```yaml
labels:
  athena.project: PROJECT_NAME       # Project identification
  athena.service: service_name       # Service identification  
  athena.type: webapp               # Auto-detected type
  athena.generated: 2025-09-13      # Generation date
```

### Label-Based Management

These labels enable advanced Docker operations:

```bash
# List all Athena-generated services
docker ps --filter "label=athena.project"

# Stop all database services
docker ps --filter "label=athena.type=database" -q | xargs docker stop

# Clean up project containers
docker ps --filter "label=athena.project=MY_PROJECT" -q | xargs docker rm
```

## 🌐 Network Optimization

### Automatic Network Configuration

**Smart Network Naming:**
```yaml
# Project: ECOMMERCE_STACK
# Custom network: ecommerce_net (from NETWORK-NAME)
# Auto-generated: ecommerce_stack_network (default)

networks:
  ecommerce_net:           # Custom or auto-generated
    driver: bridge         # Optimized default
    # Future: custom IPAM configuration
```

### Service Network Assignment

All services automatically join the project network:

```yaml
services:
  api:
    networks:
      - ecommerce_net      # ← Automatic assignment
  database:
    networks:  
      - ecommerce_net      # ← Consistent across all services
```

### Network Isolation Benefits

- **Security**: Services isolated from other Docker networks
- **DNS**: Services can reference each other by name
- **Performance**: Optimized bridge networking
- **Scalability**: Easy service addition without configuration

## 🔧 Resource Management

### Intelligent Resource Defaults

**CPU Limits by Service Type:**
- **Database**: 1.0 CPU (high I/O workloads)
- **Cache**: 0.5 CPU (memory-focused)
- **WebApp**: 0.5 CPU (balanced workload)
- **Proxy**: 0.2 CPU (lightweight)

**Memory Limits by Service Type:**
- **Database**: 1GB (buffer pools, caching)
- **Cache**: 512MB (in-memory data)
- **WebApp**: 512MB (application runtime)
- **Proxy**: 256MB (minimal requirements)

### Production Deploy Configuration

```yaml
deploy:
  resources:
    limits:
      cpus: '1.0'
      memory: 1024M
    reservations:          # Future: resource reservations
      cpus: '0.25'
      memory: 256M
  restart_policy:
    condition: on-failure  # Production-grade restart
    delay: 5s             # Backoff on failures
    max_attempts: 3       # Prevent infinite restart loops
    window: 120s          # Restart window
```

## 📊 Future Enhancements

### Planned Features

**Enhanced Build Support:**
- Build arguments from environment
- Multi-stage Dockerfile optimization  
- Custom build contexts

**Advanced Networking:**
- Service mesh integration (Istio/Consul)
- Custom IPAM configuration
- External network attachments

**Monitoring Integration:**
- Prometheus metrics endpoints
- Grafana dashboard generation
- Log aggregation configuration

**Security Enhancements:**
- Secret management integration
- Security scanning in build process
- Non-root user defaults

**Cloud Integration:**
- Kubernetes manifest generation
- Cloud-specific optimizations (AWS ECS, GCP Cloud Run)
- Infrastructure as Code integration

### Performance Goals

**Target Metrics:**
- Parse time: <0.5ms (50% improvement)
- Generation time: <1ms (50% improvement)  
- Memory usage: <1MB (50% reduction)
- File size: Support 100+ service compositions

**Scalability:**
- Support for microservice architectures (50+ services)
- Template caching for repeated generations
- Parallel validation for large compositions

---

This documentation covers the current advanced features and planned enhancements for Athena's intelligent Docker Compose generation system.