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
# Flask + PostgreSQL (default)
athena init flask my-app

# Flask + MySQL
athena init flask my-app --with-mysql

# Without Docker files
athena init flask my-app --no-docker
```

**Generated Flask Structure:**
```
my-app/
├── app/
│   ├── __init__.py            # Flask application factory
│   ├── core/
│   │   ├── config.py          # Configuration management
│   │   ├── extensions.py      # Flask extensions
│   │   └── logging.py         # Structured logging
│   ├── api/
│   │   ├── health.py          # Health check endpoints
│   │   └── v1/               # API versioning
│   │       ├── auth.py       # JWT authentication
│   │       └── users.py      # User management
│   ├── models/               # SQLAlchemy models
│   ├── schemas/              # Marshmallow schemas
│   └── services/             # Business logic layer
├── tests/                    # Comprehensive test suite
├── nginx/                    # Reverse proxy config
├── requirements.txt          # Python dependencies
├── Dockerfile               # Multi-stage production build
├── docker-compose.yml       # Full stack deployment
└── .env.example            # Environment template
```

## Laravel Projects (Clean Architecture)
```bash
# Laravel + PostgreSQL (default)
athena init laravel my-project

# Laravel + MySQL
athena init laravel my-project --with-mysql

# Without Docker files
athena init laravel my-project --no-docker
```

**Generated Laravel Structure:**
```
my-project/
├── app/
│   ├── Domain/                    # Domain layer (Clean Architecture)
│   │   └── User/
│   │       ├── Entities/          # Domain entities
│   │       │   └── User.php       # User entity with business logic
│   │       ├── Repositories/      # Repository interfaces
│   │       └── Services/          # Domain services
│   ├── Application/               # Application layer
│   │   └── User/
│   │       ├── UseCases/          # Use cases (business logic)
│   │       ├── DTOs/              # Data Transfer Objects
│   │       └── Services/          # Application services
│   └── Infrastructure/            # Infrastructure layer
│       ├── Http/
│       │   ├── Controllers/       # API controllers
│       │   └── Middleware/        # Custom middleware
│       ├── Persistence/           # Data persistence
│       │   ├── Eloquent/          # Eloquent models
│       │   └── Repositories/      # Repository implementations
│       └── Providers/             # Service providers
├── config/                        # Laravel configuration
├── database/
│   ├── migrations/               # Database migrations
│   └── seeders/                  # Data seeders
├── tests/                        # Feature & Unit tests
├── docker/                       # Docker configurations
├── nginx/                        # Nginx configuration
├── composer.json                 # PHP dependencies (Laravel 11, PHP 8.2)
├── Dockerfile                    # Multi-stage production build
├── docker-compose.yml            # Full stack deployment
└── .env.example                  # Environment template
```

## Symfony Projects (Hexagonal Architecture)
```bash
# Symfony + PostgreSQL (default)
athena init symfony my-api

# Symfony + MySQL
athena init symfony my-api --with-mysql

# Without Docker files
athena init symfony my-api --no-docker
```

**Generated Symfony Structure:**
```
my-api/
├── src/
│   ├── Domain/                    # Domain layer (Hexagonal Architecture)
│   │   └── User/
│   │       ├── Entities/          # Domain entities
│   │       │   └── User.php       # Pure domain entity
│   │       ├── ValueObjects/      # Value objects
│   │       │   ├── UserId.php     # User ID value object
│   │       │   ├── Email.php      # Email value object
│   │       │   ├── UserName.php   # User name value object
│   │       │   └── HashedPassword.php
│   │       └── Repositories/      # Repository interfaces
│   │           └── UserRepositoryInterface.php
│   ├── Application/               # Application layer
│   │   └── User/
│   │       ├── Commands/          # CQRS Commands
│   │       │   ├── CreateUserCommand.php
│   │       │   └── LoginCommand.php
│   │       ├── Queries/           # CQRS Queries
│   │       │   └── GetUserQuery.php
│   │       ├── Handlers/          # Command/Query handlers
│   │       │   ├── UserHandler.php
│   │       │   └── AuthHandler.php
│   │       └── Services/          # Application services
│   │           ├── UserService.php
│   │           └── AuthService.php
│   └── Infrastructure/            # Infrastructure layer
│       ├── Http/
│       │   └── Controllers/       # API controllers
│       │       └── UserController.php
│       └── Persistence/
│           └── Doctrine/
│               ├── Entities/      # Doctrine entities
│               │   └── User.php   # Infrastructure User entity
│               └── Repositories/  # Repository implementations
│                   └── DoctrineUserRepository.php
├── config/                        # Symfony configuration
├── migrations/                    # Doctrine migrations
├── tests/                        # Functional & Unit tests
├── docker/                       # Docker configurations
├── nginx/                        # Nginx configuration
├── composer.json                 # PHP dependencies (Symfony 7, PHP 8.2)
├── Dockerfile                    # Multi-stage production build
├── docker-compose.yml            # Full stack deployment
└── .env.example                  # Environment template
```

## Features & Best Practices 2025

### **Architecture Patterns**
- **Laravel**: Clean Architecture with Domain/Application/Infrastructure layers
- **Symfony**: Hexagonal Architecture with CQRS pattern
- **FastAPI**: Async-first architecture with dependency injection
- **Flask**: Layered architecture with factory pattern
- **Go**: Clean architecture with interfaces and dependency injection

### **Security & Authentication**
- **JWT Authentication** with refresh tokens
- **Password hashing** with modern algorithms (bcrypt/argon2)
- **CORS configuration** for cross-origin requests
- **Input validation** and sanitization
- **Security headers** in Nginx configuration
- **Environment-based secrets** management

### **Modern Language Features**
- **PHP 8.2+**: Strict types, readonly properties, attributes
- **Python 3.12+**: Type hints, async/await, dataclasses
- **Go 1.22+**: Generics, structured logging with slog
- **Dependency injection** and inversion of control
- **Value objects** and domain-driven design

### **Production-Ready Infrastructure**
- **Multi-stage Dockerfiles** for optimized builds
- **Nginx reverse proxy** with caching and compression
- **Health checks** and monitoring endpoints
- **Structured logging** with correlation IDs
- **Database migrations** and seeding
- **Redis caching** integration

### **Testing & Quality**
- **Comprehensive test suites** (unit, integration, functional)
- **PHPUnit 10** / **pytest** / **testify** frameworks
- **Code quality tools**: PHPStan, mypy, golangci-lint
- **Code formatting**: PHP-CS-Fixer, black, gofmt
- **Test coverage** reporting
- **CI/CD ready** configurations

### **Development Experience**
- **Hot reload** in development environments
- **Environment-based configuration** (.env files)
- **Database GUI tools** (Adminer/phpMyAdmin)
- **API documentation** with OpenAPI/Swagger
- **Pre-commit hooks** for code quality
- **Development scripts** and automation

## Quick Start Example

```bash
# Create a modern Laravel API
athena init laravel my-laravel-api
cd my-laravel-api
cp .env.example .env

# Start with Docker
docker-compose up --build

# Install dependencies and migrate
docker-compose exec app composer install
docker-compose exec app php artisan migrate

# Test the API
curl http://localhost/api/health
```

```bash
# Create a Symfony hexagonal API
athena init symfony my-symfony-api --with-mysql
cd my-symfony-api
cp .env.example .env

# Start with Docker
docker-compose up --build

# Install dependencies and migrate
docker-compose exec app composer install
docker-compose exec app php bin/console doctrine:migrations:migrate

# Test the API
curl http://localhost/api/health
```

## PHP Vanilla Projects (Clean Architecture)
```bash
# PHP Vanilla + PostgreSQL (default)
athena init vanilla my-api

# PHP Vanilla + MySQL
athena init vanilla my-api --with-mysql

# Without Docker files
athena init vanilla my-api --no-docker
```

**Generated PHP Vanilla Structure:**
```
my-api/
├── src/
│   ├── Domain/                       # Domain layer (Clean Architecture)
│   │   └── User/
│   │       ├── Entity/               # Domain entities
│   │       │   └── User.php          # Pure domain entity with business logic
│   │       ├── Repository/           # Repository interfaces
│   │       │   └── UserRepositoryInterface.php
│   │       ├── Service/              # Domain services
│   │       │   └── UserService.php   # User business logic
│   │       └── ValueObject/          # Value objects
│   │           ├── UserId.php        # UUID-based user ID
│   │           └── Email.php         # Email validation value object
│   ├── Application/                  # Application layer
│   │   ├── User/
│   │   │   ├── Command/              # Command objects
│   │   │   │   └── CreateUserCommand.php
│   │   │   └── Handler/              # Command handlers
│   │   │       └── CreateUserHandler.php
│   │   └── Auth/
│   │       ├── Command/              # Authentication commands
│   │       │   └── LoginCommand.php
│   │       └── Handler/              # Authentication handlers
│   │           └── LoginHandler.php
│   └── Infrastructure/               # Infrastructure layer
│       ├── Http/
│       │   ├── Router.php            # Custom routing system
│       │   ├── Request.php           # HTTP request abstraction
│       │   ├── Response.php          # HTTP response abstraction
│       │   ├── Controller/
│       │   │   └── Api/V1/           # Versioned API controllers
│       │   │       ├── AuthController.php    # JWT authentication
│       │   │       └── UserController.php    # User management
│       │   └── Middleware/           # HTTP middleware
│       │       ├── AuthMiddleware.php        # JWT validation
│       │       └── CorsMiddleware.php        # CORS handling
│       ├── Persistence/
│       │   └── PDO/                  # PDO implementations
│       │       └── UserRepository.php       # User data access
│       ├── Database/
│       │   └── PDOConnection.php     # Database connection management
│       ├── Security/
│       │   └── JWTManager.php        # JWT token management
│       └── Config/
│           └── AppConfig.php         # Configuration management
├── public/
│   ├── index.php                     # Application entry point
│   └── .htaccess                     # Apache rewrite rules
├── config/
│   ├── app.php                       # Application configuration
│   └── database.php                  # Database configuration
├── database/
│   └── migrations/
│       └── 001_create_users_table.sql       # Database schema
├── tests/                            # Comprehensive test suite
│   ├── Unit/                         # Unit tests
│   │   └── UserTest.php             # Domain entity tests
│   ├── Integration/                  # Integration tests
│   └── Functional/                   # Functional tests
│       └── AuthTest.php             # API endpoint tests
├── docker/                           # Docker configurations
├── composer.json                     # PHP dependencies (PHP 8.2+)
├── phpunit.xml                       # Testing configuration
├── Dockerfile                        # Multi-stage production build
├── docker-compose.yml                # Full stack deployment
├── .env.example                      # Environment template
└── README.md                         # Setup and API documentation
```

**PHP Vanilla Architecture Features:**
- **Pure Clean Architecture**: Domain-driven design without framework constraints
- **Custom HTTP Layer**: Built-in Router, Request/Response handling
- **PDO Database Layer**: Multi-database support (PostgreSQL, MySQL)
- **JWT Authentication**: Secure token-based authentication
- **Value Objects**: Type-safe domain modeling
- **Command/Handler Pattern**: CQRS-lite for business operations
- **PSR-4 Autoloading**: Modern PHP namespace organization
- **Dependency Injection**: Manual DI for learning and control

**Database Support:**
- **PostgreSQL** (default): Production-ready with UUID support
- **MySQL**: Alternative with charset configuration
- **PDO Abstraction**: Database-agnostic query layer
- **Migration System**: SQL-based schema management
- **Connection Pooling**: Singleton pattern for efficiency

**API Endpoints:**
```
GET  /api/v1/health          # Health check
POST /api/v1/auth/register   # User registration  
POST /api/v1/auth/login      # JWT authentication
POST /api/v1/auth/logout     # Token invalidation
GET  /api/v1/auth/me         # Current user info
GET  /api/v1/users           # List users
GET  /api/v1/users/{id}      # Get user by ID
POST /api/v1/users           # Create user
```

All generated projects include comprehensive README files with setup instructions, API documentation, and deployment guides.
