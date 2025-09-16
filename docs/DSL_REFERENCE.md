# DSL Reference

## File Structure
```cobol
DEPLOYMENT-ID project_name    # Required: Project identifier
VERSION-ID version           # Optional: Project version (just in the .ath)

ENVIRONMENT SECTION          # Optional: Environment configuration
NETWORK-NAME custom_network  # Optional: Custom network name

SERVICES SECTION             # Required: Service definitions

SERVICE service_name         # Service block start
# Service directives here
END SERVICE                  # Service block end
```

## Service Directives

| Directive | Description | Example |
|-----------|-------------|---------|
| `IMAGE-ID` | Docker image (if no Dockerfile) | `IMAGE-ID postgres:15` |
| `PORT-MAPPING` | Port forwarding | `PORT-MAPPING 8000 TO 8000` |
| `ENV-VARIABLE` | Environment variable | `ENV-VARIABLE {{DATABASE_URL}}` |
| `COMMAND` | Container command | `COMMAND "npm start"` |
| `DEPENDS-ON` | Service dependency | `DEPENDS-ON database` |
| `HEALTH-CHECK` | Health check command | `HEALTH-CHECK "curl -f http://localhost/health"` |
| `RESTART-POLICY` | Restart behavior | `RESTART-POLICY unless-stopped` |
| `RESOURCE-LIMITS` | CPU/Memory limits | `RESOURCE-LIMITS CPU "0.5" MEMORY "512M"` |
| `VOLUME-MAPPING` | Volume mount | `VOLUME-MAPPING "./data" TO "/app/data"` |

## Smart Defaults by Service Type

| Service Type | Auto-Detection | Restart Policy | Health Check Interval |
|--------------|----------------|----------------|---------------------|
| **Database** | `postgres`, `mysql`, `mongodb` | `always` | `10s` |
| **Cache** | `redis`, `memcached` | `always` | `15s` |
| **Proxy** | `nginx`, `apache`, `traefik` | `always` | `20s` |
| **WebApp** | `node`, `python`, `java` | `unless-stopped` | `30s` |
| **Generic** | Other images or Dockerfile | `unless-stopped` | `30s` |
