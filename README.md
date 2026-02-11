# Athena -- Declarative DevOps Toolkit

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-0.1.0-green.svg)](Cargo.toml)

Athena is an open source CLI tool for back-end developers and DevOps engineers that turns a COBOL-inspired DSL into clean, consistent Docker Compose configurations.

The goal is simple: reduce YAML noise, enforce sane defaults, and make infrastructure definitions easier to read, validate and evolve.

Athena focuses on correctness, clarity and automation first. Production hardening is intentionally progressive and opinionated, not magic.

## Why Athena DSL?

Writing infrastructure directly in YAML often leads to repetitive, fragile and hard to review configurations.

Athena introduces a DSL designed to be explicit, readable and tooling-friendly, while remaining close to Docker concepts.

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

You write:

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

Athena expands this into a complete Docker Compose file with validated structure and consistent defaults.

## Quick Start

```bash
git clone https://github.com/Jeck0v/Athena
cd athena
cargo install --path .

athena build deploy.ath
```

## Usage

```bash
athena build deploy.ath                 # Generate docker-compose.yml
athena build deploy.ath -o custom.yml   # Custom output file
athena validate deploy.ath              # Validate syntax only
athena info                             # Show DSL information
athena info --examples                  # Show usage examples
athena info --directives                # Show all directives
```

If no file is specified, Athena looks for a `.ath` file in the current directory.

## What Athena Handles

- **Service type detection** - recognizes Database, Cache, WebApp and Proxy patterns from the image name
- **Healthchecks** - generates appropriate healthchecks per service type with tuned intervals
- **Restart policies** - `always` for databases and caches, `unless-stopped` for applications
- **Network isolation** - automatic bridge network shared across services
- **Dependency ordering** - services are sorted topologically in the output
- **Dockerfile fallback** - no image specified means Athena looks for a Dockerfile
- **Resource limits** - CPU and memory constraints via `RESOURCE-LIMITS`
- **Metadata labels** - every service is tagged with project, type and generation date
- **Docker Swarm** - replicas, update config and overlay networks when needed

Everything Athena adds is visible in the generated output. No hidden behavior.

## Error Handling

Athena provides precise parser errors with line and column information, visual context and actionable suggestions.

```
Error: Parse error at line 8, column 1: Missing 'END SERVICE' statement
   |
 8 | # Missing END SERVICE
   | ^ Error here

Suggestion: Each SERVICE block must be closed with 'END SERVICE'
```

Validation also catches port conflicts, invalid service references and circular dependencies before any file is generated.

See [Error Handling documentation](docs/ERROR_HANDLING.md) for the full reference.

## DSL Reference

| Directive | Example |
|---|---|
| `DEPLOYMENT-ID` | `DEPLOYMENT-ID my_project` |
| `VERSION-ID` | `VERSION-ID 1.0.0` |
| `NETWORK-NAME` | `NETWORK-NAME app_network` |
| `IMAGE-ID` | `IMAGE-ID postgres:15` |
| `PORT-MAPPING` | `PORT-MAPPING 8080 TO 80` |
| `ENV-VARIABLE` | `ENV-VARIABLE {{DATABASE_URL}}` |
| `COMMAND` | `COMMAND "npm start"` |
| `VOLUME-MAPPING` | `VOLUME-MAPPING "./data" TO "/app/data"` |
| `DEPENDS-ON` | `DEPENDS-ON database` |
| `HEALTH-CHECK` | `HEALTH-CHECK "curl -f http://localhost/health"` |
| `RESTART-POLICY` | `RESTART-POLICY always` |
| `RESOURCE-LIMITS` | `RESOURCE-LIMITS CPU "0.5" MEMORY "512M"` |
| `REPLICAS` | `REPLICAS 3` |

Full syntax documentation: [DSL Reference](docs/DSL_REFERENCE.md)

## Documentation

- [DSL Reference](docs/DSL_REFERENCE.md) - complete syntax and directives
- [Error Handling](docs/ERROR_HANDLING.md) - error system and validation rules
- [Architecture](docs/ARCHITECTURE.md) - project structure and design
- [Features](docs/FEATURES.md) - detailed feature documentation
- [Examples](docs/EXAMPLES.md) - example configurations
- [Testing](docs/TESTING.md) - test suite and conventions
- [Development](docs/DEVELOPMENT.md) - contributing and development workflow
- [Installation](docs/INSTALLATION.md) - installation options

## Design Principles

Athena is opinionated but transparent. Everything it adds is explicit in the generated output.

Defaults are meant to be reasonable starting points, not final production guarantees. You are expected to review and adapt the output to your actual deployment constraints.

Athena does not hide Docker. It reduces friction around it.

## License

MIT License. See [LICENSE](LICENSE) for details.

---

Built with ❤️ using Rust | Make DevOps great again.
