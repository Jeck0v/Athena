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