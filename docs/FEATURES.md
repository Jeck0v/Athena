#  Athena Features Documentation

This document details the advanced features and capabilities of Athena, especially the recent performance and intelligence improvements.

## Intelligent Defaults Engine

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

## Performance Optimizations

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

## Docker Compose 2025+ Features

### Modern Compose Specification Compliance

**Removed Deprecated Fields:**
```yaml
# Old format (deprecated)
version: '3.8'
services: ...

# New format (Athena generates)
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

### Dockerfile-First Approach with Build Arguments

When no `IMAGE-ID` is specified, Athena automatically configures Docker build with full build arguments support:

**DSL Input:**
```athena
SERVICE api
BUILD-ARGS NODE_ENV="production" API_VERSION="v2.0" DATABASE_POOL_SIZE="20"
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
    args:
      NODE_ENV: production
      API_VERSION: v2.0
      DATABASE_POOL_SIZE: "20"
  ports:
    - "8000:8000"
  environment:
    - API_KEY=${API_KEY}
  # ... rest of configuration with intelligent defaults
```

**Build Arguments Features:**
- **Type Safety**: All build args are properly quoted in YAML output
- **Environment Integration**: Build args work seamlessly with environment variables
- **Multi-Stage Support**: Perfect for multi-stage Dockerfile builds
- **Development/Production**: Easy switching between build configurations

## Enhanced Validation System

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

## Advanced Error Handling System (NEW)

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
 7 | PORT-MAPPING 8080 : 80
   |                   ^ Error here

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

## Metadata and Labels

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

## Network Optimization

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

## Resource Management (New)

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

## Architecture Examples

Athena provides comprehensive examples demonstrating various architectural patterns:

### Available Architecture Templates

**1. Microservices Architecture (`examples/microservices.ath`)**
- Complete e-commerce microservices stack
- API Gateway with multiple specialized services
- Individual databases per service
- Message queuing with RabbitMQ
- Search with Elasticsearch
- Monitoring with Prometheus/Grafana
- Build arguments for different environments

**2. Monolithic Architecture (`examples/monolithic.ath`)**
- Traditional single-application deployment
- Reverse proxy with SSL termination
- Background worker processes
- Shared database and caching
- Production-ready configuration

**3. Three-Tier Web Application (`examples/three-tier.ath`)**
- Separated presentation, business, and data layers
- Frontend with React build arguments
- API server with Spring Boot configuration
- Dedicated cache and storage layers
- Load balancing and SSL support

**4. Event-Driven Architecture (`examples/event-driven.ath`)**
- Apache Kafka event streaming platform
- Multiple event processors in different languages
- Real-time analytics with Elasticsearch
- Scalable message processing
- Event monitoring and management UI

**5. Full-Stack Web Application (`examples/fullstack-web.ath`)**
- Complete web application stack
- Frontend, API, WebSocket, and background workers
- File storage with MinIO S3-compatible service
- Full-text search capabilities
- Comprehensive monitoring and visualization

### Build Arguments in Examples

All examples demonstrate the new BUILD-ARGS feature:
```athena
SERVICE user_service
BUILD-ARGS NODE_ENV="production" API_VERSION="v2.0" DATABASE_POOL_SIZE="20"
# ... service configuration
END SERVICE
```

### Usage Examples

```bash
# Generate from any architecture example
athena build examples/microservices.ath
athena build examples/monolithic.ath -o my-stack.yml
athena build examples/event-driven.ath

# Validate architecture before generation
athena validate examples/fullstack-web.ath
```

## Docker Swarm Support (**NEW** 12/10/2025)

Athena now provides comprehensive Docker Swarm support with native DSL directives for production-ready cluster deployments.

### Replica Management

**Control service scaling with intelligent replica management:**

```athena
SERVICE api_gateway
IMAGE-ID python:3.11-slim
REPLICAS 3                          # Scale to 3 instances
UPDATE-CONFIG PARALLELISM 1 DELAY 10s FAILURE-ACTION ROLLBACK
END SERVICE
```

**Generated Swarm Configuration:**
```yaml
api_gateway:
  image: python:3.11-slim
  deploy:
    replicas: 3
    update_config:
      parallelism: 1        # Update one replica at a time
      delay: 10s            # Wait 10s between updates
      failure_action: rollback  # Rollback on failure
```

### Overlay Network Support

**Production-ready overlay networks for multi-host communication:**

```athena
ENVIRONMENT SECTION
NETWORK-NAME swarm_overlay DRIVER OVERLAY ATTACHABLE TRUE ENCRYPTED TRUE
```

**Generated Network Configuration:**
```yaml
networks:
  swarm_overlay:
    driver: overlay         # Multi-host networking
    attachable: true        # Allow container attachment
    encrypted: true         # Encrypt network traffic
```

### Update Configuration Options

**Comprehensive update control for zero-downtime deployments:**

| Directive | Description | Example |
|-----------|-------------|---------|
| `PARALLELISM` | Replicas updated simultaneously | `PARALLELISM 2` |
| `DELAY` | Pause between update batches | `DELAY 30s` |
| `FAILURE-ACTION` | Action on update failure | `FAILURE-ACTION ROLLBACK` |
| `MONITOR` | Duration to monitor for failures | `MONITOR 60s` |
| `MAX-FAILURE-RATIO` | Maximum allowed failure ratio | `MAX-FAILURE-RATIO 0.3` |

```athena
SERVICE microservice
IMAGE-ID node:18-alpine
REPLICAS 5
UPDATE-CONFIG PARALLELISM 2 DELAY 15s FAILURE-ACTION PAUSE MONITOR 30s MAX-FAILURE-RATIO 0.2
END SERVICE
```

### Swarm-Specific Labels

**Enhanced labeling for service discovery and management:**

```athena
SERVICE web_frontend
IMAGE-ID nginx:alpine
REPLICAS 2
SWARM-LABELS environment="production" tier="frontend" version="v2.1"
END SERVICE
```

**Generated Labels:**
```yaml
web_frontend:
  deploy:
    replicas: 2
    labels:
      environment: production
      tier: frontend
      version: v2.1
```

### Complete Swarm Stack Example

**Production-ready microservices with Swarm orchestration:**

```athena
DEPLOYMENT-ID MICROSERVICES_SWARM
VERSION-ID 2.0.0

ENVIRONMENT SECTION
NETWORK-NAME overlay_network DRIVER OVERLAY ATTACHABLE TRUE ENCRYPTED TRUE

SERVICES SECTION

SERVICE api_gateway
BUILD-ARGS NODE_ENV="production" API_VERSION="v2.0"
REPLICAS 3
UPDATE-CONFIG PARALLELISM 1 DELAY 10s FAILURE-ACTION ROLLBACK
SWARM-LABELS tier="api" environment="production"
DEPENDS-ON user_service
DEPENDS-ON order_service
END SERVICE

SERVICE user_service
IMAGE-ID python:3.11-slim
REPLICAS 2
UPDATE-CONFIG PARALLELISM 1 DELAY 15s
SWARM-LABELS tier="backend" service="users"
DEPENDS-ON database
END SERVICE

SERVICE order_service
IMAGE-ID java:17-jdk-slim
REPLICAS 3
UPDATE-CONFIG PARALLELISM 2 DELAY 20s FAILURE-ACTION PAUSE
SWARM-LABELS tier="backend" service="orders"
DEPENDS-ON database
END SERVICE

SERVICE database
IMAGE-ID postgres:15
REPLICAS 1
SWARM-LABELS tier="data" critical="true"
RESOURCE-LIMITS CPU "2.0" MEMORY "2048M"
END SERVICE
```

### Deployment Commands

**Deploy your Swarm stack with intelligent configurations:**

```bash
# Generate Swarm-compatible compose file
athena build microservices.ath -o swarm-stack.yml

# Deploy to Docker Swarm cluster
docker stack deploy -c swarm-stack.yml myapp

# Scale services dynamically
docker service scale myapp_api_gateway=5

# Monitor service status
docker service ls
docker service ps myapp_api_gateway
```

### Mixed Mode Support

**Seamlessly combine Docker Compose and Swarm features:**

```athena
SERVICE development_service
IMAGE-ID alpine:latest
PORT-MAPPING 8080 TO 80    # Compose-style port mapping
END SERVICE

SERVICE production_service
IMAGE-ID nginx:alpine
REPLICAS 3                 # Swarm-specific scaling
UPDATE-CONFIG PARALLELISM 1 DELAY 10s
SWARM-LABELS tier="production"
END SERVICE
```

### Network Driver Options

| Driver | Use Case | Generated Config |
|--------|----------|-----------------|
| `BRIDGE` | Single-host development | `driver: bridge` |
| `OVERLAY` | Multi-host production | `driver: overlay` |
| `HOST` | Direct host networking | `driver: host` |

### Failure Actions

| Action | Behavior | When to Use |
|--------|----------|-------------|
| `CONTINUE` | Continue despite failures | Non-critical updates |
| `PAUSE` | Stop updates on failure | Manual intervention needed |
| `ROLLBACK` | Revert to previous version | Automatic recovery |

## Future Enhancements

### Planned Features

**Monitoring Integration:**
- Prometheus metrics endpoints
- Grafana dashboard generation
- Log aggregation configuration

**Security Enhancements:**
- Docker secrets integration
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

This documentation covers the current advanced features and planned enhancements for Athena's intelligent generation system.
