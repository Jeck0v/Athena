# 🧬 Boilerplate Project Generator

Athena includes a powerful project generator that creates production-ready full-stack applications with modern best practices, Docker integration, and comprehensive tooling.

## 📋 Table of Contents

- [🚀 Quick Start](#-quick-start)
- [🐍 FastAPI Projects](#-fastapi-projects)
- [🐹 Go Projects](#-go-projects)  
- [🌶️ Flask Projects](#-flask-projects)
- [🏗️ Architecture Patterns](#️-architecture-patterns)
- [🐳 Docker Integration](#-docker-integration)
- [🔧 Customization](#-customization)

## 🚀 Quick Start

### Generate Projects

```bash
# FastAPI + PostgreSQL + Nginx
athena init fastapi my-api --with-postgresql

# Go microservice with Gin + MongoDB
athena init go my-service --framework gin --with-mongodb  

# Flask application with PostgreSQL
athena init flask my-webapp --with-postgresql
```

### Project Structure Overview

All generated projects follow these principles:
- **Production-ready** configuration from day one
- **Docker integration** with multi-stage builds
- **Security best practices** (JWT, password hashing, CORS)
- **Testing framework** with example tests
- **Environment configuration** with validation
- **API documentation** auto-generation
- **Reverse proxy** configuration (Nginx)

## 🐍 FastAPI Projects

### Command Options

```bash
athena init fastapi PROJECT_NAME [OPTIONS]

Options:
  --with-postgresql     Use PostgreSQL database (async drivers)
  --with-mongodb        Use MongoDB database (default)
  --no-docker          Skip Docker configuration files
  --help               Show help information
```

### Generated FastAPI Structure

```
my-api/
├── app/
│   ├── __init__.py
│   ├── main.py                    # FastAPI application entry
│   ├── core/
│   │   ├── __init__.py
│   │   ├── config.py              # Pydantic settings management
│   │   ├── security.py            # JWT + bcrypt password hashing  
│   │   ├── database.py            # Async database configuration
│   │   └── middleware.py          # CORS, security headers
│   ├── api/
│   │   ├── __init__.py
│   │   └── v1/                    # API versioning
│   │       ├── __init__.py
│   │       ├── router.py          # API router configuration
│   │       ├── auth.py            # Authentication endpoints
│   │       ├── users.py           # User management endpoints
│   │       └── health.py          # Health check endpoint
│   ├── models/
│   │   ├── __init__.py
│   │   ├── user.py               # Database models (SQLAlchemy/Beanie)
│   │   └── base.py               # Base model configuration
│   ├── schemas/
│   │   ├── __init__.py
│   │   ├── user.py               # Pydantic schemas
│   │   ├── auth.py               # Authentication schemas
│   │   └── response.py           # Standard response schemas
│   ├── services/
│   │   ├── __init__.py
│   │   ├── auth.py               # Authentication business logic
│   │   └── user.py               # User management logic
│   └── utils/
│       ├── __init__.py
│       └── dependencies.py       # FastAPI dependencies
├── tests/
│   ├── __init__.py
│   ├── conftest.py               # Pytest configuration
│   ├── test_auth.py              # Authentication tests
│   ├── test_users.py             # User management tests
│   └── api/
│       ├── __init__.py
│       └── test_v1/              # API endpoint tests
│           ├── test_auth.py
│           └── test_users.py
├── nginx/
│   ├── nginx.conf                # Main Nginx configuration
│   └── conf.d/
│       └── default.conf          # Virtual host configuration
├── logs/                         # Application logs directory
├── scripts/
│   ├── start.sh                  # Application startup script
│   ├── test.sh                   # Testing script
│   └── lint.sh                   # Linting script
├── requirements.txt              # Python dependencies
├── requirements-dev.txt          # Development dependencies
├── Dockerfile                    # Multi-stage production build
├── docker-compose.yml            # Development environment
├── docker-compose.prod.yml       # Production environment
├── .env.example                  # Environment template
├── .gitignore                    # Git ignore rules
├── pytest.ini                   # Pytest configuration
├── pyproject.toml               # Project configuration
└── README.md                    # Project documentation
```

### FastAPI Features

**🔐 Authentication & Security:**
```python
# JWT token authentication with refresh
@router.post("/login")
async def login(credentials: UserLogin):
    user = await authenticate_user(credentials.email, credentials.password)
    tokens = create_tokens(user.id)
    return {"access_token": tokens.access, "refresh_token": tokens.refresh}

# Password hashing with bcrypt  
password_hash = hash_password("user_password")
is_valid = verify_password("user_password", password_hash)
```

**⚡ Async Database Integration:**
```python
# PostgreSQL with asyncpg
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine

# MongoDB with Motor  
from motor.motor_asyncio import AsyncIOMotorClient
```

**🚀 Performance Optimizations:**
- Connection pooling for databases
- Async request handling
- Response caching headers
- Gzip compression middleware

**📊 API Documentation:**
- Automatic OpenAPI/Swagger generation
- ReDoc alternative documentation
- Request/response schema validation

### FastAPI Configuration Examples

**Environment Variables (.env):**
```bash
# Application
APP_NAME=My FastAPI Application
APP_VERSION=1.0.0
DEBUG=False
SECRET_KEY=your-super-secret-key-change-in-production

# Database (PostgreSQL)
DATABASE_URL=postgresql+asyncpg://user:password@db:5432/myapp
POSTGRES_USER=myapp_user
POSTGRES_PASSWORD=secure_password
POSTGRES_DB=myapp_db

# Database (MongoDB)  
MONGODB_URL=mongodb://mongo:27017/myapp
MONGODB_DB=myapp

# Security
JWT_SECRET_KEY=jwt-secret-key
JWT_ALGORITHM=HS256
JWT_ACCESS_TOKEN_EXPIRE_MINUTES=30
JWT_REFRESH_TOKEN_EXPIRE_DAYS=7

# CORS
CORS_ORIGINS=["http://localhost:3000", "https://myapp.com"]
```

**Docker Compose (Development):**
```yaml
services:
  api:
    build: .
    ports:
      - "8000:8000"
    environment:
      - DEBUG=True
    volumes:
      - ./app:/app/app:ro
    depends_on:
      - db
    command: uvicorn app.main:app --host 0.0.0.0 --port 8000 --reload
      
  db:
    image: postgres:15
    environment:
      POSTGRES_USER: myapp_user
      POSTGRES_PASSWORD: myapp_password  
      POSTGRES_DB: myapp_db
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
      
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"  
    volumes:
      - ./nginx/conf.d:/etc/nginx/conf.d:ro
    depends_on:
      - api
```

## 🐹 Go Projects

### Command Options

```bash
athena init go PROJECT_NAME [OPTIONS]

Options:
  --framework FRAMEWORK    Web framework (gin, echo, fiber) [default: gin]
  --with-postgresql       Use PostgreSQL database 
  --with-mongodb          Use MongoDB database (default)
  --no-docker            Skip Docker configuration files
  --help                 Show help information
```

### Generated Go Structure

```
my-service/
├── cmd/
│   └── server/
│       └── main.go               # Application entrypoint
├── internal/                     # Private application code
│   ├── config/
│   │   └── config.go            # Configuration management
│   ├── handler/                 # HTTP handlers
│   │   ├── auth.go              # Authentication handlers
│   │   ├── user.go              # User management handlers
│   │   └── health.go            # Health check handler
│   ├── middleware/
│   │   ├── auth.go              # JWT authentication middleware
│   │   ├── cors.go              # CORS middleware
│   │   ├── logging.go           # Request logging middleware
│   │   └── recovery.go          # Panic recovery middleware
│   ├── model/
│   │   ├── user.go              # User model
│   │   └── response.go          # Standard response models
│   ├── repository/              # Data access layer
│   │   ├── user_postgres.go     # PostgreSQL implementation
│   │   ├── user_mongo.go        # MongoDB implementation
│   │   └── interfaces.go        # Repository interfaces
│   ├── service/                 # Business logic layer
│   │   ├── auth.go              # Authentication service
│   │   └── user.go              # User management service
│   └── router/
│       └── router.go            # Route configuration
├── pkg/                         # Public packages
│   ├── database/
│   │   ├── postgres.go          # PostgreSQL connection
│   │   └── mongo.go             # MongoDB connection
│   ├── jwt/
│   │   └── jwt.go               # JWT utilities
│   └── validator/
│       └── validator.go         # Input validation
├── tests/
│   ├── integration/
│   │   ├── auth_test.go         # Authentication integration tests
│   │   └── user_test.go         # User management integration tests
│   └── unit/
│       ├── service/
│       │   ├── auth_test.go     # Service unit tests
│       │   └── user_test.go
│       └── handler/
│           ├── auth_test.go     # Handler unit tests
│           └── user_test.go
├── scripts/
│   ├── build.sh                 # Build script
│   ├── test.sh                  # Testing script
│   └── lint.sh                  # Linting script
├── migrations/                  # Database migrations
│   ├── postgres/
│   │   └── 001_create_users.sql
│   └── mongo/
│       └── 001_create_indexes.js
├── Dockerfile                   # Multi-stage production build
├── docker-compose.yml           # Development environment
├── go.mod                       # Go modules
├── go.sum                       # Go modules checksum
├── .env.example                 # Environment template
├── .gitignore                   # Git ignore rules
├── Makefile                     # Build automation
└── README.md                    # Project documentation
```

### Go Framework Comparisons

| Feature | Gin | Echo | Fiber |
|---------|-----|------|-------|
| **Performance** | High | High | Very High |
| **Middleware** | Rich ecosystem | Built-in + community | Express.js-like |
| **Routing** | Radix tree | Radix tree | Fastest |
| **JSON Binding** | Built-in | Built-in | Built-in |
| **Community** | Largest | Growing | Rapidly growing |
| **Learning Curve** | Easy | Easy | Express.js familiar |

**Gin Example (Default):**
```go
func SetupRouter() *gin.Engine {
    r := gin.New()
    r.Use(gin.Logger())
    r.Use(gin.Recovery())
    r.Use(middleware.CORS())
    
    api := r.Group("/api/v1")
    {
        api.POST("/auth/login", handler.Login)
        api.GET("/users", middleware.AuthRequired(), handler.GetUsers)
    }
    return r
}
```

**Echo Example:**
```go
func SetupRouter() *echo.Echo {
    e := echo.New()
    e.Use(echomiddleware.Logger())
    e.Use(echomiddleware.Recover())
    e.Use(middleware.CORS())
    
    api := e.Group("/api/v1")
    api.POST("/auth/login", handler.Login)  
    api.GET("/users", handler.GetUsers, middleware.AuthRequired())
    return e
}
```

**Fiber Example:**
```go
func SetupRouter() *fiber.App {
    app := fiber.New()
    app.Use(logger.New())
    app.Use(recover.New())
    app.Use(middleware.CORS())
    
    api := app.Group("/api/v1")
    api.Post("/auth/login", handler.Login)
    api.Get("/users", middleware.AuthRequired(), handler.GetUsers)
    return app
}
```

### Go Features

**🏗️ Clean Architecture:**
- Clear separation of concerns
- Dependency injection patterns
- Interface-based design
- Testable code structure

**🔐 Security Implementation:**
```go
// JWT middleware
func AuthRequired() gin.HandlerFunc {
    return gin.HandlerFunc(func(c *gin.Context) {
        token := extractToken(c)
        claims, err := jwt.ValidateToken(token)
        if err != nil {
            c.JSON(401, gin.H{"error": "Invalid token"})
            c.Abort()
            return
        }
        c.Set("user_id", claims.UserID)
        c.Next()
    })
}

// Password hashing
hashedPassword := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
```

**💾 Database Integration:**
```go
// PostgreSQL with pgx
conn, err := pgx.Connect(context.Background(), databaseURL)

// MongoDB with official driver  
client, err := mongo.Connect(ctx, options.Client().ApplyURI(mongoURL))
```

## 🌶️ Flask Projects

### Command Options

```bash
athena init flask PROJECT_NAME [OPTIONS]

Options:
  --with-postgresql     Use PostgreSQL database
  --with-mongodb        Use MongoDB database (default)
  --no-docker          Skip Docker configuration files
  --help               Show help information
```

### Generated Flask Structure

```
my-webapp/
├── app/
│   ├── __init__.py               # Flask application factory
│   ├── models/
│   │   ├── __init__.py
│   │   ├── user.py               # User model (SQLAlchemy/MongoEngine)
│   │   └── base.py               # Base model
│   ├── api/
│   │   ├── __init__.py
│   │   ├── auth.py               # Authentication blueprints
│   │   ├── users.py              # User management blueprints
│   │   └── health.py             # Health check endpoint
│   ├── services/
│   │   ├── __init__.py
│   │   ├── auth_service.py       # Authentication business logic
│   │   └── user_service.py       # User management logic
│   ├── utils/
│   │   ├── __init__.py
│   │   ├── decorators.py         # Custom decorators
│   │   └── validators.py         # Input validation
│   └── config.py                 # Flask configuration
├── tests/
│   ├── __init__.py
│   ├── conftest.py              # Pytest configuration
│   ├── test_auth.py             # Authentication tests
│   └── test_users.py            # User management tests
├── migrations/                   # Flask-Migrate database migrations
├── instance/                     # Instance-specific configurations
├── nginx/                        # Nginx reverse proxy configuration
├── logs/                         # Application logs
├── requirements.txt              # Python dependencies
├── requirements-dev.txt          # Development dependencies
├── Dockerfile                    # Production Docker build
├── docker-compose.yml            # Development environment
├── .env.example                  # Environment template
├── .flaskenv                     # Flask environment variables
├── config.py                     # Application configuration
└── run.py                        # Development server runner
```

### Flask Features

**🏭 Application Factory Pattern:**
```python
def create_app(config_name='development'):
    app = Flask(__name__)
    app.config.from_object(config[config_name])
    
    # Initialize extensions
    db.init_app(app)
    migrate.init_app(app, db)
    jwt.init_app(app)
    cors.init_app(app)
    
    # Register blueprints
    app.register_blueprint(auth_bp, url_prefix='/api/v1/auth')
    app.register_blueprint(users_bp, url_prefix='/api/v1/users')
    
    return app
```

**🗄️ Database Models:**
```python
# SQLAlchemy model
class User(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    email = db.Column(db.String(120), unique=True, nullable=False)
    password_hash = db.Column(db.String(60), nullable=False)
    created_at = db.Column(db.DateTime, default=datetime.utcnow)

# MongoEngine model
class User(Document):
    email = StringField(required=True, unique=True)
    password_hash = StringField(required=True)
    created_at = DateTimeField(default=datetime.utcnow)
```

## 🏗️ Architecture Patterns

### Common Patterns Across All Projects

**1. Layered Architecture:**
```
┌─────────────────┐
│   Presentation  │  ← API handlers/controllers
│      Layer      │
├─────────────────┤
│   Business      │  ← Services/use cases
│     Logic       │
├─────────────────┤
│   Data Access   │  ← Repositories/models
│      Layer      │
├─────────────────┤
│   Infrastructure│  ← Database/external services
│      Layer      │
└─────────────────┘
```

**2. Dependency Injection:**
- Interface-based design
- Easy testing and mocking
- Loose coupling between layers

**3. Configuration Management:**
- Environment-based configuration
- Validation of required settings
- Secure secret handling

**4. Error Handling:**
- Structured error responses
- Proper HTTP status codes
- Logging integration

### Security Best Practices

**Authentication & Authorization:**
- JWT tokens with refresh mechanism
- Password hashing (bcrypt)
- Role-based access control (future enhancement)

**Input Validation:**
- Schema validation for all endpoints
- SQL injection prevention
- XSS protection

**Security Headers:**
- CORS configuration
- Security headers middleware
- Rate limiting (future enhancement)

## 🐳 Docker Integration

### Multi-Stage Dockerfile Pattern

All generated projects use optimized Docker builds:

```dockerfile
# Build stage
FROM node:18-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

# Production stage  
FROM node:18-alpine AS production
WORKDIR /app
COPY --from=builder /app/node_modules ./node_modules
COPY . .
EXPOSE 3000
CMD ["npm", "start"]
```

### Docker Compose Environments

**Development (docker-compose.yml):**
- Volume mounts for hot reload
- Debug configuration
- Database with exposed ports

**Production (docker-compose.prod.yml):**
- Optimized builds
- Security hardening
- Health checks
- Resource limits

### Nginx Reverse Proxy

All web projects include Nginx configuration:

```nginx
server {
    listen 80;
    server_name localhost;
    
    location / {
        proxy_pass http://api:8000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## 🔧 Customization

### Template Variables

Projects are generated with configurable template variables:

```bash
# Project name transformations
{{project_name}}     # MY_PROJECT  
{{kebab_case}}       # my-project
{{snake_case}}       # my_project
{{pascal_case}}      # MyProject
{{camel_case}}       # myProject
```

### Database Selection

**PostgreSQL Features:**
- AsyncPG driver (FastAPI)
- Pgx driver (Go)
- SQLAlchemy (Flask)
- Migration support
- Connection pooling

**MongoDB Features:**
- Motor driver (FastAPI) 
- Official MongoDB driver (Go)
- MongoEngine (Flask)
- Index creation
- Aggregation pipeline support

### Framework Selection (Go)

Each Go framework includes optimized patterns:
- Middleware configuration
- Route grouping
- Error handling
- JSON binding/validation

---

The Athena boilerplate generator provides a solid foundation for modern web applications with production-ready configurations, security best practices, and comprehensive tooling.