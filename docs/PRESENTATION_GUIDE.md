# Athena Presentation Files Guide

This directory contains two presentation configurations demonstrating Athena's capabilities:

## Files Overview

### `presentation.ath` - Production Version
**Use for:** Demonstrating production-ready security practices

**Features:**
- ✅ **Secure**: Only nginx exposed (ports 80/443)
- ✅ **Internal services**: No external ports for databases/APIs
- ✅ **Production-ready**: Docker Swarm compatible
- ✅ **Comments**: Comprehensive documentation

**Exposed Ports:**
- `80, 443` - nginx_reverse_proxy (only entry point)
- `9090` - monitoring (Prometheus UI)

### `light_presentation.ath` - Demo/Testing Version
**Use for:** Live demonstrations, API testing, development

**Features:**
- ✅ **Testing-friendly**: All services accessible
- ✅ **Development mode**: NODE_ENV=development
- ✅ **Direct access**: Can test APIs individually
- ✅ **Database tools**: Direct database connections

**Exposed Ports:**
- `80, 443` - nginx_reverse_proxy (web entry)
- `3000` - api_gateway (REST API)
- `3001` - product_service (Catalog API)
- `3002` - auth_service (Authentication API)
- `5432` - database (PostgreSQL)
- `6379` - cache (Redis)
- `9090` - monitoring (Prometheus)

##  Usage Examples

### Quick Demo
```bash
# Start demo environment
athena build light_presentation.ath
docker-compose up -d

# Test APIs directly
curl http://localhost:3000/health    # API Gateway
curl http://localhost:3001/health    # Product Service
curl http://localhost:3002/health    # Auth Service

# Access monitoring
open http://localhost:9090           # Prometheus

# Database access
psql -h localhost -p 5432 -U postgres
redis-cli -h localhost -p 6379
```

### Production Demo
```bash
# Show production security
athena build presentation.ath
docker-compose up -d

# Only nginx accessible externally
curl http://localhost              # ✅ Works
curl http://localhost:3000        # ❌ Blocked (secure)
```

## Presentation Tips

1. **Start with `light_presentation.ath`** to show functionality
2. **Switch to `presentation.ath`** to demonstrate security
3. **Compare generated YAML** to highlight Athena's intelligence
4. **Show comment features** and intelligent defaults

## Key Differences

| Aspect | light_presentation.ath | presentation.ath |
|--------|----------------------|------------------|
| **Security** | Demo-friendly | Production-ready |
| **Port Exposure** | All services | Nginx only |
| **Testing** | Direct API access | Proxy-only access |
| **Environment** | Development | Production |
| **Use Case** | Demos, Testing | Production deployment |

## Generated Files

- `light_presentation.ath` → `docker-compose.yml` (with all ports)
- `presentation.ath` → `docker-compose.yml` (secure, minimal ports)

The presentation files will evolve as the project progresses (We will soon be working on more advanced support for swarm).
Both demonstrate Athena's intelligent defaults, healthchecks, and Docker Swarm readiness!
