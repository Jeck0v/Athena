# Boilerplate Project Generator

Generate production-ready full-stack applications with modern best practices.

## FastAPI Projects
```bash
# FastAPI + PostgreSQL
athena init fastapi my-api --with-postgresql

# FastAPI + MongoDB (default)
athena init fastapi my-api --with-mongodb

# Without Docker files
athena init fastapi my-api --no-docker
```

**Generated FastAPI Structure:**
```
my-api/
├── app/
│   ├── __init__.py
│   ├── main.py              # FastAPI application
│   ├── core/
│   │   ├── config.py        # Settings management
│   │   ├── security.py      # JWT + password hashing
│   │   └── database.py      # Async database config
│   ├── api/
│   │   ├── v1/             # API versioning
│   │   │   ├── auth.py     # Authentication endpoints
│   │   │   └── users.py    # User management
│   ├── models/             # Database models
│   ├── schemas/            # Pydantic models
│   └── services/           # Business logic
├── tests/                  # Comprehensive test suite
├── nginx/                  # Reverse proxy config
├── logs/                   # Application logs
├── requirements.txt        # Python dependencies
├── Dockerfile             # Production Docker build
├── docker-compose.yml     # Full stack deployment
└── .env.example          # Environment template
```

## Go Projects
```bash
# Go + Gin (default)
athena init go my-service

# Go + Echo framework
athena init go my-service --framework echo --with-postgresql

# Go + Fiber framework
athena init go my-service --framework fiber --with-mongodb
```

**Generated Go Structure:**
```
my-service/
├── cmd/
│   └── server/
│       └── main.go         # Application entrypoint
├── internal/
│   ├── config/            # Configuration management
│   ├── handler/           # HTTP handlers
│   ├── middleware/        # Custom middleware
│   ├── model/            # Data models
│   ├── repository/       # Data access layer
│   └── service/          # Business logic
├── pkg/                   # Public packages
├── tests/                 # Test suite
├── scripts/               # Build & deployment scripts
├── Dockerfile            # Production build
├── docker-compose.yml    # Development environment
├── go.mod               # Go modules
└── .env.example         # Environment template
```

## Flask Projects
```bash
# Flask + PostgreSQL
athena init flask my-app --with-postgresql

# Flask + MongoDB
athena init flask my-app --with-mongodb
```