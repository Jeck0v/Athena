//! Template definitions for FastAPI, Flask, and Go boilerplates

pub mod fastapi {
    pub const MAIN_PY: &str = r#"from __future__ import annotations

from typing import Any
from fastapi import FastAPI, APIRouter, Request, Response, status
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.trustedhost import TrustedHostMiddleware
from fastapi.responses import JSONResponse
import uvicorn
import structlog
import uuid
from contextlib import asynccontextmanager

from app.core.config import settings
from app.core.logging import setup_logging, RequestIdMiddleware
from app.core.security import verify_token
from app.database.connection import init_database, close_database_connection
from app.api import health
from app.api.v1 import auth, users
from app.core.rate_limiting import RateLimitMiddleware

# Setup structured logging
setup_logging()
logger = structlog.get_logger()

@asynccontextmanager
async def lifespan(app: FastAPI):
    # Startup
    logger.info("Starting up {{project_name}} application", service="{{project_name}}", version="1.0.0")
    await init_database()
    yield
    # Shutdown
    logger.info("Shutting down {{project_name}} application", service="{{project_name}}")
    await close_database_connection()

app = FastAPI(
    title="{{project_name}} API",
    description="Production-ready {{project_name}} API with authentication and observability",
    version="1.0.0",
    lifespan=lifespan,
    openapi_tags=[
        {"name": "health", "description": "Health and readiness checks"},
        {"name": "authentication", "description": "User authentication operations"},
        {"name": "users", "description": "User management operations"},
    ],
)

# Add request ID middleware first
app.add_middleware(RequestIdMiddleware)

# Rate limiting middleware
app.add_middleware(RateLimitMiddleware, calls=100, period=60)

# Security middleware
app.add_middleware(TrustedHostMiddleware, allowed_hosts=settings.ALLOWED_HOSTS + ["localhost", "127.0.0.1"])
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.ALLOWED_HOSTS,
    allow_credentials=True,
    allow_methods=["GET", "POST", "PUT", "DELETE", "OPTIONS"],
    allow_headers=["*"],
    expose_headers=["X-Request-ID"],
)

# Exception handler for better error responses
@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception) -> JSONResponse:
    request_id = getattr(request.state, "request_id", str(uuid.uuid4()))
    logger.error(
        "Unhandled exception",
        request_id=request_id,
        path=request.url.path,
        method=request.method,
        error=str(exc),
        exc_info=True
    )
    return JSONResponse(
        status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        content={
            "error": "Internal server error",
            "request_id": request_id,
            "message": "An unexpected error occurred"
        },
        headers={"X-Request-ID": request_id}
    )

# Include routers
app.include_router(health.router, tags=["health"])

# API v1 routes
api_v1 = APIRouter(prefix="/api/v1")
api_v1.include_router(auth.router, prefix="/auth", tags=["authentication"])
api_v1.include_router(users.router, prefix="/users", tags=["users"])
app.include_router(api_v1)

@app.get("/")
async def root(request: Request) -> dict[str, Any]:
    request_id = getattr(request.state, "request_id", str(uuid.uuid4()))
    logger.info("Root endpoint accessed", request_id=request_id)
    return {
        "message": "{{project_name}} API is running",
        "version": "1.0.0",
        "environment": settings.ENVIRONMENT,
        "request_id": request_id
    }

if __name__ == "__main__":
    uvicorn.run(
        "app.main:app",
        host="0.0.0.0",
        port=8000,
        reload=settings.ENVIRONMENT == "development",
        access_log=False,  # Use structured logging instead
        log_config=None   # Disable default logging
    )
"#;

    pub const CONFIG_PY: &str = r#"from __future__ import annotations

from pydantic_settings import BaseSettings
from pydantic import ConfigDict, Field, validator
from typing import List
import os

class Settings(BaseSettings):
    # Application
    PROJECT_NAME: str = "{{project_name}}"
    VERSION: str = "1.0.0"
    ENVIRONMENT: str = Field(default="development", description="Environment: development, staging, production")
    DEBUG: bool = Field(default=False, description="Enable debug mode")

    # Security
    SECRET_KEY: str = Field("{{secret_key}}", min_length=32, description="Secret key for JWT signing")
    ACCESS_TOKEN_EXPIRE_MINUTES: int = Field(default=30, ge=5, le=1440)
    REFRESH_TOKEN_EXPIRE_MINUTES: int = Field(default=60 * 24 * 7, ge=60)  # 7 days
    ALGORITHM: str = "HS256"

    # CORS and Security
    ALLOWED_HOSTS: List[str] = Field(
        default=["http://localhost:3000", "http://127.0.0.1:3000"],
        description="Allowed CORS origins"
    )
    TRUSTED_HOSTS: List[str] = Field(
        default=["localhost", "127.0.0.1"],
        description="Trusted hosts for TrustedHostMiddleware"
    )

    # Database
    MONGODB_URL: str = "mongodb://localhost:27017"
    DATABASE_NAME: str = "{{snake_case}}_db"
    DATABASE_URL: str = "postgresql+asyncpg://user:password@localhost/{{snake_case}}_db"
    POSTGRES_PASSWORD: str = "changeme"

    # Connection Pool Settings
    DB_POOL_SIZE: int = Field(default=20, ge=5, le=100)
    DB_POOL_OVERFLOW: int = Field(default=0, ge=0, le=50)

    # Redis (for caching/sessions)
    REDIS_URL: str = "redis://localhost:6379"

    # Logging
    LOG_LEVEL: str = Field(default="INFO", description="Logging level")
    LOG_FORMAT: str = Field(default="json", description="Logging format: json or text")

    # Rate Limiting
    RATE_LIMIT_ENABLED: bool = Field(default=True, description="Enable rate limiting")
    RATE_LIMIT_CALLS: int = Field(default=100, ge=1, description="Rate limit calls per period")
    RATE_LIMIT_PERIOD: int = Field(default=60, ge=1, description="Rate limit period in seconds")

    # OpenTelemetry (optional)
    OTEL_ENABLED: bool = Field(default=False, description="Enable OpenTelemetry")
    OTEL_ENDPOINT: str = Field(default="", description="OpenTelemetry endpoint")

    @validator("ENVIRONMENT")
    def validate_environment(cls, v: str) -> str:
        if v not in ["development", "staging", "production"]:
            raise ValueError("ENVIRONMENT must be development, staging, or production")
        return v

    @validator("LOG_LEVEL")
    def validate_log_level(cls, v: str) -> str:
        if v.upper() not in ["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"]:
            raise ValueError("LOG_LEVEL must be DEBUG, INFO, WARNING, ERROR, or CRITICAL")
        return v.upper()

    @validator("LOG_FORMAT")
    def validate_log_format(cls, v: str) -> str:
        if v not in ["json", "text"]:
            raise ValueError("LOG_FORMAT must be json or text")
        return v

    model_config = ConfigDict(
        env_file=".env",
        case_sensitive=True,
        validate_assignment=True,
        extra="forbid"
    )

    @property
    def is_development(self) -> bool:
        return self.ENVIRONMENT == "development"

    @property
    def is_production(self) -> bool:
        return self.ENVIRONMENT == "production"

settings = Settings()
"#;

    pub const SECURITY_PY: &str = r#"from datetime import datetime, timedelta
from typing import Optional, Dict, Any
from jose import JWTError, jwt
from passlib.context import CryptContext
from fastapi import HTTPException, status
import secrets

from app.core.config import settings

# Password hashing
pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")

def verify_password(plain_password: str, hashed_password: str) -> bool:
    """Verify a password against its hash"""
    return pwd_context.verify(plain_password, hashed_password)

def get_password_hash(password: str) -> str:
    """Hash a password"""
    return pwd_context.hash(password)

def create_access_token(data: Dict[str, Any]) -> str:
    """Create JWT access token"""
    to_encode = data.copy()
    expire = datetime.utcnow() + timedelta(minutes=settings.ACCESS_TOKEN_EXPIRE_MINUTES)
    to_encode.update({"exp": expire, "type": "access"})

    encoded_jwt = jwt.encode(to_encode, settings.SECRET_KEY, algorithm=settings.ALGORITHM)
    return encoded_jwt

def create_refresh_token(data: Dict[str, Any]) -> str:
    """Create JWT refresh token"""
    to_encode = data.copy()
    expire = datetime.utcnow() + timedelta(minutes=settings.REFRESH_TOKEN_EXPIRE_MINUTES)
    to_encode.update({"exp": expire, "type": "refresh"})

    encoded_jwt = jwt.encode(to_encode, settings.SECRET_KEY, algorithm=settings.ALGORITHM)
    return encoded_jwt

def verify_token(token: str, token_type: str = "access") -> Dict[str, Any]:
    """Verify and decode JWT token"""
    try:
        payload = jwt.decode(token, settings.SECRET_KEY, algorithms=[settings.ALGORITHM])

        # Verify token type
        if payload.get("type") != token_type:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Invalid token type"
            )

        return payload
    except JWTError:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Could not validate credentials"
        )

def generate_secure_secret() -> str:
    """Generate a secure secret key"""
    return secrets.token_urlsafe(32)
"#;

    pub const REQUIREMENTS_TXT: &str = r#"# Core Framework - Latest 2025 versions
fastapi==0.115.0
uvicorn[standard]==0.32.0
gunicorn==23.0.0
pydantic==2.9.0
pydantic-settings==2.5.0

# Authentication & Security
python-jose[cryptography]==3.3.0
passlib[bcrypt]==1.7.4
python-multipart==0.0.12

# Structured Logging
structlog==24.4.0
python-json-logger==2.0.7

# Database{{#if mongodb}}
motor==3.6.0
pymongo==4.10.1{{/if}}{{#if postgresql}}
asyncpg==0.29.0
sqlalchemy[asyncio]==2.0.36
sqlmodel==0.0.22
alembic==1.14.0{{/if}}

# Caching & Storage
redis==5.1.1

# Performance & Monitoring
slowapi==0.1.9
prometheus-fastapi-instrumentator==7.0.0

# Optional: OpenTelemetry
opentelemetry-api==1.27.0
opentelemetry-sdk==1.27.0
opentelemetry-instrumentation-fastapi==0.48b0
opentelemetry-instrumentation-sqlalchemy==0.48b0
opentelemetry-exporter-otlp==1.27.0

# Development & Testing
pytest==8.3.3
pytest-asyncio==0.24.0
pytest-cov==5.0.0
httpx==0.27.2
factory-boy==3.3.1

# Code Quality
ruff==0.7.4
mypy==1.13.0
pre-commit==4.0.1
"#;

    pub const DOCKERFILE: &str = r#"FROM python:3.12-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends gcc \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy requirements first for better caching
COPY requirements.txt .

# Install Python dependencies in user space
RUN pip install --no-cache-dir --user -r requirements.txt

# Production stage
FROM python:3.11-slim

# Set environment variables
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    PATH=/home/appuser/.local/bin:$PATH

WORKDIR /app

# Create non-root user first
RUN useradd --uid 1000 --create-home --shell /bin/bash appuser

# Copy dependencies from builder stage
COPY --from=builder /root/.local /home/appuser/.local

# Install wget for health check
RUN apt-get update && apt-get install -y --no-install-recommends wget \
    && rm -rf /var/lib/apt/lists/*

# Copy application code
COPY . .

# Change ownership to non-root user
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget -qO- http://localhost:8000/health || exit 1

# Expose port
EXPOSE 8000

# Run with gunicorn for production
CMD ["gunicorn", "app.main:app", "-k", "uvicorn.workers.UvicornWorker", "-b", "0.0.0.0:8000", "--workers", "4", "--timeout", "120"]
"#;

    pub const DOCKER_COMPOSE_YML: &str = r#"services:
  {{kebab_case}}-api:
    build: .
    ports:
      - "8000:8000"
    environment:
      - ENVIRONMENT=production{{#if mongodb}}
      - MONGODB_URL=mongodb://mongo:27017{{/if}}{{#if postgresql}}
      - DATABASE_URL=postgresql://postgres:${POSTGRES_PASSWORD}@postgres:5432/{{snake_case}}_db{{/if}}
      - REDIS_URL=redis://redis:6379
      - SECRET_KEY=${SECRET_KEY}
    depends_on:{{#if mongodb}}
      - mongo{{/if}}{{#if postgresql}}
      - postgres{{/if}}
      - redis
    volumes:
      - ./logs:/app/logs
    restart: unless-stopped
    networks:
      - {{kebab_case}}-network

{{#if mongodb}}
  mongo:
    image: mongo:7
    environment:
      - MONGO_INITDB_DATABASE={{snake_case}}_db
    volumes:
      - mongo_data:/data/db
    restart: unless-stopped
    networks:
      - {{kebab_case}}-network
{{/if}}

{{#if postgresql}}
  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB={{snake_case}}_db
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped
    networks:
      - {{kebab_case}}-network
{{/if}}

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    restart: unless-stopped
    networks:
      - {{kebab_case}}-network

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/conf.d:/etc/nginx/conf.d:ro
    depends_on:
      - {{kebab_case}}-api
    restart: unless-stopped
    networks:
      - {{kebab_case}}-network

volumes:{{#if mongodb}}
  mongo_data:{{/if}}{{#if postgresql}}
  postgres_data:{{/if}}
  redis_data:

networks:
  {{kebab_case}}-network:
    driver: bridge
"#;

    pub const NGINX_CONF: &str = r#"# Main Nginx configuration
user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log warn;
pid /var/run/nginx.pid;

events {
    worker_connections 1024;
    use epoll;
    multi_accept on;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # Basic settings
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;
    client_max_body_size 16M;

    # Security headers
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self'" always;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types
        text/plain
        text/css
        text/xml
        text/javascript
        application/json
        application/javascript
        application/xml+rss
        application/atom+xml;

    # Logging format
    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';

    access_log /var/log/nginx/access.log main;

    # Include server configurations
    include /etc/nginx/conf.d/*.conf;
}
"#;

    pub const NGINX_DEFAULT_CONF: &str = r#"# Default server configuration for {{project_name}}
upstream {{kebab_case}}_api {
    server {{kebab_case}}-api:8000;
    keepalive 32;
}

server {
    listen 80;
    server_name _;

    # Apply rate limiting
    limit_req zone=api burst=20 nodelay;

    # API proxy
    location /api/ {
        proxy_pass http://{{kebab_case}}_api;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }

    # Health check endpoint
    location /health {
        proxy_pass http://{{kebab_case}}_api;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        access_log off;
    }

    # Root endpoint
    location / {
        proxy_pass http://{{kebab_case}}_api;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}
"#;
}

// New structured logging module
pub const LOGGING_PY: &str = r#"from __future__ import annotations

import logging
import logging.config
import structlog
import uuid
from typing import Any, Dict
from fastapi import Request, Response
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request as StarletteRequest

from app.core.config import settings

def setup_logging() -> None:
    """Configure structured logging with structlog"""

    # Configure structlog
    if settings.LOG_FORMAT == "json":
        processors = [
            structlog.stdlib.filter_by_level,
            structlog.stdlib.add_logger_name,
            structlog.stdlib.add_log_level,
            structlog.stdlib.PositionalArgumentsFormatter(),
            structlog.processors.TimeStamper(fmt="iso"),
            structlog.processors.StackInfoRenderer(),
            structlog.processors.format_exc_info,
            structlog.processors.UnicodeDecoder(),
            structlog.processors.JSONRenderer()
        ]
    else:
        processors = [
            structlog.stdlib.filter_by_level,
            structlog.stdlib.add_logger_name,
            structlog.stdlib.add_log_level,
            structlog.stdlib.PositionalArgumentsFormatter(),
            structlog.processors.TimeStamper(fmt="%Y-%m-%d %H:%M:%S"),
            structlog.processors.StackInfoRenderer(),
            structlog.processors.format_exc_info,
            structlog.dev.ConsoleRenderer()
        ]

    structlog.configure(
        processors=processors,
        context_class=dict,
        logger_factory=structlog.stdlib.LoggerFactory(),
        wrapper_class=structlog.stdlib.BoundLogger,
        cache_logger_on_first_use=True,
    )

    # Configure standard library logging
    logging.basicConfig(
        format="%(message)s",
        level=getattr(logging, settings.LOG_LEVEL),
        force=True,
    )

class RequestIdMiddleware(BaseHTTPMiddleware):
    """Middleware to add request ID to each request"""

    async def dispatch(self, request: Request, call_next) -> Response:
        # Generate or extract request ID
        request_id = request.headers.get("X-Request-ID") or str(uuid.uuid4())

        # Store request ID in request state
        request.state.request_id = request_id

        # Set up structured logging context
        logger = structlog.get_logger()
        logger = logger.bind(
            request_id=request_id,
            method=request.method,
            path=request.url.path,
            user_agent=request.headers.get("User-Agent", ""),
            remote_addr=request.client.host if request.client else None
        )

        # Log request
        logger.info(
            "Request started",
            query_params=dict(request.query_params)
        )

        try:
            response = await call_next(request)

            # Log response
            logger.info(
                "Request completed",
                status_code=response.status_code,
                response_time_ms=None  # Could add timing here
            )

            # Add request ID to response headers
            response.headers["X-Request-ID"] = request_id

            return response

        except Exception as exc:
            logger.error(
                "Request failed",
                error=str(exc),
                exc_info=True
            )
            raise
"#;

// Rate limiting middleware
pub const RATE_LIMITING_PY: &str = r#"from __future__ import annotations

import time
from typing import Optional
from fastapi import Request, HTTPException, status
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.responses import Response
import structlog
from collections import defaultdict, deque
from threading import Lock

from app.core.config import settings

logger = structlog.get_logger()

class InMemoryRateLimiter:
    """Simple in-memory rate limiter using sliding window"""

    def __init__(self):
        self.requests = defaultdict(deque)
        self.lock = Lock()

    def is_allowed(self, key: str, limit: int, window: int) -> bool:
        """Check if request is allowed under rate limit"""
        now = time.time()

        with self.lock:
            # Remove old requests outside the window
            while self.requests[key] and self.requests[key][0] <= now - window:
                self.requests[key].popleft()

            # Check if under limit
            if len(self.requests[key]) >= limit:
                return False

            # Add current request
            self.requests[key].append(now)
            return True

    def get_remaining(self, key: str, limit: int, window: int) -> int:
        """Get remaining requests allowed"""
        now = time.time()

        with self.lock:
            # Clean old requests
            while self.requests[key] and self.requests[key][0] <= now - window:
                self.requests[key].popleft()

            return max(0, limit - len(self.requests[key]))

    def get_reset_time(self, key: str, window: int) -> Optional[float]:
        """Get time when rate limit resets"""
        with self.lock:
            if not self.requests[key]:
                return None
            return self.requests[key][0] + window

class RateLimitMiddleware(BaseHTTPMiddleware):
    """Rate limiting middleware"""

    def __init__(self, app, calls: int = 100, period: int = 60):
        super().__init__(app)
        self.calls = calls
        self.period = period
        self.limiter = InMemoryRateLimiter()

    def get_client_key(self, request: Request) -> str:
        """Get client identifier for rate limiting"""
        # Use X-Forwarded-For if available (behind proxy)
        forwarded_for = request.headers.get("X-Forwarded-For")
        if forwarded_for:
            return forwarded_for.split(",")[0].strip()

        # Fall back to direct client IP
        return request.client.host if request.client else "unknown"

    async def dispatch(self, request: Request, call_next) -> Response:
        if not settings.RATE_LIMIT_ENABLED:
            return await call_next(request)

        # Skip rate limiting for health checks
        if request.url.path in ["/health", "/health/ready", "/health/live"]:
            return await call_next(request)

        client_key = self.get_client_key(request)

        # Check rate limit
        if not self.limiter.is_allowed(client_key, self.calls, self.period):
            logger.warning(
                "Rate limit exceeded",
                client=client_key,
                path=request.url.path,
                method=request.method
            )

            reset_time = self.limiter.get_reset_time(client_key, self.period)

            raise HTTPException(
                status_code=status.HTTP_429_TOO_MANY_REQUESTS,
                detail="Rate limit exceeded",
                headers={
                    "X-RateLimit-Limit": str(self.calls),
                    "X-RateLimit-Remaining": "0",
                    "X-RateLimit-Reset": str(int(reset_time)) if reset_time else "",
                    "Retry-After": str(self.period)
                }
            )

        # Add rate limit headers to response
        response = await call_next(request)

        remaining = self.limiter.get_remaining(client_key, self.calls, self.period)
        reset_time = self.limiter.get_reset_time(client_key, self.period)

        response.headers["X-RateLimit-Limit"] = str(self.calls)
        response.headers["X-RateLimit-Remaining"] = str(remaining)
        if reset_time:
            response.headers["X-RateLimit-Reset"] = str(int(reset_time))

        return response
"#;

pub mod go {
    pub const MAIN_GO: &str = r#"package main

import (
	"context"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"{{module_name}}/internal/config"
	"{{module_name}}/internal/database"
	"{{module_name}}/internal/handlers"
	"{{module_name}}/internal/middleware"
	"{{module_name}}/internal/routes"
	"{{module_name}}/pkg/logger"

	"github.com/gin-gonic/gin"
	"github.com/joho/godotenv"
)

func main() {
	// Setup structured logging first
	logger.Setup()
	log := slog.Default()

	// Load environment variables
	if err := godotenv.Load(); err != nil {
		log.Warn("No .env file found", "error", err)
	}

	// Load configuration
	cfg := config.Load()

	log.Info("Starting {{project_name}} application",
		"environment", cfg.Environment,
		"port", cfg.Port,
		"version", "1.0.0")

	// Set gin mode and disable default logging (we use slog)
	if cfg.Environment == "production" {
		gin.SetMode(gin.ReleaseMode)
	}
	gin.DisableConsoleColor()

	// Initialize database with context
	ctx := context.Background()
	db, err := database.Connect(ctx, cfg)
	if err != nil {
		log.Error("Failed to connect to database", "error", err)
		os.Exit(1)
	}
	defer database.Close(db)

	// Initialize handlers
	h := handlers.New(db, cfg)

	// Setup router with structured logging middleware
	router := gin.New()
	router.Use(middleware.StructuredLogger())
	router.Use(middleware.Recovery())
	router.Use(middleware.CORS())
	router.Use(middleware.Security())
	router.Use(middleware.RequestID())

	// Setup routes
	routes.Setup(router, h)

	// Create server with improved settings
	srv := &http.Server{
		Addr:           ":" + cfg.Port,
		Handler:        router,
		ReadTimeout:    10 * time.Second,
		WriteTimeout:   10 * time.Second,
		IdleTimeout:    60 * time.Second,
		MaxHeaderBytes: 1 << 20, // 1MB
	}

	// Start server in goroutine
	go func() {
		log.Info("Server starting", "port", cfg.Port, "environment", cfg.Environment)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Error("Server failed to start", "error", err)
			os.Exit(1)
		}
	}()

	// Wait for interrupt signal with graceful shutdown
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit
	log.Info("Shutting down server gracefully...")

	// Graceful shutdown with extended timeout
	ctx, cancel := context.WithTimeout(context.Background(), 45*time.Second)
	defer cancel()

	if err := srv.Shutdown(ctx); err != nil {
		log.Error("Server forced shutdown", "error", err)
		os.Exit(1)
	}

	log.Info("Server shutdown completed successfully")
}
"#;

    pub const GO_MOD: &str = r#"module {{module_name}}

go 1.22

require (
	// Core framework
	github.com/gin-gonic/gin v1.10.0

	// Authentication & Security
	github.com/golang-jwt/jwt/v5 v5.2.1
	golang.org/x/crypto v0.28.0

	// Configuration
	github.com/joho/godotenv v1.5.1

	// Database drivers & ORM
	go.mongodb.org/mongo-driver v1.17.1
	github.com/lib/pq v1.10.9
	github.com/jmoiron/sqlx v1.4.0

	// Utilities
	github.com/google/uuid v1.6.0

	// Validation
	github.com/go-playground/validator/v10 v10.22.1

	// Testing
	github.com/stretchr/testify v1.9.0
)
"#;

    pub const DOCKERFILE_GO: &str = r#"# Build stage
FROM golang:1.21-alpine AS builder

# Install git and ca-certificates
RUN apk add --no-cache git ca-certificates tzdata

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Download dependencies and tidy
RUN go mod tidy && go mod download

# Build the application
RUN CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -o main cmd/main.go

# Final stage
FROM alpine:latest

# Install ca-certificates
RUN apk --no-cache add ca-certificates

# Create app directory
WORKDIR /root/

# Copy binary from builder
COPY --from=builder /app/main .

# Create non-root user
RUN adduser -D -s /bin/sh appuser
USER appuser

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Run the application
CMD ["./main"]
"#;

    // New structured logging module for Go with slog
    pub const GO_LOGGER: &str = r#"package logger

import (
	"context"
	"log/slog"
	"os"
	"strings"
)

// LogFormat represents the logging format
type LogFormat string

const (
	FormatJSON LogFormat = "json"
	FormatText LogFormat = "text"
)

// Setup initializes structured logging with slog
func Setup() {
	logLevel := getLogLevel()
	logFormat := getLogFormat()

	var handler slog.Handler

	switch logFormat {
	case FormatJSON:
		handler = slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{
			Level: logLevel,
			AddSource: true,
		})
	default:
		handler = slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{
			Level: logLevel,
			AddSource: true,
		})
	}

	logger := slog.New(handler)
	slog.SetDefault(logger)
}

// getLogLevel returns the log level from environment
func getLogLevel() slog.Level {
	level := strings.ToUpper(os.Getenv("LOG_LEVEL"))
	switch level {
	case "DEBUG":
		return slog.LevelDebug
	case "INFO":
		return slog.LevelInfo
	case "WARN", "WARNING":
		return slog.LevelWarn
	case "ERROR":
		return slog.LevelError
	default:
		return slog.LevelInfo
	}
}

// getLogFormat returns the log format from environment
func getLogFormat() LogFormat {
	format := strings.ToLower(os.Getenv("LOG_FORMAT"))
	switch format {
	case "json":
		return FormatJSON
	default:
		return FormatText
	}
}

// WithRequestID adds request ID to the logger context
func WithRequestID(ctx context.Context, requestID string) context.Context {
	logger := slog.Default().With("request_id", requestID)
	return context.WithValue(ctx, "logger", logger)
}

// FromContext returns a logger with context information
func FromContext(ctx context.Context) *slog.Logger {
	if logger, ok := ctx.Value("logger").(*slog.Logger); ok {
		return logger
	}
	return slog.Default()
}
"#;
}

pub mod flask {
    pub const APP_INIT_PY: &str = r#"from __future__ import annotations

from typing import Type
from flask import Flask, request, g
from flask_sqlalchemy import SQLAlchemy
from flask_migrate import Migrate
from flask_jwt_extended import JWTManager
from flask_cors import CORS
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address
import structlog
import uuid
import time
import os

from app.core.config import Config
from app.core.extensions import db, migrate, jwt, cors, limiter
from app.core.logging import setup_logging
from app.api import health
from app.api.v1 import auth, users

def create_app(config_class: Type[Config] = Config) -> Flask:
    app = Flask(__name__)
    app.config.from_object(config_class)

    # Setup structured logging first
    setup_logging(app)
    logger = structlog.get_logger()

    # Initialize extensions
    db.init_app(app)
    migrate.init_app(app, db)
    jwt.init_app(app)
    cors.init_app(app)
    limiter.init_app(app)

    # Request ID middleware
    @app.before_request
    def before_request() -> None:
        g.request_id = request.headers.get('X-Request-ID', str(uuid.uuid4()))
        g.start_time = time.time()

        logger.info(
            \"Request started\",
            request_id=g.request_id,
            method=request.method,
            path=request.path,
            remote_addr=request.remote_addr,
            user_agent=request.headers.get('User-Agent')
        )

    @app.after_request
    def after_request(response):
        response.headers['X-Request-ID'] = g.get('request_id', 'unknown')

        duration = time.time() - g.get('start_time', time.time())

        logger.info(
            \"Request completed\",
            request_id=g.get('request_id'),
            status_code=response.status_code,
            duration_ms=round(duration * 1000, 2)
        )

        return response

    # Error handlers with structured logging
    @app.errorhandler(404)
    def not_found(error):
        logger.warning(
            \"Resource not found\",
            request_id=g.get('request_id'),
            path=request.path
        )
        return {'error': 'Resource not found', 'request_id': g.get('request_id')}, 404

    @app.errorhandler(500)
    def internal_error(error):
        logger.error(
            \"Internal server error\",
            request_id=g.get('request_id'),
            error=str(error)
        )
        return {'error': 'Internal server error', 'request_id': g.get('request_id')}, 500

    # Register blueprints
    app.register_blueprint(health.bp)
    app.register_blueprint(auth.bp, url_prefix='/api/v1/auth')
    app.register_blueprint(users.bp, url_prefix='/api/v1/users')

    @app.route('/')
    def index():
        return {
            'message': '{{project_name}} API is running',
            'version': '1.0.0',
            'environment': app.config.get('FLASK_ENV', 'unknown'),
            'request_id': g.get('request_id')
        }

    logger.info('{{project_name}} application created successfully')
    return app
"#;

    pub const CONFIG_PY: &str = r#"from __future__ import annotations

import os
from datetime import timedelta
from typing import Dict, Type

class Config:
    \"\"\"Base configuration class with 2025 best practices\"\"\"

    # Basic Flask configuration
    SECRET_KEY: str = os.environ.get('SECRET_KEY', '{{secret_key}}')

    # Application settings
    PROJECT_NAME: str = '{{project_name}}'
    VERSION: str = '1.0.0'

    # Database configuration with connection pooling
    SQLALCHEMY_DATABASE_URI: str = os.environ.get(
        'DATABASE_URL',
        'postgresql://user:password@localhost/{{snake_case}}_db'
    )
    SQLALCHEMY_TRACK_MODIFICATIONS: bool = False
    SQLALCHEMY_ENGINE_OPTIONS: Dict = {
        'pool_size': int(os.environ.get('DB_POOL_SIZE', '20')),
        'pool_timeout': int(os.environ.get('DB_POOL_TIMEOUT', '30')),
        'pool_recycle': int(os.environ.get('DB_POOL_RECYCLE', '3600')),
        'max_overflow': int(os.environ.get('DB_MAX_OVERFLOW', '0'))
    }

    # JWT Configuration with enhanced security
    JWT_SECRET_KEY: str = os.environ.get('JWT_SECRET_KEY', '{{secret_key}}')
    JWT_ACCESS_TOKEN_EXPIRES: timedelta = timedelta(
        minutes=int(os.environ.get('JWT_ACCESS_TOKEN_EXPIRES', '30'))
    )
    JWT_REFRESH_TOKEN_EXPIRES: timedelta = timedelta(
        days=int(os.environ.get('JWT_REFRESH_TOKEN_EXPIRES', '7'))
    )
    JWT_ALGORITHM: str = 'HS256'
    JWT_BLACKLIST_ENABLED: bool = True
    JWT_BLACKLIST_TOKEN_CHECKS: list = ['access', 'refresh']

    # CORS with enhanced security
    CORS_ORIGINS: list = os.environ.get(
        'ALLOWED_ORIGINS',
        'http://localhost:3000,http://127.0.0.1:3000'
    ).split(',')
    CORS_METHODS: list = ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS']
    CORS_ALLOW_HEADERS: list = ['Content-Type', 'Authorization']

    # Rate limiting
    RATELIMIT_STORAGE_URL: str = os.environ.get('REDIS_URL', 'redis://localhost:6379')
    RATELIMIT_DEFAULT: str = \"200 per day, 50 per hour\"

    # Logging configuration
    LOG_LEVEL: str = os.environ.get('LOG_LEVEL', 'INFO')
    LOG_FORMAT: str = os.environ.get('LOG_FORMAT', 'json')

    # Security headers
    SECURITY_HEADERS: Dict = {
        'X-Frame-Options': 'DENY',
        'X-Content-Type-Options': 'nosniff',
        'X-XSS-Protection': '1; mode=block',
        'Referrer-Policy': 'strict-origin-when-cross-origin',
        'Content-Security-Policy': \"default-src 'self'\"
    }

    # Environment
    FLASK_ENV: str = os.environ.get('FLASK_ENV', 'development')
    DEBUG: bool = os.environ.get('FLASK_DEBUG', '0') == '1'
    TESTING: bool = False

    @property
    def is_development(self) -> bool:
        return self.FLASK_ENV == 'development'

    @property
    def is_production(self) -> bool:
        return self.FLASK_ENV == 'production'

    @property
    def is_testing(self) -> bool:
        return self.TESTING

class DevelopmentConfig(Config):
    \"\"\"Development configuration\"\"\"
    DEBUG: bool = True
    LOG_LEVEL: str = 'DEBUG'

class ProductionConfig(Config):
    \"\"\"Production configuration with enhanced security\"\"\"
    DEBUG: bool = False
    TESTING: bool = False

    # Enhanced security for production
    SESSION_COOKIE_SECURE: bool = True
    SESSION_COOKIE_HTTPONLY: bool = True
    SESSION_COOKIE_SAMESITE: str = 'Lax'

    # Force HTTPS in production
    PREFERRED_URL_SCHEME: str = 'https'

class TestingConfig(Config):
    \"\"\"Testing configuration\"\"\"
    TESTING: bool = True
    DEBUG: bool = True
    SQLALCHEMY_DATABASE_URI: str = 'sqlite:///:memory:'
    JWT_ACCESS_TOKEN_EXPIRES: timedelta = timedelta(minutes=5)

    # Disable rate limiting in tests
    RATELIMIT_ENABLED: bool = False

config: Dict[str, Type[Config]] = {
    'development': DevelopmentConfig,
    'production': ProductionConfig,
    'testing': TestingConfig,
    'default': DevelopmentConfig
}
"#;

    pub const EXTENSIONS_PY: &str = r#"from flask_sqlalchemy import SQLAlchemy
from flask_migrate import Migrate
from flask_jwt_extended import JWTManager
from flask_cors import CORS
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address

# Initialize extensions
db = SQLAlchemy()
migrate = Migrate()
jwt = JWTManager()
cors = CORS()
limiter = Limiter(
    key_func=get_remote_address,
    default_limits=["200 per day", "50 per hour"]
)
"#;

    pub const SECURITY_PY: &str = r#"from werkzeug.security import generate_password_hash, check_password_hash
from flask_jwt_extended import create_access_token, create_refresh_token, jwt_required, get_jwt_identity
import secrets

def hash_password(password):
    """Hash a password using Werkzeug's secure methods"""
    return generate_password_hash(password)

def verify_password(password, password_hash):
    """Verify a password against its hash"""
    return check_password_hash(password_hash, password)

def generate_tokens(identity):
    """Generate access and refresh tokens for a user"""
    access_token = create_access_token(identity=identity)
    refresh_token = create_refresh_token(identity=identity)
    return {
        'access_token': access_token,
        'refresh_token': refresh_token,
        'token_type': 'bearer'
    }

def generate_secret_key():
    """Generate a secure secret key"""
    return secrets.token_urlsafe(32)
"#;

    pub const MAIN_PY: &str = r#"#!/usr/bin/env python3
import os
from app import create_app
from app.core.config import config

# Get environment
config_name = os.getenv('FLASK_ENV', 'development')
app = create_app(config[config_name])

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000, debug=app.config['DEBUG'])
"#;

    pub const HEALTH_BP: &str = r#"from flask import Blueprint, jsonify

bp = Blueprint('health', __name__)

@bp.route('/health')
def health_check():
    return jsonify({
        'status': 'healthy',
        'service': '{{project_name}} API'
    })
"#;

    pub const AUTH_BP: &str = r#"from flask import Blueprint, request, jsonify
from flask_jwt_extended import jwt_required, create_access_token, create_refresh_token, get_jwt_identity
from marshmallow import ValidationError
import logging

from app.core.extensions import db, limiter
from app.core.security import hash_password, verify_password, generate_tokens
from app.models.user import User
from app.schemas.user import UserRegistrationSchema, UserLoginSchema, TokenRefreshSchema
from app.services.user_service import UserService

bp = Blueprint('auth', __name__)
logger = logging.getLogger(__name__)

@bp.route('/register', methods=['POST'])
@limiter.limit("5 per minute")
def register():
    try:
        schema = UserRegistrationSchema()
        data = schema.load(request.get_json())
    except ValidationError as err:
        return jsonify({'errors': err.messages}), 400

    user_service = UserService()

    # Check if user already exists
    if user_service.get_by_email(data['email']):
        return jsonify({'message': 'User already exists'}), 409

    # Create new user
    hashed_password = hash_password(data['password'])
    user = user_service.create({
        'email': data['email'],
        'password_hash': hashed_password,
        'full_name': data['full_name']
    })

    # Generate tokens
    tokens = generate_tokens(str(user.id))

    logger.info(f"New user registered: {user.email}")
    return jsonify(tokens), 201

@bp.route('/login', methods=['POST'])
@limiter.limit("10 per minute")
def login():
    try:
        schema = UserLoginSchema()
        data = schema.load(request.get_json())
    except ValidationError as err:
        return jsonify({'errors': err.messages}), 400

    user_service = UserService()
    user = user_service.get_by_email(data['email'])

    if not user or not verify_password(data['password'], user.password_hash):
        logger.warning(f"Failed login attempt for email: {data['email']}")
        return jsonify({'message': 'Invalid credentials'}), 401

    if not user.is_active:
        return jsonify({'message': 'Account is deactivated'}), 403

    # Generate tokens
    tokens = generate_tokens(str(user.id))

    logger.info(f"User logged in: {user.email}")
    return jsonify(tokens), 200

@bp.route('/refresh', methods=['POST'])
@jwt_required(refresh=True)
def refresh():
    current_user_id = get_jwt_identity()

    user_service = UserService()
    user = user_service.get_by_id(current_user_id)

    if not user or not user.is_active:
        return jsonify({'message': 'User not found or inactive'}), 404

    # Create new access token
    new_access_token = create_access_token(identity=current_user_id)

    return jsonify({
        'access_token': new_access_token,
        'token_type': 'bearer'
    }), 200
"#;

    pub const USERS_BP: &str = r#"from flask import Blueprint, jsonify
from flask_jwt_extended import jwt_required, get_jwt_identity

from app.services.user_service import UserService
from app.schemas.user import UserResponseSchema

bp = Blueprint('users', __name__)

@bp.route('/me', methods=['GET'])
@jwt_required()
def get_current_user():
    current_user_id = get_jwt_identity()

    user_service = UserService()
    user = user_service.get_by_id(current_user_id)

    if not user:
        return jsonify({'message': 'User not found'}), 404

    schema = UserResponseSchema()
    return jsonify(schema.dump(user)), 200
"#;

    pub const POSTGRES_CONNECTION: &str = r#"from app.core.extensions import db

def init_db(app):
    """Initialize database tables"""
    with app.app_context():
        db.create_all()

def get_db():
    """Get database session"""
    return db.session
"#;

    pub const MYSQL_CONNECTION: &str = r#"from app.core.extensions import db

def init_db(app):
    """Initialize database tables"""
    with app.app_context():
        db.create_all()

def get_db():
    """Get database session"""
    return db.session
"#;

    pub const USER_MODEL: &str = r#"from datetime import datetime
import uuid
from sqlalchemy.dialects.postgresql import UUID
from app.core.extensions import db

class User(db.Model):
    __tablename__ = 'users'

    id = db.Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    email = db.Column(db.String(120), unique=True, nullable=False, index=True)
    password_hash = db.Column(db.String(255), nullable=False)
    full_name = db.Column(db.String(100), nullable=False)
    is_active = db.Column(db.Boolean, default=True)
    created_at = db.Column(db.DateTime, default=datetime.utcnow)
    updated_at = db.Column(db.DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

    def __repr__(self):
        return f'<User {self.email}>'

    def to_dict(self):
        return {
            'id': str(self.id),
            'email': self.email,
            'full_name': self.full_name,
            'is_active': self.is_active,
            'created_at': self.created_at.isoformat(),
            'updated_at': self.updated_at.isoformat()
        }
"#;

    pub const USER_MODEL_MYSQL: &str = r#"from datetime import datetime
import uuid
from app.core.extensions import db

class User(db.Model):
    __tablename__ = 'users'

    id = db.Column(db.String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    email = db.Column(db.String(120), unique=True, nullable=False, index=True)
    password_hash = db.Column(db.String(255), nullable=False)
    full_name = db.Column(db.String(100), nullable=False)
    is_active = db.Column(db.Boolean, default=True)
    created_at = db.Column(db.DateTime, default=datetime.utcnow)
    updated_at = db.Column(db.DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

    def __repr__(self):
        return f'<User {self.email}>'

    def to_dict(self):
        return {
            'id': self.id,
            'email': self.email,
            'full_name': self.full_name,
            'is_active': self.is_active,
            'created_at': self.created_at.isoformat(),
            'updated_at': self.updated_at.isoformat()
        }
"#;

    pub const USER_SCHEMA: &str = r#"from marshmallow import Schema, fields, validate, ValidationError

class UserRegistrationSchema(Schema):
    email = fields.Email(required=True)
    password = fields.Str(required=True, validate=validate.Length(min=8))
    full_name = fields.Str(required=True, validate=validate.Length(min=2, max=100))

class UserLoginSchema(Schema):
    email = fields.Email(required=True)
    password = fields.Str(required=True)

class UserResponseSchema(Schema):
    id = fields.Str()
    email = fields.Email()
    full_name = fields.Str()
    is_active = fields.Bool()
    created_at = fields.DateTime()
    updated_at = fields.DateTime()

class TokenRefreshSchema(Schema):
    refresh_token = fields.Str(required=True)
"#;

    pub const USER_SERVICE: &str = r#"from typing import Optional, Dict, Any
from app.core.extensions import db
from app.models.user import User

class UserService:
    def create(self, user_data: Dict[str, Any]) -> User:
        """Create a new user"""
        user = User(
            email=user_data['email'],
            password_hash=user_data['password_hash'],
            full_name=user_data['full_name'],
            is_active=user_data.get('is_active', True)
        )

        db.session.add(user)
        db.session.commit()
        return user

    def get_by_id(self, user_id: str) -> Optional[User]:
        """Get user by ID"""
        return User.query.filter_by(id=user_id).first()

    def get_by_email(self, email: str) -> Optional[User]:
        """Get user by email"""
        return User.query.filter_by(email=email).first()

    def update(self, user_id: str, update_data: Dict[str, Any]) -> Optional[User]:
        """Update user"""
        user = self.get_by_id(user_id)
        if not user:
            return None

        for key, value in update_data.items():
            if hasattr(user, key):
                setattr(user, key, value)

        db.session.commit()
        return user

    def delete(self, user_id: str) -> bool:
        """Delete user"""
        user = self.get_by_id(user_id)
        if not user:
            return False

        db.session.delete(user)
        db.session.commit()
        return True

    def list_users(self, page: int = 1, per_page: int = 20) -> Dict[str, Any]:
        """List users with pagination"""
        users = User.query.paginate(
            page=page,
            per_page=per_page,
            error_out=False
        )

        return {
            'users': [user.to_dict() for user in users.items],
            'total': users.total,
            'pages': users.pages,
            'page': page,
            'per_page': per_page
        }
"#;

    pub const DECORATORS: &str = r#"from functools import wraps
from flask import jsonify
from flask_jwt_extended import get_jwt_identity, jwt_required
from app.services.user_service import UserService

def admin_required(f):
    @wraps(f)
    @jwt_required()
    def decorated_function(*args, **kwargs):
        current_user_id = get_jwt_identity()
        user_service = UserService()
        user = user_service.get_by_id(current_user_id)

        if not user or not getattr(user, 'is_admin', False):
            return jsonify({'message': 'Admin access required'}), 403

        return f(*args, **kwargs)
    return decorated_function

def active_user_required(f):
    @wraps(f)
    @jwt_required()
    def decorated_function(*args, **kwargs):
        current_user_id = get_jwt_identity()
        user_service = UserService()
        user = user_service.get_by_id(current_user_id)

        if not user or not user.is_active:
            return jsonify({'message': 'Account is inactive'}), 403

        return f(*args, **kwargs)
    return decorated_function
"#;

    pub const REQUIREMENTS_TXT: &str = r#"# Core Framework - Latest 2025 versions
Flask==3.1.0
Flask-SQLAlchemy==3.2.0
Flask-Migrate==4.1.0
Flask-JWT-Extended==4.7.1
Flask-CORS==5.0.0
Flask-Limiter==3.8.0

# Database drivers
psycopg2-binary==2.9.10
SQLAlchemy==2.0.36

# Structured Logging
structlog==24.4.0
python-json-logger==2.0.7

# Serialization & Validation
marshmallow==3.23.0
marshmallow-sqlalchemy==1.1.0

# Security & Password hashing
Werkzeug==3.1.0
bcrypt==4.2.0

# Production server
gunicorn==23.0.0

# Configuration & Environment
python-dotenv==1.0.1

# Caching & Storage
redis==5.1.1

# Development & Testing
pytest==8.3.3
pytest-flask==1.3.0
pytest-cov==5.0.0
factory-boy==3.3.1

# Type checking & Code quality
mypy==1.13.0
ruff==0.7.4
pre-commit==4.0.1

# Monitoring (optional)
prometheus-flask-exporter==0.24.0
"#;

    pub const REQUIREMENTS_TXT_MYSQL: &str = r#"Flask==3.0.0
Flask-SQLAlchemy==3.1.1
Flask-Migrate==4.0.5
Flask-JWT-Extended==4.6.0
Flask-CORS==4.0.0
Flask-Limiter==3.5.0
PyMySQL==1.1.0
cryptography==41.0.7
marshmallow==3.20.1
Werkzeug==3.0.1
gunicorn==21.2.0
python-dotenv==1.0.0
redis==5.0.1
pytest==7.4.3
pytest-flask==1.3.0
"#;

    pub const DOCKERFILE: &str = r#"# =========================
# Build stage
# =========================
FROM python:3.12-slim AS builder

ENV VENV_PATH=/opt/venv
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    libpq-dev \
    && python -m venv $VENV_PATH \
    && rm -rf /var/lib/apt/lists/*

ENV PATH="$VENV_PATH/bin:$PATH"

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# =========================
# Runtime stage
# =========================
FROM python:3.11-slim

ENV VENV_PATH=/opt/venv \
    PATH="/opt/venv/bin:$PATH" \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    FLASK_ENV=production

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Création user non-root sans shell
RUN useradd --uid 1000 --create-home --shell /usr/sbin/nologin appuser

WORKDIR /app

# Copier seulement le venv depuis le builder
COPY --from=builder $VENV_PATH $VENV_PATH
COPY . .

# Répertoires nécessaires
RUN mkdir -p /app/logs /app/instance \
    && chown -R appuser:appuser /app

USER appuser

# Healthcheck basique
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:5000/health || exit 1

EXPOSE 5000

# Variables Gunicorn configurables
ENV WORKERS=4 \
    TIMEOUT=120

CMD ["sh", "-c", "exec gunicorn --bind 0.0.0.0:5000 --workers ${WORKERS} --timeout ${TIMEOUT} run:app"]
"#;

    pub const DOCKERFILE_MYSQL: &str = r#"# =========================
# Build stage
# =========================
FROM python:3.11-slim AS builder

ENV VENV_PATH=/opt/venv
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    pkg-config \
    default-libmysqlclient-dev \
    && python -m venv $VENV_PATH \
    && rm -rf /var/lib/apt/lists/*

ENV PATH="$VENV_PATH/bin:$PATH"

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# =========================
# Runtime stage
# =========================
FROM python:3.11-slim

ENV VENV_PATH=/opt/venv \
    PATH="/opt/venv/bin:$PATH" \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    FLASK_ENV=production

RUN apt-get update && apt-get install -y --no-install-recommends \
    default-mysql-client \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Création user non-root sans shell
RUN useradd --uid 1000 --create-home --shell /usr/sbin/nologin appuser

WORKDIR /app

# Copier seulement le venv depuis le builder
COPY --from=builder $VENV_PATH $VENV_PATH
COPY . .

# Répertoires nécessaires
RUN mkdir -p /app/logs /app/instance \
    && chown -R appuser:appuser /app

USER appuser

# Healthcheck basique
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:5000/health || exit 1

EXPOSE 5000

# Variables Gunicorn configurables
ENV WORKERS=4 \
    TIMEOUT=120

CMD ["sh", "-c", "exec gunicorn --bind 0.0.0.0:5000 --workers ${WORKERS} --timeout ${TIMEOUT} run:app"]
"#;

    pub const DOCKER_COMPOSE_YML: &str = r#"services:
  {{kebab_case}}-api:
    build: .
    ports:
      - "5000:5000"
    environment:
      - FLASK_ENV=production
      - DATABASE_URL=postgresql://postgres:${POSTGRES_PASSWORD}@postgres:5432/{{snake_case}}_db
      - REDIS_URL=redis://redis:6379
      - SECRET_KEY=${SECRET_KEY}
      - JWT_SECRET_KEY=${JWT_SECRET_KEY}
    depends_on:
      - postgres
      - redis
    volumes:
      - ./logs:/app/logs
    restart: unless-stopped
    networks:
      - {{kebab_case}}-network

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB={{snake_case}}_db
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped
    networks:
      - {{kebab_case}}-network

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    restart: unless-stopped
    networks:
      - {{kebab_case}}-network

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/conf.d:/etc/nginx/conf.d:ro
    depends_on:
      - {{kebab_case}}-api
    restart: unless-stopped
    networks:
      - {{kebab_case}}-network

volumes:
  postgres_data:
  redis_data:

networks:
  {{kebab_case}}-network:
    driver: bridge
"#;

    pub const DOCKER_COMPOSE_YML_MYSQL: &str = r#"services:
  {{kebab_case}}-api:
    build: .
    ports:
      - "5000:5000"
    environment:
      - FLASK_ENV=production
      - DATABASE_URL=mysql+pymysql://root:${MYSQL_ROOT_PASSWORD}@mysql:3306/{{snake_case}}_db
      - REDIS_URL=redis://redis:6379
      - SECRET_KEY=${SECRET_KEY}
      - JWT_SECRET_KEY=${JWT_SECRET_KEY}
    depends_on:
      - mysql
      - redis
    volumes:
      - ./logs:/app/logs
    restart: unless-stopped
    networks:
      - {{kebab_case}}-network

  mysql:
    image: mysql:8.0
    environment:
      - MYSQL_ROOT_PASSWORD=${MYSQL_ROOT_PASSWORD}
      - MYSQL_DATABASE={{snake_case}}_db
      - MYSQL_USER=app_user
      - MYSQL_PASSWORD=${MYSQL_PASSWORD}
    volumes:
      - mysql_data:/var/lib/mysql
    restart: unless-stopped
    networks:
      - {{kebab_case}}-network
    command: --default-authentication-plugin=mysql_native_password

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    restart: unless-stopped
    networks:
      - {{kebab_case}}-network

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/conf.d:/etc/nginx/conf.d:ro
    depends_on:
      - {{kebab_case}}-api
    restart: unless-stopped
    networks:
      - {{kebab_case}}-network

volumes:
  mysql_data:
  redis_data:

networks:
  {{kebab_case}}-network:
    driver: bridge
"#;

    pub const NGINX_CONF: &str = r#"# Main Nginx configuration
user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log warn;
pid /var/run/nginx.pid;

events {
    worker_connections 1024;
    use epoll;
    multi_accept on;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # Basic settings
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;
    client_max_body_size 16M;

    # Security headers
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self'" always;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types
        text/plain
        text/css
        text/xml
        text/javascript
        application/json
        application/javascript
        application/xml+rss
        application/atom+xml;

    # Logging format
    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';

    access_log /var/log/nginx/access.log main;

    # Include server configurations
    include /etc/nginx/conf.d/*.conf;
}
"#;

    pub const NGINX_DEFAULT_CONF: &str = r#"# Default server configuration for {{project_name}}
upstream {{kebab_case}}_api {
    server {{kebab_case}}-api:5000;
    keepalive 32;
}

server {
    listen 80;
    server_name _;

    # Apply rate limiting
    limit_req zone=api burst=20 nodelay;

    # API proxy
    location /api/ {
        proxy_pass http://{{kebab_case}}_api;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }

    # Health check endpoint
    location /health {
        proxy_pass http://{{kebab_case}}_api;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        access_log off;
    }

    # Root endpoint
    location / {
        proxy_pass http://{{kebab_case}}_api;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}
"#;

    pub const TEST_CONFIG: &str = r#"import pytest
import tempfile
import os
from app import create_app
from app.core.extensions import db
from app.core.config import TestingConfig

@pytest.fixture
def app():
    """Create application for the tests."""
    db_fd, db_path = tempfile.mkstemp()

    app = create_app(TestingConfig)
    app.config['SQLALCHEMY_DATABASE_URI'] = f'sqlite:///{db_path}'

    with app.app_context():
        db.create_all()
        yield app
        db.drop_all()

    os.close(db_fd)
    os.unlink(db_path)

@pytest.fixture
def client(app):
    """Create a test client for the app."""
    return app.test_client()

@pytest.fixture
def runner(app):
    """Create a test runner for the app's Click commands."""
    return app.test_cli_runner()
"#;

    pub const TEST_MAIN: &str = r#"def test_index(client):
    """Test the index endpoint."""
    response = client.get('/')
    assert response.status_code == 200
    data = response.get_json()
    assert data['message'] == '{{project_name}} API is running'
    assert data['version'] == '1.0.0'

def test_health_check(client):
    """Test the health check endpoint."""
    response = client.get('/health')
    assert response.status_code == 200
    data = response.get_json()
    assert data['status'] == 'healthy'
    assert data['service'] == '{{project_name}} API'
"#;

    pub const TEST_AUTH: &str = r#"import json
import pytest

def test_register_user(client):
    """Test user registration."""
    response = client.post('/api/v1/auth/register',
                          data=json.dumps({
                              'email': 'test@example.com',
                              'password': 'testpassword123',
                              'full_name': 'Test User'
                          }),
                          content_type='application/json')

    assert response.status_code == 201
    data = response.get_json()
    assert 'access_token' in data
    assert 'refresh_token' in data
    assert data['token_type'] == 'bearer'

def test_login_user(client):
    """Test user login."""
    # First register a user
    client.post('/api/v1/auth/register',
                data=json.dumps({
                    'email': 'login@example.com',
                    'password': 'testpassword123',
                    'full_name': 'Login User'
                }),
                content_type='application/json')

    # Then login
    response = client.post('/api/v1/auth/login',
                          data=json.dumps({
                              'email': 'login@example.com',
                              'password': 'testpassword123'
                          }),
                          content_type='application/json')

    assert response.status_code == 200
    data = response.get_json()
    assert 'access_token' in data
    assert 'refresh_token' in data

def test_invalid_login(client):
    """Test invalid login."""
    response = client.post('/api/v1/auth/login',
                          data=json.dumps({
                              'email': 'nonexistent@example.com',
                              'password': 'wrongpassword'
                          }),
                          content_type='application/json')

    assert response.status_code == 401
    data = response.get_json()
    assert data['message'] == 'Invalid credentials'

def test_duplicate_registration(client):
    """Test registering with existing email."""
    user_data = {
        'email': 'duplicate@example.com',
        'password': 'testpassword123',
        'full_name': 'Duplicate User'
    }

    # Register first time
    client.post('/api/v1/auth/register',
                data=json.dumps(user_data),
                content_type='application/json')

    # Try to register again
    response = client.post('/api/v1/auth/register',
                          data=json.dumps(user_data),
                          content_type='application/json')

    assert response.status_code == 409
    data = response.get_json()
    assert data['message'] == 'User already exists'
"#;

    // New structured logging module for Flask
    pub const FLASK_LOGGING_PY: &str = r#"from __future__ import annotations

import logging
import logging.config
import structlog
from typing import Dict, Any
from flask import Flask, has_request_context, g

def setup_logging(app: Flask) -> None:
    """Configure structured logging for Flask with structlog"""

    log_level = app.config.get('LOG_LEVEL', 'INFO')
    log_format = app.config.get('LOG_FORMAT', 'json')

    # Configure structlog
    if log_format == 'json':
        processors = [
            structlog.stdlib.filter_by_level,
            structlog.stdlib.add_logger_name,
            structlog.stdlib.add_log_level,
            structlog.stdlib.PositionalArgumentsFormatter(),
            structlog.processors.TimeStamper(fmt='iso'),
            structlog.processors.StackInfoRenderer(),
            structlog.processors.format_exc_info,
            structlog.processors.UnicodeDecoder(),
            add_flask_context,
            structlog.processors.JSONRenderer()
        ]
    else:
        processors = [
            structlog.stdlib.filter_by_level,
            structlog.stdlib.add_logger_name,
            structlog.stdlib.add_log_level,
            structlog.stdlib.PositionalArgumentsFormatter(),
            structlog.processors.TimeStamper(fmt='%Y-%m-%d %H:%M:%S'),
            structlog.processors.StackInfoRenderer(),
            structlog.processors.format_exc_info,
            add_flask_context,
            structlog.dev.ConsoleRenderer()
        ]

    structlog.configure(
        processors=processors,
        context_class=dict,
        logger_factory=structlog.stdlib.LoggerFactory(),
        wrapper_class=structlog.stdlib.BoundLogger,
        cache_logger_on_first_use=True,
    )

    # Configure standard library logging
    logging.basicConfig(
        format='%(message)s',
        level=getattr(logging, log_level.upper()),
        force=True,
    )

    # Disable werkzeug logs in production
    if not app.debug:
        logging.getLogger('werkzeug').setLevel(logging.WARNING)

def add_flask_context(logger, method_name: str, event_dict: Dict[str, Any]) -> Dict[str, Any]:
    """Add Flask request context to log events"""
    if has_request_context():
        event_dict['request_id'] = getattr(g, 'request_id', None)
    return event_dict
"#;
}

pub mod php {
    // Laravel Templates with Clean Architecture
    pub const LARAVEL_COMPOSER_JSON: &str = r#"{
    "name": "{{kebab_case}}/{{kebab_case}}",
    "type": "project",
    "description": "{{project_name}} - Laravel application with Clean Architecture",
    "keywords": ["laravel", "clean-architecture", "ddd", "api"],
    "license": "MIT",
    "require": {
        "php": "^8.2",
        "laravel/framework": "^11.0",
        "laravel/sanctum": "^4.0",
        "tymon/jwt-auth": "^2.0",
        "guzzlehttp/guzzle": "^7.2",
        "spatie/laravel-data": "^4.0",
        "spatie/laravel-query-builder": "^5.0"
    },
    "require-dev": {
        "fakerphp/faker": "^1.23",
        "laravel/pint": "^1.0",
        "laravel/sail": "^1.18",
        "mockery/mockery": "^1.4.4",
        "nunomaduro/collision": "^8.0",
        "phpunit/phpunit": "^10.0",
        "spatie/laravel-ignition": "^2.0",
        "larastan/larastan": "^2.0"
    },
    "autoload": {
        "psr-4": {
            "App\\": "app/",
            "Database\\Factories\\": "database/factories/",
            "Database\\Seeders\\": "database/seeders/"
        }
    },
    "autoload-dev": {
        "psr-4": {
            "Tests\\": "tests/"
        }
    },
    "scripts": {
        "post-autoload-dump": [
            "Illuminate\\Foundation\\ComposerScripts::postAutoloadDump",
            "@php artisan package:discover --ansi"
        ],
        "post-update-cmd": [
            "@php artisan vendor:publish --tag=laravel-assets --ansi --force"
        ],
        "post-root-package-install": [
            "@php -r \"file_exists('.env') || copy('.env.example', '.env');\""
        ],
        "post-create-project-cmd": [
            "@php artisan key:generate --ansi"
        ],
        "test": "phpunit",
        "test-coverage": "phpunit --coverage-html coverage",
        "pint": "pint",
        "stan": "phpstan analyse"
    },
    "extra": {
        "laravel": {
            "dont-discover": []
        }
    },
    "config": {
        "optimize-autoloader": true,
        "preferred-install": "dist",
        "sort-packages": true,
        "allow-plugins": {
            "pestphp/pest-plugin": true,
            "php-http/discovery": true
        }
    },
    "minimum-stability": "stable",
    "prefer-stable": true
}
"#;

    pub const LARAVEL_ARTISAN: &str = r#"#!/usr/bin/env php
<?php

define('LARAVEL_START', microtime(true));

// Register the Composer autoloader
require __DIR__.'/vendor/autoload.php';

// Bootstrap Laravel and handle the command
$app = require_once __DIR__.'/bootstrap/app.php';

$kernel = $app->make(Illuminate\Contracts\Console\Kernel::class);

$status = $kernel->handle(
    $input = new Symfony\Component\Console\Input\ArgvInput,
    new Symfony\Component\Console\Output\ConsoleOutput
);

$kernel->terminate($input, $status);

exit($status);
"#;

    pub const LARAVEL_APP_CONFIG: &str = r#"<?php

use Illuminate\Support\Facades\Facade;
use Illuminate\Support\ServiceProvider;

return [

    /*
    |--------------------------------------------------------------------------
    | Application Name
    |--------------------------------------------------------------------------
    */

    'name' => env('APP_NAME', '{{project_name}}'),

    /*
    |--------------------------------------------------------------------------
    | Application Environment
    |--------------------------------------------------------------------------
    */

    'env' => env('APP_ENV', 'production'),

    /*
    |--------------------------------------------------------------------------
    | Application Debug Mode
    |--------------------------------------------------------------------------
    */

    'debug' => (bool) env('APP_DEBUG', false),

    /*
    |--------------------------------------------------------------------------
    | Application URL
    |--------------------------------------------------------------------------
    */

    'url' => env('APP_URL', 'http://localhost'),

    'asset_url' => env('ASSET_URL'),

    /*
    |--------------------------------------------------------------------------
    | Application Timezone
    |--------------------------------------------------------------------------
    */

    'timezone' => 'UTC',

    /*
    |--------------------------------------------------------------------------
    | Application Locale Configuration
    |--------------------------------------------------------------------------
    */

    'locale' => 'en',

    'fallback_locale' => 'en',

    'faker_locale' => 'en_US',

    /*
    |--------------------------------------------------------------------------
    | Encryption Key
    |--------------------------------------------------------------------------
    */

    'key' => env('APP_KEY'),

    'cipher' => 'AES-256-CBC',

    /*
    |--------------------------------------------------------------------------
    | Maintenance Mode Driver
    |--------------------------------------------------------------------------
    */

    'maintenance' => [
        'driver' => 'file',
    ],

    /*
    |--------------------------------------------------------------------------
    | Autoloaded Service Providers
    |--------------------------------------------------------------------------
    */

    'providers' => ServiceProvider::defaultProviders()->merge([
        /*
         * Package Service Providers...
         */
        Tymon\JWTAuth\Providers\LaravelServiceProvider::class,

        /*
         * Application Service Providers...
         */
        App\Infrastructure\Providers\AppServiceProvider::class,
        App\Infrastructure\Providers\AuthServiceProvider::class,
        App\Infrastructure\Providers\EventServiceProvider::class,
        App\Infrastructure\Providers\RouteServiceProvider::class,
    ])->toArray(),

    /*
    |--------------------------------------------------------------------------
    | Class Aliases
    |--------------------------------------------------------------------------
    */

    'aliases' => Facade::defaultAliases()->merge([
        'JWTAuth' => Tymon\JWTAuth\Facades\JWTAuth::class,
        'JWTFactory' => Tymon\JWTAuth\Facades\JWTFactory::class,
    ])->toArray(),

];
"#;

    pub const LARAVEL_DATABASE_CONFIG: &str = r#"<?php

use Illuminate\Support\Str;

return [

    /*
    |--------------------------------------------------------------------------
    | Default Database Connection Name
    |--------------------------------------------------------------------------
    */

    'default' => env('DB_CONNECTION', 'mysql'),

    /*
    |--------------------------------------------------------------------------
    | Database Connections
    |--------------------------------------------------------------------------
    */

    'connections' => [

        'sqlite' => [
            'driver' => 'sqlite',
            'url' => env('DATABASE_URL'),
            'database' => env('DB_DATABASE', database_path('database.sqlite')),
            'prefix' => '',
            'foreign_key_constraints' => env('DB_FOREIGN_KEYS', true),
        ],

        'mysql' => [
            'driver' => 'mysql',
            'url' => env('DATABASE_URL'),
            'host' => env('DB_HOST', '127.0.0.1'),
            'port' => env('DB_PORT', '3306'),
            'database' => env('DB_DATABASE', '{{snake_case}}_db'),
            'username' => env('DB_USERNAME', 'forge'),
            'password' => env('DB_PASSWORD', ''),
            'unix_socket' => env('DB_SOCKET', ''),
            'charset' => 'utf8mb4',
            'collation' => 'utf8mb4_unicode_ci',
            'prefix' => '',
            'prefix_indexes' => true,
            'strict' => true,
            'engine' => null,
            'options' => extension_loaded('pdo_mysql') ? array_filter([
                PDO::MYSQL_ATTR_SSL_CA => env('MYSQL_ATTR_SSL_CA'),
            ]) : [],
        ],

        'pgsql' => [
            'driver' => 'pgsql',
            'url' => env('DATABASE_URL'),
            'host' => env('DB_HOST', '127.0.0.1'),
            'port' => env('DB_PORT', '5432'),
            'database' => env('DB_DATABASE', '{{snake_case}}_db'),
            'username' => env('DB_USERNAME', 'forge'),
            'password' => env('DB_PASSWORD', ''),
            'charset' => 'utf8',
            'prefix' => '',
            'prefix_indexes' => true,
            'search_path' => 'public',
            'sslmode' => 'prefer',
        ],

    ],

    /*
    |--------------------------------------------------------------------------
    | Migration Repository Table
    |--------------------------------------------------------------------------
    */

    'migrations' => 'migrations',

    /*
    |--------------------------------------------------------------------------
    | Redis Databases
    |--------------------------------------------------------------------------
    */

    'redis' => [

        'client' => env('REDIS_CLIENT', 'phpredis'),

        'options' => [
            'cluster' => env('REDIS_CLUSTER', 'redis'),
            'prefix' => env('REDIS_PREFIX', Str::slug(env('APP_NAME', 'laravel'), '_').'_database_'),
        ],

        'default' => [
            'url' => env('REDIS_URL'),
            'host' => env('REDIS_HOST', '127.0.0.1'),
            'password' => env('REDIS_PASSWORD'),
            'port' => env('REDIS_PORT', '6379'),
            'database' => env('REDIS_DB', '0'),
        ],

        'cache' => [
            'url' => env('REDIS_URL'),
            'host' => env('REDIS_HOST', '127.0.0.1'),
            'password' => env('REDIS_PASSWORD'),
            'port' => env('REDIS_PORT', '6379'),
            'database' => env('REDIS_CACHE_DB', '1'),
        ],

    ],

];
"#;

    pub const LARAVEL_AUTH_CONFIG: &str = r#"<?php

return [

    /*
    |--------------------------------------------------------------------------
    | Authentication Defaults
    |--------------------------------------------------------------------------
    */

    'defaults' => [
        'guard' => 'api',
        'passwords' => 'users',
    ],

    /*
    |--------------------------------------------------------------------------
    | Authentication Guards
    |--------------------------------------------------------------------------
    */

    'guards' => [
        'web' => [
            'driver' => 'session',
            'provider' => 'users',
        ],

        'api' => [
            'driver' => 'jwt',
            'provider' => 'users',
        ],
    ],

    /*
    |--------------------------------------------------------------------------
    | User Providers
    |--------------------------------------------------------------------------
    */

    'providers' => [
        'users' => [
            'driver' => 'eloquent',
            'model' => App\Domain\User\Entities\User::class,
        ],
    ],

    /*
    |--------------------------------------------------------------------------
    | Resetting Passwords
    |--------------------------------------------------------------------------
    */

    'passwords' => [
        'users' => [
            'provider' => 'users',
            'table' => 'password_reset_tokens',
            'expire' => 60,
            'throttle' => 60,
        ],
    ],

    /*
    |--------------------------------------------------------------------------
    | Password Confirmation Timeout
    |--------------------------------------------------------------------------
    */

    'password_timeout' => 10800,

];
"#;

    // User Entity (Domain Layer)
    pub const LARAVEL_USER_ENTITY: &str = r#"<?php

declare(strict_types=1);

namespace App\Domain\User\Entities;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Foundation\Auth\User as Authenticatable;
use Illuminate\Notifications\Notifiable;
use Laravel\Sanctum\HasApiTokens;
use Tymon\JWTAuth\Contracts\JWTSubject;

class User extends Authenticatable implements JWTSubject
{
    use HasApiTokens, HasFactory, Notifiable;

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'name',
        'email',
        'password',
        'email_verified_at',
    ];

    /**
     * The attributes that should be hidden for serialization.
     *
     * @var array<int, string>
     */
    protected $hidden = [
        'password',
        'remember_token',
    ];

    /**
     * Get the attributes that should be cast.
     *
     * @return array<string, string>
     */
    protected function casts(): array
    {
        return [
            'email_verified_at' => 'datetime',
            'password' => 'hashed',
        ];
    }

    /**
     * Get the identifier that will be stored in the subject claim of the JWT.
     */
    public function getJWTIdentifier()
    {
        return $this->getKey();
    }

    /**
     * Return a key value array, containing any custom claims to be added to the JWT.
     */
    public function getJWTCustomClaims(): array
    {
        return [];
    }

    /**
     * Business logic methods
     */
    public function isEmailVerified(): bool
    {
        return !is_null($this->email_verified_at);
    }

    public function markEmailAsVerified(): void
    {
        if (is_null($this->email_verified_at)) {
            $this->email_verified_at = now();
            $this->save();
        }
    }

    public function getDisplayName(): string
    {
        return $this->name ?? $this->email;
    }
}
"#;

    // User Repository Interface (Domain Layer)
    pub const LARAVEL_USER_REPOSITORY: &str = r#"<?php

declare(strict_types=1);

namespace App\Domain\User\Repositories;

use App\Domain\User\Entities\User;
use Illuminate\Contracts\Pagination\LengthAwarePaginator;

interface UserRepositoryInterface
{
    public function findById(int $id): ?User;

    public function findByEmail(string $email): ?User;

    public function create(array $data): User;

    public function update(User $user, array $data): User;

    public function delete(User $user): bool;

    public function paginate(int $perPage = 15): LengthAwarePaginator;

    public function existsByEmail(string $email): bool;
}
"#;

    // User Service (Domain Layer)
    pub const LARAVEL_USER_SERVICE: &str = r#"<?php

declare(strict_types=1);

namespace App\Domain\User\Services;

use App\Domain\User\Entities\User;
use App\Domain\User\Repositories\UserRepositoryInterface;
use Illuminate\Support\Facades\Hash;
use Illuminate\Validation\ValidationException;

class UserService
{
    public function __construct(
        private readonly UserRepositoryInterface $userRepository
    ) {}

    public function createUser(array $userData): User
    {
        if ($this->userRepository->existsByEmail($userData['email'])) {
            throw ValidationException::withMessages([
                'email' => ['A user with this email already exists.']
            ]);
        }

        $userData['password'] = Hash::make($userData['password']);

        return $this->userRepository->create($userData);
    }

    public function updateUser(User $user, array $userData): User
    {
        if (isset($userData['email']) &&
            $userData['email'] !== $user->email &&
            $this->userRepository->existsByEmail($userData['email'])) {
            throw ValidationException::withMessages([
                'email' => ['A user with this email already exists.']
            ]);
        }

        if (isset($userData['password'])) {
            $userData['password'] = Hash::make($userData['password']);
        }

        return $this->userRepository->update($user, $userData);
    }

    public function getUserById(int $id): ?User
    {
        return $this->userRepository->findById($id);
    }

    public function getUserByEmail(string $email): ?User
    {
        return $this->userRepository->findByEmail($email);
    }

    public function deleteUser(User $user): bool
    {
        return $this->userRepository->delete($user);
    }

    public function getPaginatedUsers(int $perPage = 15)
    {
        return $this->userRepository->paginate($perPage);
    }
}
"#;

    // Auth Service (Domain Layer)
    pub const LARAVEL_AUTH_SERVICE: &str = r#"<?php

declare(strict_types=1);

namespace App\Domain\Auth\Services;

use App\Domain\User\Entities\User;
use App\Domain\User\Repositories\UserRepositoryInterface;
use Illuminate\Support\Facades\Hash;
use Illuminate\Validation\ValidationException;
use Tymon\JWTAuth\Facades\JWTAuth;

class AuthService
{
    public function __construct(
        private readonly UserRepositoryInterface $userRepository
    ) {}

    public function login(string $email, string $password): array
    {
        $user = $this->userRepository->findByEmail($email);

        if (!$user || !Hash::check($password, $user->password)) {
            throw ValidationException::withMessages([
                'email' => ['The provided credentials are incorrect.']
            ]);
        }

        $token = JWTAuth::fromUser($user);

        return [
            'access_token' => $token,
            'token_type' => 'bearer',
            'expires_in' => config('jwt.ttl') * 60,
            'user' => $user->only(['id', 'name', 'email', 'email_verified_at'])
        ];
    }

    public function register(array $userData): array
    {
        if ($this->userRepository->existsByEmail($userData['email'])) {
            throw ValidationException::withMessages([
                'email' => ['A user with this email already exists.']
            ]);
        }

        $userData['password'] = Hash::make($userData['password']);
        $user = $this->userRepository->create($userData);

        $token = JWTAuth::fromUser($user);

        return [
            'access_token' => $token,
            'token_type' => 'bearer',
            'expires_in' => config('jwt.ttl') * 60,
            'user' => $user->only(['id', 'name', 'email', 'email_verified_at'])
        ];
    }

    public function logout(): void
    {
        JWTAuth::invalidate(JWTAuth::getToken());
    }

    public function refresh(): string
    {
        return JWTAuth::refresh(JWTAuth::getToken());
    }

    public function me(): User
    {
        return JWTAuth::parseToken()->authenticate();
    }
}
"#;

    // Create User Command (Application Layer)
    pub const LARAVEL_CREATE_USER_COMMAND: &str = r#"<?php

declare(strict_types=1);

namespace App\Application\User\Commands;

class CreateUserCommand
{
    public function __construct(
        public readonly string $name,
        public readonly string $email,
        public readonly string $password,
        public readonly ?string $emailVerifiedAt = null
    ) {}

    public function toArray(): array
    {
        return [
            'name' => $this->name,
            'email' => $this->email,
            'password' => $this->password,
            'email_verified_at' => $this->emailVerifiedAt,
        ];
    }
}
"#;

    // Get User Query (Application Layer)
    pub const LARAVEL_GET_USER_QUERY: &str = r#"<?php

declare(strict_types=1);

namespace App\Application\User\Queries;

class GetUserQuery
{
    public function __construct(
        public readonly int $userId
    ) {}
}
"#;

    // User Handler (Application Layer)
    pub const LARAVEL_USER_HANDLER: &str = r#"<?php

declare(strict_types=1);

namespace App\Application\User\Handlers;

use App\Application\User\Commands\CreateUserCommand;
use App\Application\User\Queries\GetUserQuery;
use App\Domain\User\Entities\User;
use App\Domain\User\Services\UserService;

class UserHandler
{
    public function __construct(
        private readonly UserService $userService
    ) {}

    public function handleCreateUser(CreateUserCommand $command): User
    {
        return $this->userService->createUser($command->toArray());
    }

    public function handleGetUser(GetUserQuery $query): ?User
    {
        return $this->userService->getUserById($query->userId);
    }
}
"#;

    // Login Command (Application Layer)
    pub const LARAVEL_LOGIN_COMMAND: &str = r#"<?php

declare(strict_types=1);

namespace App\Application\Auth\Commands;

class LoginCommand
{
    public function __construct(
        public readonly string $email,
        public readonly string $password
    ) {}
}
"#;

    // Auth Handler (Application Layer)
    pub const LARAVEL_AUTH_HANDLER: &str = r#"<?php

declare(strict_types=1);

namespace App\Application\Auth\Handlers;

use App\Application\Auth\Commands\LoginCommand;
use App\Application\User\Commands\CreateUserCommand;
use App\Domain\Auth\Services\AuthService;

class AuthHandler
{
    public function __construct(
        private readonly AuthService $authService
    ) {}

    public function handleLogin(LoginCommand $command): array
    {
        return $this->authService->login($command->email, $command->password);
    }

    public function handleRegister(CreateUserCommand $command): array
    {
        return $this->authService->register($command->toArray());
    }

    public function handleLogout(): void
    {
        $this->authService->logout();
    }

    public function handleRefresh(): string
    {
        return $this->authService->refresh();
    }

    public function handleMe()
    {
        return $this->authService->me();
    }
}
"#;

    // Eloquent User Repository (Infrastructure Layer)
    pub const LARAVEL_ELOQUENT_USER_REPOSITORY: &str = r#"<?php

declare(strict_types=1);

namespace App\Infrastructure\Persistence\Eloquent;

use App\Domain\User\Entities\User;
use App\Domain\User\Repositories\UserRepositoryInterface;
use Illuminate\Contracts\Pagination\LengthAwarePaginator;

class EloquentUserRepository implements UserRepositoryInterface
{
    public function findById(int $id): ?User
    {
        return User::find($id);
    }

    public function findByEmail(string $email): ?User
    {
        return User::where('email', $email)->first();
    }

    public function create(array $data): User
    {
        return User::create($data);
    }

    public function update(User $user, array $data): User
    {
        $user->update($data);
        return $user->fresh();
    }

    public function delete(User $user): bool
    {
        return $user->delete();
    }

    public function paginate(int $perPage = 15): LengthAwarePaginator
    {
        return User::paginate($perPage);
    }

    public function existsByEmail(string $email): bool
    {
        return User::where('email', $email)->exists();
    }
}
"#;

    // User Controller (Infrastructure Layer)
    pub const LARAVEL_USER_CONTROLLER: &str = r#"<?php

declare(strict_types=1);

namespace App\Infrastructure\Http\Controllers\Api\V1;

use App\Application\User\Commands\CreateUserCommand;
use App\Application\User\Handlers\UserHandler;
use App\Application\User\Queries\GetUserQuery;
use App\Infrastructure\Http\Controllers\Controller;
use App\Infrastructure\Http\Requests\CreateUserRequest;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class UserController extends Controller
{
    public function __construct(
        private readonly UserHandler $userHandler
    ) {}

    public function index(): JsonResponse
    {
        // Implementation for listing users with pagination
        return response()->json(['message' => 'Users list']);
    }

    public function show(int $id): JsonResponse
    {
        $query = new GetUserQuery($id);
        $user = $this->userHandler->handleGetUser($query);

        if (!$user) {
            return response()->json(['message' => 'User not found'], 404);
        }

        return response()->json([
            'data' => $user->only(['id', 'name', 'email', 'email_verified_at'])
        ]);
    }

    public function store(CreateUserRequest $request): JsonResponse
    {
        $command = new CreateUserCommand(
            name: $request->validated('name'),
            email: $request->validated('email'),
            password: $request->validated('password')
        );

        $user = $this->userHandler->handleCreateUser($command);

        return response()->json([
            'message' => 'User created successfully',
            'data' => $user->only(['id', 'name', 'email', 'email_verified_at'])
        ], 201);
    }
}
"#;

    // Auth Controller (Infrastructure Layer)
    pub const LARAVEL_AUTH_CONTROLLER: &str = r#"<?php

declare(strict_types=1);

namespace App\Infrastructure\Http\Controllers\Api\V1;

use App\Application\Auth\Commands\LoginCommand;
use App\Application\Auth\Handlers\AuthHandler;
use App\Application\User\Commands\CreateUserCommand;
use App\Infrastructure\Http\Controllers\Controller;
use App\Infrastructure\Http\Requests\LoginRequest;
use App\Infrastructure\Http\Requests\RegisterRequest;
use Illuminate\Http\JsonResponse;
use Illuminate\Validation\ValidationException;

class AuthController extends Controller
{
    public function __construct(
        private readonly AuthHandler $authHandler
    ) {}

    public function register(RegisterRequest $request): JsonResponse
    {
        try {
            $command = new CreateUserCommand(
                name: $request->validated('name'),
                email: $request->validated('email'),
                password: $request->validated('password')
            );

            $result = $this->authHandler->handleRegister($command);

            return response()->json([
                'message' => 'User registered successfully',
                'data' => $result
            ], 201);
        } catch (ValidationException $e) {
            return response()->json([
                'message' => 'Validation failed',
                'errors' => $e->errors()
            ], 422);
        }
    }

    public function login(LoginRequest $request): JsonResponse
    {
        try {
            $command = new LoginCommand(
                email: $request->validated('email'),
                password: $request->validated('password')
            );

            $result = $this->authHandler->handleLogin($command);

            return response()->json([
                'message' => 'Login successful',
                'data' => $result
            ]);
        } catch (ValidationException $e) {
            return response()->json([
                'message' => 'Invalid credentials',
                'errors' => $e->errors()
            ], 401);
        }
    }

    public function logout(): JsonResponse
    {
        $this->authHandler->handleLogout();

        return response()->json([
            'message' => 'Successfully logged out'
        ]);
    }

    public function refresh(): JsonResponse
    {
        $token = $this->authHandler->handleRefresh();

        return response()->json([
            'access_token' => $token,
            'token_type' => 'bearer',
            'expires_in' => config('jwt.ttl') * 60
        ]);
    }

    public function me(): JsonResponse
    {
        $user = $this->authHandler->handleMe();

        return response()->json([
            'data' => $user->only(['id', 'name', 'email', 'email_verified_at'])
        ]);
    }
}
"#;

    // Register Request (Infrastructure Layer)
    pub const LARAVEL_REGISTER_REQUEST: &str = r#"<?php

declare(strict_types=1);

namespace App\Infrastructure\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class RegisterRequest extends FormRequest
{
    public function authorize(): bool
    {
        return true;
    }

    public function rules(): array
    {
        return [
            'name' => ['required', 'string', 'max:255'],
            'email' => ['required', 'string', 'email', 'max:255', 'unique:users'],
            'password' => ['required', 'string', 'min:8', 'confirmed'],
        ];
    }

    public function messages(): array
    {
        return [
            'name.required' => 'Name is required',
            'email.required' => 'Email is required',
            'email.email' => 'Email must be a valid email address',
            'email.unique' => 'This email is already registered',
            'password.required' => 'Password is required',
            'password.min' => 'Password must be at least 8 characters',
            'password.confirmed' => 'Password confirmation does not match',
        ];
    }
}
"#;

    // Login Request (Infrastructure Layer)
    pub const LARAVEL_LOGIN_REQUEST: &str = r#"<?php

declare(strict_types=1);

namespace App\Infrastructure\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class LoginRequest extends FormRequest
{
    public function authorize(): bool
    {
        return true;
    }

    public function rules(): array
    {
        return [
            'email' => ['required', 'string', 'email'],
            'password' => ['required', 'string'],
        ];
    }

    public function messages(): array
    {
        return [
            'email.required' => 'Email is required',
            'email.email' => 'Email must be a valid email address',
            'password.required' => 'Password is required',
        ];
    }
}
"#;

    // API Routes
    pub const LARAVEL_API_ROUTES: &str = r#"<?php

use App\Infrastructure\Http\Controllers\Api\V1\AuthController;
use App\Infrastructure\Http\Controllers\Api\V1\UserController;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Route;

/*
|--------------------------------------------------------------------------
| API Routes
|--------------------------------------------------------------------------
*/

Route::get('/health', function () {
    return response()->json([
        'status' => 'healthy',
        'service' => '{{project_name}} API',
        'version' => '1.0.0',
        'timestamp' => now()->toISOString()
    ]);
});

Route::prefix('v1')->group(function () {
    // Authentication routes
    Route::prefix('auth')->group(function () {
        Route::post('register', [AuthController::class, 'register']);
        Route::post('login', [AuthController::class, 'login']);

        Route::middleware('auth:api')->group(function () {
            Route::post('logout', [AuthController::class, 'logout']);
            Route::post('refresh', [AuthController::class, 'refresh']);
            Route::get('me', [AuthController::class, 'me']);
        });
    });

    // Protected routes
    Route::middleware('auth:api')->group(function () {
        Route::apiResource('users', UserController::class);
    });
});
"#;

    // App Service Provider
    pub const LARAVEL_APP_SERVICE_PROVIDER: &str = r#"<?php

declare(strict_types=1);

namespace App\Infrastructure\Providers;

use App\Domain\User\Repositories\UserRepositoryInterface;
use App\Infrastructure\Persistence\Eloquent\EloquentUserRepository;
use Illuminate\Support\ServiceProvider;

class AppServiceProvider extends ServiceProvider
{
    /**
     * Register any application services.
     */
    public function register(): void
    {
        // Bind repository interfaces to implementations
        $this->app->bind(UserRepositoryInterface::class, EloquentUserRepository::class);
    }

    /**
     * Bootstrap any application services.
     */
    public function boot(): void
    {
        //
    }
}
"#;

    // Multi-stage Dockerfile for PHP
    #[allow(dead_code)]
    pub const PHP_DOCKERFILE: &str = r#"# =========================
# Build stage
# =========================
FROM composer:2.6 AS composer

COPY composer.json composer.lock ./
RUN composer install --no-dev --no-scripts --optimize-autoloader --no-interaction

# =========================
# Runtime stage
# =========================
FROM php:8.2-fpm-alpine AS runtime

# Install system dependencies
RUN apk add --no-cache \
    nginx \
    postgresql-dev \
    libpng-dev \
    libjpeg-turbo-dev \
    freetype-dev \
    zip \
    libzip-dev \
    icu-dev \
    oniguruma-dev \
    curl \
    git \
    supervisor

# Install PHP extensions
RUN docker-php-ext-configure gd --with-freetype --with-jpeg \
    && docker-php-ext-install -j$(nproc) \
        gd \
        pdo \
        pdo_pgsql \
        pdo_mysql \
        zip \
        intl \
        mbstring \
        opcache \
        bcmath

# Install Redis extension
RUN apk add --no-cache $PHPIZE_DEPS \
    && pecl install redis \
    && docker-php-ext-enable redis \
    && apk del $PHPIZE_DEPS

# Create application user
RUN addgroup -g 1000 -S www && \
    adduser -u 1000 -S www -G www

# Set working directory
WORKDIR /var/www/html

# Copy composer dependencies
COPY --from=composer --chown=www:www /app/vendor ./vendor

# Copy application code
COPY --chown=www:www . .

# Copy configuration files
COPY docker/nginx/nginx.conf /etc/nginx/nginx.conf
COPY docker/nginx/default.conf /etc/nginx/http.d/default.conf
COPY docker/php/php-fpm.conf /usr/local/etc/php-fpm.d/www.conf
COPY docker/php/php.ini /usr/local/etc/php/php.ini

# Create necessary directories and set permissions
RUN mkdir -p /var/www/html/storage/logs \
    /var/www/html/storage/framework/cache \
    /var/www/html/storage/framework/sessions \
    /var/www/html/storage/framework/views \
    /var/www/html/bootstrap/cache \
    /run/nginx \
    /var/log/supervisor \
    && chown -R www:www /var/www/html/storage \
    && chown -R www:www /var/www/html/bootstrap/cache \
    && chmod -R 775 /var/www/html/storage \
    && chmod -R 775 /var/www/html/bootstrap/cache

# Copy supervisor configuration
COPY docker/supervisor/supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Switch to www user
USER www

# Expose port
EXPOSE 80

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost/health || exit 1

# Start supervisor
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]
"#;

    // Laravel Docker Compose
    pub const LARAVEL_DOCKER_COMPOSE: &str = r#"services:
  app:
    build:
      context: .
      dockerfile: docker/php/Dockerfile
      target: production
    container_name: {{kebab_case}}-app
    env_file:
      - .env.docker
    environment:
      - APP_NAME={{project_name}}
      - DB_CONNECTION=pgsql
      - DB_HOST=postgres
      - DB_PORT=5432
      - DB_DATABASE={{snake_case}}_db
      - REDIS_HOST=redis
      - REDIS_PORT=6379
      - CACHE_DRIVER=redis
      - SESSION_DRIVER=redis
      - QUEUE_CONNECTION=redis
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_started
    volumes:
      - app_data:/var/www/html
      - app_logs:/var/www/html/storage/logs
    networks:
      - {{kebab_case}}-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "php", "artisan", "app:health-check"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  nginx:
    build:
      context: .
      dockerfile: docker/nginx/Dockerfile
    container_name: {{kebab_case}}-nginx
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - app_data:/var/www/html:ro
      - nginx_logs:/var/log/nginx
    depends_on:
      app:
        condition: service_healthy
    networks:
      - {{kebab_case}}-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  postgres:
    image: postgres:16-alpine
    container_name: {{kebab_case}}-postgres
    env_file:
      - .env.docker
    environment:
      - POSTGRES_DB={{snake_case}}_db
      - POSTGRES_USER=postgres
    volumes:
      - postgres_data:/var/lib/postgresql/data
    expose:
      - "5432"
    networks:
      - {{kebab_case}}-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "pg_isready", "-U", "postgres", "-d", "{{snake_case}}_db"]
      interval: 30s
      timeout: 10s
      retries: 5
      start_period: 30s

  redis:
    image: redis:7-alpine
    container_name: {{kebab_case}}-redis
    volumes:
      - redis_data:/data
      - ./docker/redis/redis.conf:/usr/local/etc/redis/redis.conf:ro
    expose:
      - "6379"
    networks:
      - {{kebab_case}}-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 30s
      timeout: 10s
      retries: 3
    command: redis-server /usr/local/etc/redis/redis.conf

volumes:
  postgres_data:
  redis_data:
  app_data:
  app_logs:
  nginx_logs:

networks:
  {{kebab_case}}-network:
    driver: bridge
"#;

    // Nginx Configuration
    pub const PHP_NGINX_CONF: &str = r#"user www;
worker_processes auto;
error_log /var/log/nginx/error.log warn;
pid /run/nginx.pid;

events {
    worker_connections 1024;
    use epoll;
    multi_accept on;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # Logging
    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';

    access_log /var/log/nginx/access.log main;

    # Basic Settings
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;
    client_max_body_size 16M;

    # Gzip Settings
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types
        text/plain
        text/css
        text/xml
        text/javascript
        application/json
        application/javascript
        application/xml+rss
        application/atom+xml;

    # Security Headers
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Include server configurations
    include /etc/nginx/http.d/*.conf;
}
"#;

    // Laravel Nginx Default Configuration
    pub const LARAVEL_NGINX_DEFAULT_CONF: &str = r#"server {
    listen 80;
    server_name _;
    root /var/www/html/public;
    index index.php index.html;

    # Security
    server_tokens off;

    # Logging
    access_log /var/log/nginx/access.log;
    error_log /var/log/nginx/error.log;

    # Laravel-specific configuration
    location / {
        try_files $uri $uri/ /index.php?$query_string;
    }

    # Handle PHP files
    location ~ \.php$ {
        try_files $uri =404;
        fastcgi_split_path_info ^(.+\.php)(/.+)$;
        fastcgi_pass 127.0.0.1:9000;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;

        # FastCGI settings
        fastcgi_connect_timeout 60;
        fastcgi_send_timeout 180;
        fastcgi_read_timeout 180;
        fastcgi_buffer_size 128k;
        fastcgi_buffers 4 256k;
        fastcgi_busy_buffers_size 256k;
        fastcgi_temp_file_write_size 256k;
        fastcgi_intercept_errors on;
    }

    # Deny access to sensitive files
    location ~ /\.(?!well-known).* {
        deny all;
    }

    # Static files caching
    location ~* \.(css|js|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
        try_files $uri =404;
    }

    # Health check endpoint
    location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }
}
"#;

    // PHP-FPM Configuration
    pub const PHP_FPM_CONF: &str = r#"[www]
user = www
group = www

listen = 127.0.0.1:9000
listen.owner = www
listen.group = www
listen.mode = 0660

pm = dynamic
pm.max_children = 20
pm.start_servers = 2
pm.min_spare_servers = 1
pm.max_spare_servers = 3
pm.max_requests = 1000

; Logging
access.log = /var/log/php-fpm.access.log
access.format = "%R - %u %t \"%m %r%Q%q\" %s %f %{mili}d %{kilo}M %C%%"

; Environment variables
env[PATH] = /usr/local/bin:/usr/bin:/bin
env[TMP] = /tmp
env[TMPDIR] = /tmp
env[TEMP] = /tmp

; PHP admin values
php_admin_value[error_log] = /var/log/php-fpm.error.log
php_admin_flag[log_errors] = on
php_admin_value[memory_limit] = 256M
php_admin_value[upload_max_filesize] = 16M
php_admin_value[post_max_size] = 16M
php_admin_value[max_execution_time] = 120
php_admin_value[max_input_time] = 120
"#;

    // Laravel Environment Example
    pub const LARAVEL_ENV_EXAMPLE: &str = r#"APP_NAME={{project_name}}
APP_ENV=local
APP_KEY=
APP_DEBUG=true
APP_URL=http://localhost

LOG_CHANNEL=stack
LOG_DEPRECATIONS_CHANNEL=null
LOG_LEVEL=debug

DB_CONNECTION=pgsql
DB_HOST=127.0.0.1
DB_PORT=5432
DB_DATABASE={{snake_case}}_db
DB_USERNAME=postgres
DB_PASSWORD=

BROADCAST_DRIVER=log
CACHE_DRIVER=redis
FILESYSTEM_DISK=local
QUEUE_CONNECTION=sync
SESSION_DRIVER=redis
SESSION_LIFETIME=120

REDIS_HOST=127.0.0.1
REDIS_PASSWORD=null
REDIS_PORT=6379

MAIL_MAILER=smtp
MAIL_HOST=mailpit
MAIL_PORT=1025
MAIL_USERNAME=null
MAIL_PASSWORD=null
MAIL_ENCRYPTION=null
MAIL_FROM_ADDRESS="hello@example.com"
MAIL_FROM_NAME="${APP_NAME}"

JWT_SECRET=
JWT_TTL=60
JWT_REFRESH_TTL=20160

VITE_APP_NAME="${APP_NAME}"
"#;

    // Laravel PHPUnit Configuration
    pub const LARAVEL_PHPUNIT_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<phpunit xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:noNamespaceSchemaLocation="vendor/phpunit/phpunit/phpunit.xsd"
         bootstrap="vendor/autoload.php"
         colors="true"
>
    <testsuites>
        <testsuite name="Unit">
            <directory>tests/Unit</directory>
        </testsuite>
        <testsuite name="Feature">
            <directory>tests/Feature</directory>
        </testsuite>
        <testsuite name="Integration">
            <directory>tests/Integration</directory>
        </testsuite>
    </testsuites>
    <source>
        <include>
            <directory>app</directory>
        </include>
    </source>
    <php>
        <env name="APP_ENV" value="testing"/>
        <env name="BCRYPT_ROUNDS" value="4"/>
        <env name="CACHE_DRIVER" value="array"/>
        <env name="DB_CONNECTION" value="sqlite"/>
        <env name="DB_DATABASE" value=":memory:"/>
        <env name="MAIL_MAILER" value="array"/>
        <env name="QUEUE_CONNECTION" value="sync"/>
        <env name="SESSION_DRIVER" value="array"/>
        <env name="TELESCOPE_ENABLED" value="false"/>
    </php>
</phpunit>
"#;

    // Laravel Auth Feature Test
    pub const LARAVEL_AUTH_FEATURE_TEST: &str = r#"<?php

declare(strict_types=1);

namespace Tests\Feature\Api;

use App\Domain\User\Entities\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class AuthTest extends TestCase
{
    use RefreshDatabase;

    public function test_user_can_register(): void
    {
        $userData = [
            'name' => 'John Doe',
            'email' => 'john@example.com',
            'password' => 'password123',
            'password_confirmation' => 'password123',
        ];

        $response = $this->postJson('/api/v1/auth/register', $userData);

        $response->assertStatus(201)
            ->assertJsonStructure([
                'message',
                'data' => [
                    'access_token',
                    'token_type',
                    'expires_in',
                    'user' => [
                        'id',
                        'name',
                        'email',
                    ]
                ]
            ]);

        $this->assertDatabaseHas('users', [
            'email' => 'john@example.com',
            'name' => 'John Doe',
        ]);
    }

    public function test_user_can_login_with_valid_credentials(): void
    {
        $user = User::factory()->create([
            'email' => 'john@example.com',
            'password' => bcrypt('password123'),
        ]);

        $loginData = [
            'email' => 'john@example.com',
            'password' => 'password123',
        ];

        $response = $this->postJson('/api/v1/auth/login', $loginData);

        $response->assertStatus(200)
            ->assertJsonStructure([
                'message',
                'data' => [
                    'access_token',
                    'token_type',
                    'expires_in',
                    'user'
                ]
            ]);
    }

    public function test_user_cannot_login_with_invalid_credentials(): void
    {
        $user = User::factory()->create([
            'email' => 'john@example.com',
            'password' => bcrypt('password123'),
        ]);

        $loginData = [
            'email' => 'john@example.com',
            'password' => 'wrongpassword',
        ];

        $response = $this->postJson('/api/v1/auth/login', $loginData);

        $response->assertStatus(401)
            ->assertJson([
                'message' => 'Invalid credentials',
            ]);
    }

    public function test_authenticated_user_can_get_profile(): void
    {
        $user = User::factory()->create();
        $token = auth('api')->login($user);

        $response = $this->withHeaders([
            'Authorization' => 'Bearer ' . $token,
        ])->getJson('/api/v1/auth/me');

        $response->assertStatus(200)
            ->assertJsonStructure([
                'data' => [
                    'id',
                    'name',
                    'email',
                ]
            ]);
    }

    public function test_authenticated_user_can_logout(): void
    {
        $user = User::factory()->create();
        $token = auth('api')->login($user);

        $response = $this->withHeaders([
            'Authorization' => 'Bearer ' . $token,
        ])->postJson('/api/v1/auth/logout');

        $response->assertStatus(200)
            ->assertJson([
                'message' => 'Successfully logged out',
            ]);
    }
}
"#;

    // Laravel User Unit Test
    pub const LARAVEL_USER_UNIT_TEST: &str = r#"<?php

declare(strict_types=1);

namespace Tests\Unit\Domain;

use App\Domain\User\Entities\User;
use Tests\TestCase;

class UserTest extends TestCase
{
    public function test_user_can_check_if_email_is_verified(): void
    {
        $userWithVerifiedEmail = new User([
            'email_verified_at' => now(),
        ]);

        $userWithoutVerifiedEmail = new User([
            'email_verified_at' => null,
        ]);

        $this->assertTrue($userWithVerifiedEmail->isEmailVerified());
        $this->assertFalse($userWithoutVerifiedEmail->isEmailVerified());
    }

    public function test_user_can_get_display_name(): void
    {
        $userWithName = new User([
            'name' => 'John Doe',
            'email' => 'john@example.com',
        ]);

        $userWithoutName = new User([
            'name' => null,
            'email' => 'jane@example.com',
        ]);

        $this->assertEquals('John Doe', $userWithName->getDisplayName());
        $this->assertEquals('jane@example.com', $userWithoutName->getDisplayName());
    }

    public function test_user_can_mark_email_as_verified(): void
    {
        $user = new User([
            'email_verified_at' => null,
        ]);

        $this->assertFalse($user->isEmailVerified());

        $user->markEmailAsVerified();

        $this->assertTrue($user->isEmailVerified());
        $this->assertNotNull($user->email_verified_at);
    }
}
"#;

    // Laravel README
    pub const LARAVEL_README: &str = r#"# {{project_name}}

Production-ready Laravel application with Clean Architecture, Domain-Driven Design (DDD), and comprehensive JWT authentication.

## Architecture

This project follows **Clean Architecture** principles with clear separation of concerns:

### Directory Structure

```
app/
├── Domain/              # Business logic and entities
│   ├── User/
│   │   ├── Entities/    # Domain entities (User.php)
│   │   ├── Repositories/ # Repository interfaces
│   │   └── Services/    # Domain services
│   └── Auth/
│       └── Services/    # Authentication domain services
├── Application/         # Use cases and application logic
│   ├── User/
│   │   ├── Commands/    # Command objects
│   │   ├── Queries/     # Query objects
│   │   └── Handlers/    # Command/Query handlers
│   └── Auth/
│       ├── Commands/
│       └── Handlers/
└── Infrastructure/      # External concerns
    ├── Http/
    │   ├── Controllers/ # API controllers
    │   └── Requests/    # Form requests
    └── Persistence/
        └── Eloquent/    # Repository implementations
```

## Features

- **Clean Architecture** with Domain-Driven Design
- **JWT Authentication** with access tokens
- **Repository Pattern** for data access abstraction
- **Command/Query Separation** (CQRS-lite)
- **Docker** containerization with Nginx + PHP-FPM
- **PostgreSQL** database with migrations
- **Redis** for caching and sessions
- **Comprehensive Testing** (Unit, Feature, Integration)
- **Code Quality** tools (PHPStan, Pint)

## Quick Start

### With Docker (Recommended)

```bash
# Clone and setup
git clone <repository>
cd {{kebab_case}}

# Environment setup
cp .env.example .env
# Edit .env with your configuration

# Build and start containers
docker-compose up --build -d

# Install dependencies
docker-compose exec app composer install

# Generate application key
docker-compose exec app php artisan key:generate

# Generate JWT secret
docker-compose exec app php artisan jwt:secret

# Run migrations
docker-compose exec app php artisan migrate

# The API will be available at http://localhost:8000
```

### Local Development

```bash
# Install dependencies
composer install

# Environment setup
cp .env.example .env
php artisan key:generate
php artisan jwt:secret

# Database setup
php artisan migrate

# Start development server
php artisan serve
```

## API Documentation

### Authentication Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/auth/register` | User registration |
| POST | `/api/v1/auth/login` | User login |
| POST | `/api/v1/auth/logout` | User logout |
| POST | `/api/v1/auth/refresh` | Refresh token |
| GET | `/api/v1/auth/me` | Get current user |

### User Endpoints (Protected)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/users` | List users |
| GET | `/api/v1/users/{id}` | Get user by ID |
| POST | `/api/v1/users` | Create user |

### System Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |

## Testing

```bash
# Run all tests
composer test

# Run with coverage
composer test-coverage

# Run specific test suite
./vendor/bin/phpunit --testsuite=Unit
./vendor/bin/phpunit --testsuite=Feature
./vendor/bin/phpunit --testsuite=Integration
```

## Code Quality

```bash
# Fix code style
composer pint

# Run static analysis
composer stan

# Run all quality checks
composer pint && composer stan && composer test
```

## Docker Services

- **app**: PHP 8.2-FPM with Laravel application
- **nginx**: Nginx web server (reverse proxy)
- **postgres**: PostgreSQL 16 database
- **redis**: Redis for caching and sessions

## Security Features

- JWT token-based authentication
- Password hashing with bcrypt
- CORS configuration
- Security headers (X-Frame-Options, CSP, etc.)
- Input validation and sanitization
- SQL injection prevention with Eloquent ORM

## Environment Variables

Key environment variables to configure:

```env
APP_NAME={{project_name}}
APP_ENV=production
APP_KEY=base64:...
APP_URL=https://yourdomain.com

DB_CONNECTION=pgsql
DB_HOST=postgres
DB_DATABASE={{snake_case}}_db
DB_USERNAME=postgres
DB_PASSWORD=your-secure-password

JWT_SECRET=your-jwt-secret
JWT_TTL=60

REDIS_HOST=redis
REDIS_PORT=6379
```

## Deployment

1. Set up your production environment
2. Configure environment variables
3. Build Docker images
4. Deploy with docker-compose or Kubernetes
5. Run migrations: `php artisan migrate --force`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new features
4. Ensure code quality checks pass
5. Submit a pull request

---

Generated by [Athena CLI](https://github.com/Jeck0v/Athena)
"#;

    // Symfony templates will be added here...
    // For now, we'll add placeholder constants to avoid compilation errors

    pub const SYMFONY_COMPOSER_JSON: &str = r#"{
    "name": "{{kebab_case}}/{{kebab_case}}",
    "type": "project",
    "description": "{{project_name}} - Symfony application with Hexagonal Architecture",
    "keywords": ["symfony", "hexagonal-architecture", "ddd", "api"],
    "license": "MIT",
    "require": {
        "php": "^8.2",
        "symfony/framework-bundle": "^7.0",
        "symfony/security-bundle": "^7.0",
        "symfony/console": "^7.0",
        "symfony/dotenv": "^7.0",
        "symfony/flex": "^2.0",
        "symfony/runtime": "^7.0",
        "symfony/yaml": "^7.0",
        "symfony/maker-bundle": "^1.0",
        "symfony/orm-pack": "^2.0",
        "symfony/validator": "^7.0",
        "symfony/serializer": "^7.0",
        "symfony/property-access": "^7.0",
        "symfony/property-info": "^7.0",
        "lexik/jwt-authentication-bundle": "^3.0",
        "doctrine/orm": "^3.0",
        "doctrine/doctrine-bundle": "^2.0",
        "doctrine/doctrine-migrations-bundle": "^3.0",
        "gesdinet/jwt-refresh-token-bundle": "^1.0",
        "nelmio/cors-bundle": "^2.0",
        "ramsey/uuid": "^4.0",
        "symfony/uid": "^7.0"
    },
    "require-dev": {
        "phpunit/phpunit": "^10.0",
        "symfony/phpunit-bridge": "^7.0",
        "symfony/test-pack": "^1.0",
        "phpstan/phpstan": "^1.0",
        "friendsofphp/php-cs-fixer": "^3.0",
        "doctrine/doctrine-fixtures-bundle": "^3.0"
    },
    "autoload": {
        "psr-4": {
            "App\\": "src/"
        }
    },
    "autoload-dev": {
        "psr-4": {
            "App\\Tests\\": "tests/"
        }
    },
    "scripts": {
        "auto-scripts": {
            "cache:clear": "symfony-cmd",
            "assets:install %PUBLIC_DIR%": "symfony-cmd"
        },
        "post-install-cmd": [
            "@auto-scripts"
        ],
        "post-update-cmd": [
            "@auto-scripts"
        ],
        "test": "phpunit",
        "cs-fix": "php-cs-fixer fix",
        "stan": "phpstan analyse"
    },
    "extra": {
        "symfony": {
            "allow-contrib": false,
            "require": "7.0.*"
        }
    },
    "config": {
        "allow-plugins": {
            "composer/package-versions-deprecated": true,
            "symfony/flex": true,
            "symfony/runtime": true
        },
        "optimize-autoloader": true,
        "preferred-install": "dist",
        "sort-packages": true
    },
    "minimum-stability": "stable",
    "prefer-stable": true
}
"#;

    pub const SYMFONY_SERVICES_YAML: &str = r#"services:
    _defaults:
        autowire: true
        autoconfigure: true
        public: false

    App\:
        resource: '../src/'
        exclude:
            - '../src/DependencyInjection/'
            - '../src/Domain/*/Entities/'
            - '../src/Domain/*/ValueObjects/'
            - '../src/Kernel.php'

    # Domain Services
    App\Domain\User\Repositories\UserRepositoryInterface:
        alias: App\Infrastructure\Persistence\Doctrine\Repositories\DoctrineUserRepository

    # Application Services
    App\Application\User\Services\:
        resource: '../src/Application/User/Services/'
        tags: ['app.application_service']

    # Infrastructure Services
    App\Infrastructure\:
        resource: '../src/Infrastructure/'
        exclude:
            - '../src/Infrastructure/Persistence/Doctrine/Entities/'

    # Controllers
    App\Infrastructure\Http\Controllers\:
        resource: '../src/Infrastructure/Http/Controllers/'
        tags: ['controller.service_arguments']

    # Security
    App\Infrastructure\Security\:
        resource: '../src/Infrastructure/Security/'

    # Event Handlers
    App\Application\User\EventHandlers\:
        resource: '../src/Application/User/EventHandlers/'
        tags: [kernel.event_listener]
"#;

    // Placeholder constants for Symfony (to be implemented)
    pub const SYMFONY_DOCTRINE_CONFIG: &str = r#"doctrine:
    dbal:
        url: '%env(resolve:DATABASE_URL)%'
        driver: 'pdo_{{database_driver}}'
        server_version: '{{database_version}}'
        charset: utf8mb4
        default_table_options:
            charset: utf8mb4
            collate: utf8mb4_unicode_ci

    orm:
        auto_generate_proxy_classes: true
        enable_lazy_ghost_objects: true
        naming_strategy: doctrine.orm.naming_strategy.underscore_number_aware
        auto_mapping: true
        mappings:
            App:
                type: attribute
                is_bundle: false
                dir: '%kernel.project_dir%/src/Infrastructure/Persistence/Doctrine/Entities'
                prefix: 'App\Infrastructure\Persistence\Doctrine\Entities'
                alias: App

when@prod:
    doctrine:
        orm:
            auto_generate_proxy_classes: false
            proxy_dir: '%kernel.build_dir%/doctrine/orm/Proxies'
            query_cache_driver:
                type: pool
                pool: doctrine.query_cache_pool
            result_cache_driver:
                type: pool
                pool: doctrine.result_cache_pool

    framework:
        cache:
            pools:
                doctrine.query_cache_pool:
                    adapter: cache.app
                doctrine.result_cache_pool:
                    adapter: cache.app
"#;
    pub const SYMFONY_SECURITY_CONFIG: &str = r#"security:
    password_hashers:
        App\Infrastructure\Persistence\Doctrine\Entities\User:
            algorithm: auto

    providers:
        app_user_provider:
            entity:
                class: App\Infrastructure\Persistence\Doctrine\Entities\User
                property: email

    firewalls:
        dev:
            pattern: ^/(_(profiler|wdt)|css|images|js)/
            security: false

        api_login:
            pattern: ^/api/auth/login
            stateless: true
            json_login:
                check_path: /api/auth/login
                success_handler: lexik_jwt_authentication.handler.authentication_success
                failure_handler: lexik_jwt_authentication.handler.authentication_failure

        api_register:
            pattern: ^/api/auth/register
            stateless: true
            security: false

        api:
            pattern: ^/api
            stateless: true
            jwt: ~

        main:
            lazy: true
            provider: app_user_provider

    access_control:
        - { path: ^/api/auth, roles: PUBLIC_ACCESS }
        - { path: ^/api/health, roles: PUBLIC_ACCESS }
        - { path: ^/api, roles: IS_AUTHENTICATED_FULLY }

when@test:
    security:
        password_hashers:
            App\Infrastructure\Persistence\Doctrine\Entities\User:
                algorithm: auto
                cost: 4
                time_cost: 3
                memory_cost: 10
"#;
    pub const SYMFONY_USER_ENTITY: &str = r#"<?php

declare(strict_types=1);

namespace App\Domain\User\Entities;

use App\Domain\User\ValueObjects\Email;
use App\Domain\User\ValueObjects\HashedPassword;
use App\Domain\User\ValueObjects\UserId;
use App\Domain\User\ValueObjects\UserName;

final class User
{
    public function __construct(
        private readonly UserId $id,
        private readonly Email $email,
        private readonly UserName $name,
        private readonly HashedPassword $password,
        private bool $isActive = true,
        private readonly \DateTimeImmutable $createdAt = new \DateTimeImmutable(),
        private \DateTimeImmutable $updatedAt = new \DateTimeImmutable()
    ) {
    }

    public function getId(): UserId
    {
        return $this->id;
    }

    public function getEmail(): Email
    {
        return $this->email;
    }

    public function getName(): UserName
    {
        return $this->name;
    }

    public function getPassword(): HashedPassword
    {
        return $this->password;
    }

    public function isActive(): bool
    {
        return $this->isActive;
    }

    public function activate(): void
    {
        $this->isActive = true;
        $this->updatedAt = new \DateTimeImmutable();
    }

    public function deactivate(): void
    {
        $this->isActive = false;
        $this->updatedAt = new \DateTimeImmutable();
    }

    public function getCreatedAt(): \DateTimeImmutable
    {
        return $this->createdAt;
    }

    public function getUpdatedAt(): \DateTimeImmutable
    {
        return $this->updatedAt;
    }
}
"#;
    pub const SYMFONY_USER_REPOSITORY: &str = r#"<?php

declare(strict_types=1);

namespace App\Domain\User\Repositories;

use App\Domain\User\Entities\User;
use App\Domain\User\ValueObjects\Email;
use App\Domain\User\ValueObjects\UserId;

interface UserRepositoryInterface
{
    public function save(User $user): void;

    public function findById(UserId $id): ?User;

    public function findByEmail(Email $email): ?User;

    public function delete(User $user): void;

    public function findAll(int $limit = 20, int $offset = 0): array;

    public function count(): int;
}
"#;
    pub const SYMFONY_USER_SERVICE: &str = r#"<?php

declare(strict_types=1);

namespace App\Application\User\Services;

use App\Domain\User\Entities\User;
use App\Domain\User\Repositories\UserRepositoryInterface;
use App\Domain\User\ValueObjects\Email;
use App\Domain\User\ValueObjects\HashedPassword;
use App\Domain\User\ValueObjects\UserId;
use App\Domain\User\ValueObjects\UserName;
use Symfony\Component\PasswordHasher\Hasher\UserPasswordHasherInterface;
use Symfony\Component\Uid\Uuid;

final class UserService
{
    public function __construct(
        private readonly UserRepositoryInterface $userRepository,
        private readonly UserPasswordHasherInterface $passwordHasher
    ) {
    }

    public function createUser(string $email, string $name, string $plainPassword): User
    {
        $existingUser = $this->userRepository->findByEmail(new Email($email));
        if ($existingUser) {
            throw new \DomainException('User with this email already exists');
        }

        $userId = new UserId(Uuid::v4()->toRfc4122());
        $userEmail = new Email($email);
        $userName = new UserName($name);

        // Create a temporary doctrine entity for password hashing
        $tempDoctrineUser = new \App\Infrastructure\Persistence\Doctrine\Entities\User(
            $userId->getValue(),
            $userEmail->getValue(),
            $userName->getValue(),
            ''
        );

        $hashedPassword = $this->passwordHasher->hashPassword($tempDoctrineUser, $plainPassword);
        $password = new HashedPassword($hashedPassword);

        $user = new User($userId, $userEmail, $userName, $password);

        $this->userRepository->save($user);

        return $user;
    }

    public function getUserById(string $id): ?User
    {
        return $this->userRepository->findById(new UserId($id));
    }

    public function getUserByEmail(string $email): ?User
    {
        return $this->userRepository->findByEmail(new Email($email));
    }

    public function updateUser(User $user): void
    {
        $this->userRepository->save($user);
    }

    public function deleteUser(User $user): void
    {
        $this->userRepository->delete($user);
    }

    public function listUsers(int $limit = 20, int $offset = 0): array
    {
        return $this->userRepository->findAll($limit, $offset);
    }

    public function getTotalUsersCount(): int
    {
        return $this->userRepository->count();
    }
}
"#;
    pub const SYMFONY_AUTH_SERVICE: &str = r#"<?php

declare(strict_types=1);

namespace App\Application\User\Services;

use App\Domain\User\Entities\User;
use App\Domain\User\ValueObjects\Email;
use Lexik\JWTAuthenticationBundle\Services\JWTTokenManagerInterface;
use Symfony\Component\PasswordHasher\Hasher\UserPasswordHasherInterface;

final class AuthService
{
    public function __construct(
        private readonly UserService $userService,
        private readonly UserPasswordHasherInterface $passwordHasher,
        private readonly JWTTokenManagerInterface $jwtManager
    ) {
    }

    public function register(string $email, string $name, string $plainPassword): array
    {
        $user = $this->userService->createUser($email, $name, $plainPassword);

        // Convert to Doctrine entity for JWT token generation
        $doctrineUser = \App\Infrastructure\Persistence\Doctrine\Entities\User::fromDomain($user);
        $token = $this->jwtManager->create($doctrineUser);

        return [
            'user' => $user,
            'token' => $token
        ];
    }

    public function authenticate(string $email, string $plainPassword): array
    {
        $user = $this->userService->getUserByEmail($email);

        if (!$user || !$user->isActive()) {
            throw new \InvalidArgumentException('Invalid credentials');
        }

        // Convert to Doctrine entity for password verification
        $doctrineUser = \App\Infrastructure\Persistence\Doctrine\Entities\User::fromDomain($user);

        if (!$this->passwordHasher->isPasswordValid($doctrineUser, $plainPassword)) {
            throw new \InvalidArgumentException('Invalid credentials');
        }

        $token = $this->jwtManager->create($doctrineUser);

        return [
            'user' => $user,
            'token' => $token
        ];
    }
}
"#;
    pub const SYMFONY_CREATE_USER_COMMAND: &str = r#"<?php

declare(strict_types=1);

namespace App\Application\User\Commands;

final readonly class CreateUserCommand
{
    public function __construct(
        public string $email,
        public string $name,
        public string $password
    ) {
    }
}
"#;
    pub const SYMFONY_GET_USER_QUERY: &str = r#"<?php

declare(strict_types=1);

namespace App\Application\User\Queries;

final readonly class GetUserQuery
{
    public function __construct(
        public string $userId
    ) {
    }
}
"#;
    pub const SYMFONY_USER_HANDLER: &str = r#"<?php

declare(strict_types=1);

namespace App\Application\User\Handlers;

use App\Application\User\Commands\CreateUserCommand;
use App\Application\User\Queries\GetUserQuery;
use App\Application\User\Services\UserService;
use App\Domain\User\Entities\User;

final class UserHandler
{
    public function __construct(
        private readonly UserService $userService
    ) {
    }

    public function handleCreateUser(CreateUserCommand $command): User
    {
        return $this->userService->createUser(
            $command->email,
            $command->name,
            $command->password
        );
    }

    public function handleGetUser(GetUserQuery $query): ?User
    {
        return $this->userService->getUserById($query->userId);
    }
}
"#;
    pub const SYMFONY_LOGIN_COMMAND: &str = r#"<?php

declare(strict_types=1);

namespace App\Application\User\Commands;

final readonly class LoginCommand
{
    public function __construct(
        public string $email,
        public string $password
    ) {
    }
}
"#;
    pub const SYMFONY_AUTH_HANDLER: &str = r#"<?php

declare(strict_types=1);

namespace App\Application\User\Handlers;

use App\Application\User\Commands\CreateUserCommand;
use App\Application\User\Commands\LoginCommand;
use App\Application\User\Services\AuthService;

final class AuthHandler
{
    public function __construct(
        private readonly AuthService $authService
    ) {
    }

    public function handleLogin(LoginCommand $command): array
    {
        return $this->authService->authenticate($command->email, $command->password);
    }

    public function handleRegister(CreateUserCommand $command): array
    {
        return $this->authService->register(
            $command->email,
            $command->name,
            $command->password
        );
    }
}
"#;
    pub const SYMFONY_DOCTRINE_USER_REPOSITORY: &str = r#"<?php

declare(strict_types=1);

namespace App\Infrastructure\Persistence\Doctrine\Repositories;

use App\Domain\User\Entities\User;
use App\Domain\User\Repositories\UserRepositoryInterface;
use App\Domain\User\ValueObjects\Email;
use App\Domain\User\ValueObjects\UserId;
use App\Infrastructure\Persistence\Doctrine\Entities\User as DoctrineUser;
use Doctrine\Bundle\DoctrineBundle\Repository\ServiceEntityRepository;
use Doctrine\Persistence\ManagerRegistry;

final class DoctrineUserRepository extends ServiceEntityRepository implements UserRepositoryInterface
{
    public function __construct(ManagerRegistry $registry)
    {
        parent::__construct($registry, DoctrineUser::class);
    }

    public function save(User $user): void
    {
        $doctrineUser = $this->findDoctrineUserById($user->getId()->getValue());

        if ($doctrineUser) {
            // Update existing
            $doctrineUser = DoctrineUser::fromDomain($user);
        } else {
            // Create new
            $doctrineUser = DoctrineUser::fromDomain($user);
            $this->getEntityManager()->persist($doctrineUser);
        }

        $this->getEntityManager()->flush();
    }

    public function findById(UserId $id): ?User
    {
        $doctrineUser = $this->findDoctrineUserById($id->getValue());

        return $doctrineUser ? $doctrineUser->toDomain() : null;
    }

    public function findByEmail(Email $email): ?User
    {
        $doctrineUser = $this->findOneBy(['email' => $email->getValue()]);

        return $doctrineUser ? $doctrineUser->toDomain() : null;
    }

    public function delete(User $user): void
    {
        $doctrineUser = $this->findDoctrineUserById($user->getId()->getValue());

        if ($doctrineUser) {
            $this->getEntityManager()->remove($doctrineUser);
            $this->getEntityManager()->flush();
        }
    }

    public function findAll(int $limit = 20, int $offset = 0): array
    {
        $doctrineUsers = $this->createQueryBuilder('u')
            ->setMaxResults($limit)
            ->setFirstResult($offset)
            ->orderBy('u.createdAt', 'DESC')
            ->getQuery()
            ->getResult();

        return array_map(fn(DoctrineUser $user) => $user->toDomain(), $doctrineUsers);
    }

    public function count(): int
    {
        return $this->createQueryBuilder('u')
            ->select('COUNT(u.id)')
            ->getQuery()
            ->getSingleScalarResult();
    }

    private function findDoctrineUserById(string $id): ?DoctrineUser
    {
        return $this->find($id);
    }
}
"#;
    #[allow(dead_code)]
    pub const SYMFONY_DOCTRINE_USER_ENTITY: &str = r#"<?php

declare(strict_types=1);

namespace App\Infrastructure\Persistence\Doctrine\Entities;

use App\Domain\User\Entities\User as DomainUser;
use App\Domain\User\ValueObjects\Email;
use App\Domain\User\ValueObjects\HashedPassword;
use App\Domain\User\ValueObjects\UserId;
use App\Domain\User\ValueObjects\UserName;
use Doctrine\ORM\Mapping as ORM;
use Symfony\Component\Security\Core\User\PasswordAuthenticatedUserInterface;
use Symfony\Component\Security\Core\User\UserInterface;
use Symfony\Component\Uid\Uuid;

#[ORM\Entity]
#[ORM\Table(name: 'users')]
class User implements UserInterface, PasswordAuthenticatedUserInterface
{
    #[ORM\Id]
    #[ORM\Column(type: 'uuid', unique: true)]
    private string $id;

    #[ORM\Column(type: 'string', length: 180, unique: true)]
    private string $email;

    #[ORM\Column(type: 'string')]
    private string $name;

    #[ORM\Column(type: 'string')]
    private string $password;

    #[ORM\Column(type: 'json')]
    private array $roles = [];

    #[ORM\Column(type: 'boolean')]
    private bool $isActive = true;

    #[ORM\Column(type: 'datetime_immutable')]
    private \DateTimeImmutable $createdAt;

    #[ORM\Column(type: 'datetime_immutable')]
    private \DateTimeImmutable $updatedAt;

    public function __construct(string $id, string $email, string $name, string $password)
    {
        $this->id = $id;
        $this->email = $email;
        $this->name = $name;
        $this->password = $password;
        $this->createdAt = new \DateTimeImmutable();
        $this->updatedAt = new \DateTimeImmutable();
    }

    public static function fromDomain(DomainUser $domainUser): self
    {
        $user = new self(
            $domainUser->getId()->getValue(),
            $domainUser->getEmail()->getValue(),
            $domainUser->getName()->getValue(),
            $domainUser->getPassword()->getValue()
        );

        $user->isActive = $domainUser->isActive();
        $user->createdAt = $domainUser->getCreatedAt();
        $user->updatedAt = $domainUser->getUpdatedAt();

        return $user;
    }

    public function toDomain(): DomainUser
    {
        return new DomainUser(
            new UserId($this->id),
            new Email($this->email),
            new UserName($this->name),
            new HashedPassword($this->password),
            $this->isActive,
            $this->createdAt,
            $this->updatedAt
        );
    }

    public function getId(): string
    {
        return $this->id;
    }

    public function getEmail(): string
    {
        return $this->email;
    }

    public function getName(): string
    {
        return $this->name;
    }

    public function getUserIdentifier(): string
    {
        return $this->email;
    }

    public function getRoles(): array
    {
        $roles = $this->roles;
        $roles[] = 'ROLE_USER';

        return array_unique($roles);
    }

    public function setRoles(array $roles): void
    {
        $this->roles = $roles;
    }

    public function getPassword(): string
    {
        return $this->password;
    }

    public function eraseCredentials(): void
    {
        // Implement if needed
    }

    public function isActive(): bool
    {
        return $this->isActive;
    }

    public function setIsActive(bool $isActive): void
    {
        $this->isActive = $isActive;
        $this->updatedAt = new \DateTimeImmutable();
    }

    public function getCreatedAt(): \DateTimeImmutable
    {
        return $this->createdAt;
    }

    public function getUpdatedAt(): \DateTimeImmutable
    {
        return $this->updatedAt;
    }
}
"#;

    pub const SYMFONY_USER_CONTROLLER: &str = r#"<?php

declare(strict_types=1);

namespace App\Infrastructure\Http\Controllers;

use App\Application\User\Commands\CreateUserCommand;
use App\Application\User\Commands\LoginCommand;
use App\Application\User\Handlers\AuthHandler;
use App\Application\User\Handlers\UserHandler;
use App\Application\User\Queries\GetUserQuery;
use Symfony\Bundle\FrameworkBundle\Controller\AbstractController;
use Symfony\Component\HttpFoundation\JsonResponse;
use Symfony\Component\HttpFoundation\Request;
use Symfony\Component\HttpFoundation\Response;
use Symfony\Component\Routing\Annotation\Route;
use Symfony\Component\Validator\Constraints as Assert;
use Symfony\Component\Validator\Validator\ValidatorInterface;

#[Route('/api')]
final class UserController extends AbstractController
{
    public function __construct(
        private readonly UserHandler $userHandler,
        private readonly AuthHandler $authHandler,
        private readonly ValidatorInterface $validator
    ) {
    }

    #[Route('/auth/register', name: 'auth_register', methods: ['POST'])]
    public function register(Request $request): JsonResponse
    {
        $data = json_decode($request->getContent(), true);

        $constraints = new Assert\Collection([
            'email' => [new Assert\NotBlank(), new Assert\Email()],
            'name' => [new Assert\NotBlank(), new Assert\Length(min: 2, max: 100)],
            'password' => [new Assert\NotBlank(), new Assert\Length(min: 8)]
        ]);

        $violations = $this->validator->validate($data, $constraints);
        if (count($violations) > 0) {
            return $this->json(['errors' => (string) $violations], Response::HTTP_BAD_REQUEST);
        }

        try {
            $command = new CreateUserCommand($data['email'], $data['name'], $data['password']);
            $result = $this->authHandler->handleRegister($command);

            return $this->json([
                'user' => [
                    'id' => $result['user']->getId()->getValue(),
                    'email' => $result['user']->getEmail()->getValue(),
                    'name' => $result['user']->getName()->getValue(),
                ],
                'token' => $result['token']
            ], Response::HTTP_CREATED);
        } catch (\DomainException $e) {
            return $this->json(['error' => $e->getMessage()], Response::HTTP_CONFLICT);
        }
    }

    #[Route('/auth/login', name: 'auth_login', methods: ['POST'])]
    public function login(Request $request): JsonResponse
    {
        $data = json_decode($request->getContent(), true);

        $constraints = new Assert\Collection([
            'email' => [new Assert\NotBlank(), new Assert\Email()],
            'password' => [new Assert\NotBlank()]
        ]);

        $violations = $this->validator->validate($data, $constraints);
        if (count($violations) > 0) {
            return $this->json(['errors' => (string) $violations], Response::HTTP_BAD_REQUEST);
        }

        try {
            $command = new LoginCommand($data['email'], $data['password']);
            $result = $this->authHandler->handleLogin($command);

            return $this->json([
                'user' => [
                    'id' => $result['user']->getId()->getValue(),
                    'email' => $result['user']->getEmail()->getValue(),
                    'name' => $result['user']->getName()->getValue(),
                ],
                'token' => $result['token']
            ]);
        } catch (\InvalidArgumentException $e) {
            return $this->json(['error' => 'Invalid credentials'], Response::HTTP_UNAUTHORIZED);
        }
    }

    #[Route('/users/{id}', name: 'get_user', methods: ['GET'])]
    public function getUser(string $id): JsonResponse
    {
        $query = new GetUserQuery($id);
        $user = $this->userHandler->handleGetUser($query);

        if (!$user) {
            return $this->json(['error' => 'User not found'], Response::HTTP_NOT_FOUND);
        }

        return $this->json([
            'id' => $user->getId()->getValue(),
            'email' => $user->getEmail()->getValue(),
            'name' => $user->getName()->getValue(),
            'isActive' => $user->isActive(),
            'createdAt' => $user->getCreatedAt()->format('c'),
            'updatedAt' => $user->getUpdatedAt()->format('c'),
        ]);
    }

    #[Route('/health', name: 'health_check', methods: ['GET'])]
    public function healthCheck(): JsonResponse
    {
        return $this->json([
            'status' => 'healthy',
            'service' => '{{project_name}} API',
            'timestamp' => (new \DateTimeImmutable())->format('c')
        ]);
    }
}
"#;

    // Symfony Value Objects
    #[allow(dead_code)]
    pub const SYMFONY_USER_ID_VALUE_OBJECT: &str = r#"<?php

declare(strict_types=1);

namespace App\Domain\User\ValueObjects;

use Symfony\Component\Uid\Uuid;

final readonly class UserId
{
    public function __construct(
        private string $value
    ) {
        if (!Uuid::isValid($this->value)) {
            throw new \InvalidArgumentException('Invalid user ID format');
        }
    }

    public static function generate(): self
    {
        return new self(Uuid::v4()->toRfc4122());
    }

    public function getValue(): string
    {
        return $this->value;
    }

    public function equals(UserId $other): bool
    {
        return $this->value === $other->value;
    }

    public function __toString(): string
    {
        return $this->value;
    }
}
"#;

    #[allow(dead_code)]
    pub const SYMFONY_EMAIL_VALUE_OBJECT: &str = r#"<?php

declare(strict_types=1);

namespace App\Domain\User\ValueObjects;

final readonly class Email
{
    public function __construct(
        private string $value
    ) {
        if (!filter_var($this->value, FILTER_VALIDATE_EMAIL)) {
            throw new \InvalidArgumentException('Invalid email format');
        }
    }

    public function getValue(): string
    {
        return $this->value;
    }

    public function equals(Email $other): bool
    {
        return $this->value === $other->value;
    }

    public function __toString(): string
    {
        return $this->value;
    }
}
"#;

    #[allow(dead_code)]
    pub const SYMFONY_USER_NAME_VALUE_OBJECT: &str = r#"<?php

declare(strict_types=1);

namespace App\Domain\User\ValueObjects;

final readonly class UserName
{
    public function __construct(
        private string $value
    ) {
        if (empty(trim($this->value))) {
            throw new \InvalidArgumentException('User name cannot be empty');
        }

        if (strlen($this->value) < 2 || strlen($this->value) > 100) {
            throw new \InvalidArgumentException('User name must be between 2 and 100 characters');
        }
    }

    public function getValue(): string
    {
        return $this->value;
    }

    public function equals(UserName $other): bool
    {
        return $this->value === $other->value;
    }

    public function __toString(): string
    {
        return $this->value;
    }
}
"#;

    #[allow(dead_code)]
    pub const SYMFONY_HASHED_PASSWORD_VALUE_OBJECT: &str = r#"<?php

declare(strict_types=1);

namespace App\Domain\User\ValueObjects;

final readonly class HashedPassword
{
    public function __construct(
        private string $value
    ) {
        if (empty($this->value)) {
            throw new \InvalidArgumentException('Hashed password cannot be empty');
        }
    }

    public function getValue(): string
    {
        return $this->value;
    }

    public function equals(HashedPassword $other): bool
    {
        return $this->value === $other->value;
    }
}
"#;
    // .env.docker for secure environment variables
    pub const PHP_ENV_DOCKER: &str = r#"# Docker Environment Variables
# These should be kept secure and not committed to version control

# Application secrets
APP_KEY=base64:$(openssl rand -base64 32)
APP_SECRET=$(openssl rand -hex 32)
JWT_SECRET=$(openssl rand -hex 32)
JWT_PASSPHRASE=$(openssl rand -hex 16)

# Database credentials
DB_PASSWORD=$(openssl rand -hex 16)
POSTGRES_PASSWORD=${DB_PASSWORD}

# Other secrets
REDIS_PASSWORD=
MAIL_PASSWORD=
"#;

    // Docker Compose Development Override
    pub const PHP_DOCKER_COMPOSE_DEV: &str = r#"# Development overrides for docker-compose.yml
# Usage: docker-compose -f docker-compose.yml -f docker-compose.dev.yml up

services:
  app:
    build:
      target: development
    environment:
      - APP_ENV=local
      - APP_DEBUG=true
    volumes:
      - .:/var/www/html
      - ./docker/php/local.ini:/usr/local/etc/php/conf.d/local.ini

  adminer:
    image: adminer:latest
    container_name: {{kebab_case}}-adminer
    ports:
      - "8080:8080"
    environment:
      - ADMINER_DEFAULT_SERVER=postgres
    depends_on:
      postgres:
        condition: service_healthy
    networks:
      - {{kebab_case}}-network
    restart: unless-stopped

  postgres:
    ports:
      - "5432:5432"

  redis:
    ports:
      - "6379:6379"

  mailhog:
    ports:
      - "1025:1025"
      - "8025:8025"
"#;

    // Optimized Dockerfile for PHP
    pub const PHP_OPTIMIZED_DOCKERFILE: &str = r#"# Multi-stage Dockerfile for PHP applications
FROM php:8.2-fpm-alpine AS base

# Install system dependencies
RUN apk add --no-cache \
    nginx \
    supervisor \
    curl \
    postgresql-dev \
    libpng-dev \
    libjpeg-turbo-dev \
    freetype-dev \
    libzip-dev \
    oniguruma-dev \
    && docker-php-ext-configure gd --with-freetype --with-jpeg \
    && docker-php-ext-install -j$(nproc) gd \
    && docker-php-ext-install pdo pdo_pgsql zip bcmath opcache

# Install Composer
COPY --from=composer:2 /usr/bin/composer /usr/bin/composer

# Set working directory
WORKDIR /var/www/html

# Copy composer files
COPY composer.json composer.lock ./

# Development stage
FROM base AS development
RUN composer install --prefer-dist --no-scripts --no-autoloader
COPY . .
RUN composer dump-autoload --optimize

# Production stage
FROM base AS production
RUN composer install --no-dev --optimize-autoloader --no-scripts
COPY . .
RUN composer dump-autoload --optimize --classmap-authoritative \
    && php artisan config:cache \
    && php artisan route:cache \
    && php artisan view:cache

# Set permissions
RUN chown -R www-data:www-data /var/www/html \
    && chmod -R 755 /var/www/html/storage

# Copy supervisor config
COPY docker/php/supervisord.conf /etc/supervisor/conf.d/supervisord.conf

EXPOSE 9000

CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]
"#;

    // Nginx Dockerfile
    pub const PHP_NGINX_DOCKERFILE: &str = r#"FROM nginx:alpine

# Copy nginx configuration
COPY docker/nginx/nginx.conf /etc/nginx/nginx.conf
COPY docker/nginx/default.conf /etc/nginx/conf.d/default.conf

# Create log directory
RUN mkdir -p /var/log/nginx

EXPOSE 80 443

CMD ["nginx", "-g", "daemon off;"]
"#;

    pub const SYMFONY_AUTH_CONTROLLER: &str = r#"<?php // Symfony Auth Controller"#;
    pub const SYMFONY_API_ROUTES: &str = r#"# Symfony API Routes"#;
    pub const SYMFONY_DOCKER_COMPOSE: &str = r#"services:
  app:
    build:
      context: .
      dockerfile: docker/php/Dockerfile
      target: production
    container_name: {{kebab_case}}-app
    env_file:
      - .env.docker
    environment:
      - APP_ENV=prod
      - DATABASE_URL=postgresql://postgres:${DB_PASSWORD}@postgres:5432/{{snake_case}}_db?serverVersion=16&charset=utf8
      - REDIS_URL=redis://redis:6379
      - MAILER_DSN=smtp://mailhog:1025
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_started
    volumes:
      - app_data:/var/www/html
      - app_logs:/var/www/html/var/log
    networks:
      - {{kebab_case}}-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "php", "bin/console", "app:health-check"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  nginx:
    build:
      context: .
      dockerfile: docker/nginx/Dockerfile
    container_name: {{kebab_case}}-nginx
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - app_data:/var/www/html:ro
      - nginx_logs:/var/log/nginx
    depends_on:
      app:
        condition: service_healthy
    networks:
      - {{kebab_case}}-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  postgres:
    image: postgres:16-alpine
    container_name: {{kebab_case}}-postgres
    env_file:
      - .env.docker
    environment:
      - POSTGRES_DB={{snake_case}}_db
      - POSTGRES_USER=postgres
    volumes:
      - postgres_data:/var/lib/postgresql/data
    expose:
      - "5432"
    networks:
      - {{kebab_case}}-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "pg_isready", "-U", "postgres", "-d", "{{snake_case}}_db"]
      interval: 30s
      timeout: 10s
      retries: 5
      start_period: 30s

  redis:
    image: redis:7-alpine
    container_name: {{kebab_case}}-redis
    volumes:
      - redis_data:/data
      - ./docker/redis/redis.conf:/usr/local/etc/redis/redis.conf:ro
    expose:
      - "6379"
    networks:
      - {{kebab_case}}-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 30s
      timeout: 10s
      retries: 3
    command: redis-server /usr/local/etc/redis/redis.conf

  mailhog:
    image: mailhog/mailhog:latest
    container_name: {{kebab_case}}-mailhog
    expose:
      - "1025"
      - "8025"
    networks:
      - {{kebab_case}}-network
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
  app_data:
  app_logs:
  nginx_logs:

networks:
  {{kebab_case}}-network:
    driver: bridge
"#;
    pub const SYMFONY_NGINX_DEFAULT_CONF: &str = r#"# Symfony Nginx Config"#;
    pub const SYMFONY_ENV_EXAMPLE: &str = r#"# Symfony Environment"#;
    pub const SYMFONY_PHPUNIT_XML: &str = r#"<!-- Symfony PHPUnit -->"#;
    pub const SYMFONY_AUTH_FUNCTIONAL_TEST: &str = r#"<?php // Symfony Auth Test"#;
    pub const SYMFONY_USER_UNIT_TEST: &str = r#"<?php // Symfony User Test"#;
    pub const SYMFONY_README: &str = r#"# Symfony Project"#;
}
