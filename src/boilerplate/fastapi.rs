//! FastAPI boilerplate generator with production-ready features

use crate::boilerplate::{BoilerplateGenerator, BoilerplateResult, ProjectConfig, DatabaseType};
use crate::boilerplate::utils::{create_directory_structure, write_file, replace_template_vars_string, generate_secret_key, ProjectNames};
use crate::boilerplate::templates::fastapi::*;
use std::path::Path;

pub struct FastAPIGenerator;

impl Default for FastAPIGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl FastAPIGenerator {
    pub fn new() -> Self {
        Self
    }

    fn get_template_vars(&self, config: &ProjectConfig) -> Vec<(&str, String)> {
        let names = ProjectNames::new(&config.name);
        let secret_key = generate_secret_key();
        
        vec![
            ("project_name", config.name.clone()),
            ("snake_case", names.snake_case.clone()),
            ("kebab_case", names.kebab_case.clone()),
            ("pascal_case", names.pascal_case),
            ("upper_case", names.upper_case),
            ("secret_key", secret_key),
            ("module_name", names.kebab_case),
        ]
    }

    fn create_fastapi_structure(&self, base_path: &Path) -> BoilerplateResult<()> {
        let directories = vec![
            "app",
            "app/api",
            "app/api/v1",
            "app/core",
            "app/database",
            "app/models",
            "app/schemas",
            "app/services",
            "tests",
            "tests/api",
            "logs",
            "nginx",
            "nginx/conf.d",
            "scripts",
        ];

        create_directory_structure(base_path, &directories)
    }

    fn generate_core_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);
        
        // Main application file
        let main_content = replace_template_vars_string(MAIN_PY, &vars);
        write_file(base_path.join("app/main.py"), &main_content)?;

        // Core configuration
        let mut config_content = CONFIG_PY.to_string();
        match config.database {
            DatabaseType::MongoDB => {
                config_content = config_content.replace("{{#if mongodb}}", "");
                config_content = config_content.replace("{{/if}}", "");
                config_content = config_content.replace("{{#if postgresql}}", "");
                config_content = config_content.replace("{{/if}}", "");
            }
            DatabaseType::PostgreSQL => {
                config_content = config_content.replace("{{#if postgresql}}", "");
                config_content = config_content.replace("{{/if}}", "");
                config_content = config_content.replace("{{#if mongodb}}", "");
                config_content = config_content.replace("{{/if}}", "");
            }
            DatabaseType::MySQL => {
                return Err(crate::athena::AthenaError::ValidationError(
                    "MySQL is not supported for FastAPI projects. Use Flask for MySQL support.".to_string()
                ));
            }
        }
        config_content = replace_template_vars_string(&config_content, &vars);
        write_file(base_path.join("app/core/config.py"), &config_content)?;
        write_file(base_path.join("app/core/__init__.py"), "")?;

        // Security module
        let security_content = replace_template_vars_string(SECURITY_PY, &vars);
        write_file(base_path.join("app/core/security.py"), &security_content)?;

        Ok(())
    }

    fn generate_api_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        // API init files
        write_file(base_path.join("app/api/__init__.py"), "")?;
        write_file(base_path.join("app/api/v1/__init__.py"), "")?;

        // Health endpoint (at root level)
        let health_py = r#"from fastapi import APIRouter

router = APIRouter()

@router.get("/health")
async def health_check():
    return {"status": "healthy", "service": "{{project_name}} API"}
"#;
        let health_content = replace_template_vars_string(health_py, &vars);
        write_file(base_path.join("app/api/health.py"), &health_content)?;

        // Auth endpoints
        let auth_py = r#"from fastapi import APIRouter, HTTPException, Depends, status
from fastapi.security import HTTPBearer
from pydantic import BaseModel
from typing import Optional

from app.core.security import (
    verify_password, get_password_hash, create_access_token, 
    create_refresh_token, verify_token
)
from app.services.user_service import UserService

router = APIRouter()
security = HTTPBearer()

class LoginRequest(BaseModel):
    email: str
    password: str

class RegisterRequest(BaseModel):
    email: str
    password: str
    full_name: str

class TokenResponse(BaseModel):
    access_token: str
    refresh_token: str
    token_type: str = "bearer"

class RefreshRequest(BaseModel):
    refresh_token: str

@router.post("/login", response_model=TokenResponse)
async def login(request: LoginRequest):
    user_service = UserService()
    
    # Find user by email
    user = await user_service.get_by_email(request.email)
    if not user or not verify_password(request.password, user.hashed_password):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid credentials"
        )
    
    # Create tokens
    access_token = create_access_token({"sub": str(user.id), "email": user.email})
    refresh_token = create_refresh_token({"sub": str(user.id)})
    
    return TokenResponse(
        access_token=access_token,
        refresh_token=refresh_token
    )

@router.post("/register", response_model=TokenResponse)
async def register(request: RegisterRequest):
    user_service = UserService()
    
    # Check if user already exists
    existing_user = await user_service.get_by_email(request.email)
    if existing_user:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="User already exists"
        )
    
    # Create user
    hashed_password = get_password_hash(request.password)
    user = await user_service.create({
        "email": request.email,
        "hashed_password": hashed_password,
        "full_name": request.full_name,
        "is_active": True
    })
    
    # Create tokens
    access_token = create_access_token({"sub": str(user.id), "email": user.email})
    refresh_token = create_refresh_token({"sub": str(user.id)})
    
    return TokenResponse(
        access_token=access_token,
        refresh_token=refresh_token
    )

@router.post("/refresh", response_model=TokenResponse)
async def refresh_token(request: RefreshRequest):
    # Verify refresh token
    payload = verify_token(request.refresh_token, "refresh")
    user_id = payload.get("sub")
    
    if not user_id:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid refresh token"
        )
    
    user_service = UserService()
    user = await user_service.get_by_id(user_id)
    
    if not user:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="User not found"
        )
    
    # Create new tokens
    access_token = create_access_token({"sub": str(user.id), "email": user.email})
    new_refresh_token = create_refresh_token({"sub": str(user.id)})
    
    return TokenResponse(
        access_token=access_token,
        refresh_token=new_refresh_token
    )
"#;
        let auth_content = replace_template_vars_string(auth_py, &vars);
        write_file(base_path.join("app/api/v1/auth.py"), &auth_content)?;

        // Users endpoints  
        let users_py = r#"from fastapi import APIRouter, HTTPException, Depends, status
from fastapi.security import HTTPAuthorizationCredentials
from pydantic import BaseModel
from typing import List, Optional

from app.core.security import verify_token
from fastapi.security import HTTPBearer

security = HTTPBearer()
from app.services.user_service import UserService

router = APIRouter()

class UserResponse(BaseModel):
    id: str
    email: str
    full_name: str
    is_active: bool

async def get_current_user(credentials: HTTPAuthorizationCredentials = Depends(security)):
    payload = verify_token(credentials.credentials)
    user_id = payload.get("sub")
    
    if not user_id:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid token"
        )
    
    user_service = UserService()
    user = await user_service.get_by_id(user_id)
    
    if not user:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="User not found"
        )
    
    return user

@router.get("/me", response_model=UserResponse)
async def get_current_user_info(current_user = Depends(get_current_user)):
    return UserResponse(
        id=str(current_user.id),
        email=current_user.email,
        full_name=current_user.full_name,
        is_active=current_user.is_active
    )
"#;
        let users_content = replace_template_vars_string(users_py, &vars);
        write_file(base_path.join("app/api/v1/users.py"), &users_content)?;

        Ok(())
    }

    fn generate_database_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let _vars = self.get_template_vars(config);

        write_file(base_path.join("app/database/__init__.py"), "")?;

        match config.database {
            DatabaseType::MongoDB => {
                let mongodb_connection = r#"from motor.motor_asyncio import AsyncIOMotorClient
from app.core.config import settings
import logging

logger = logging.getLogger(__name__)

class Database:
    client: AsyncIOMotorClient = None
    database = None

db = Database()

async def init_database():
    """Initialize database connection"""
    try:
        db.client = AsyncIOMotorClient(settings.MONGODB_URL)
        db.database = db.client[settings.DATABASE_NAME]
        # Test connection
        await db.client.admin.command('ping')
        logger.info("Successfully connected to MongoDB")
    except Exception as e:
        logger.error(f"Error connecting to MongoDB: {e}")
        raise

async def close_database_connection():
    """Close database connection"""
    if db.client:
        db.client.close()
        logger.info("Disconnected from MongoDB")

def get_database():
    """Get database instance"""
    return db.database
"#;
                write_file(base_path.join("app/database/connection.py"), mongodb_connection)?;
            }
            DatabaseType::PostgreSQL => {
                let postgres_connection = r#"from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession, async_sessionmaker
from sqlalchemy.orm import DeclarativeBase
from app.core.config import settings
import logging

logger = logging.getLogger(__name__)

# Create async engine
engine = None
async_session_maker = None

# Base class for models
class Base(DeclarativeBase):
    pass

async def init_database():
    """Initialize database connection"""
    global engine, async_session_maker
    
    try:
        # Create async engine
        engine = create_async_engine(
            settings.DATABASE_URL.replace("postgresql://", "postgresql+asyncpg://"),
            echo=settings.ENVIRONMENT == "development",
            future=True
        )
        
        # Create session maker
        async_session_maker = async_sessionmaker(
            engine, 
            class_=AsyncSession, 
            expire_on_commit=False
        )
        
        # Test connection
        async with engine.begin() as conn:
            await conn.run_sync(Base.metadata.create_all)
            
        logger.info("Successfully connected to PostgreSQL")
    except Exception as e:
        logger.error(f"Error connecting to PostgreSQL: {e}")
        raise

async def close_database_connection():
    """Close database connection"""
    global engine
    
    if engine:
        await engine.dispose()
        logger.info("Disconnected from PostgreSQL")

async def get_async_session() -> AsyncSession:
    """Get async database session"""
    if not async_session_maker:
        raise RuntimeError("Database not initialized")
    
    async with async_session_maker() as session:
        try:
            yield session
        except Exception:
            await session.rollback()
            raise
        finally:
            await session.close()

def get_engine():
    """Get database engine"""
    return engine
"#;
                write_file(base_path.join("app/database/connection.py"), postgres_connection)?;
            }
            DatabaseType::MySQL => {
                return Err(crate::athena::AthenaError::ValidationError(
                    "MySQL is not supported for FastAPI projects. Use Flask for MySQL support.".to_string()
                ));
            }
        }

        Ok(())
    }

    fn generate_models_and_services(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let _vars = self.get_template_vars(config);

        // Models
        write_file(base_path.join("app/models/__init__.py"), "")?;
        
        let user_model = match config.database {
            DatabaseType::MySQL => {
                return Err(crate::athena::AthenaError::ValidationError(
                    "MySQL is not supported for FastAPI projects. Use Flask for MySQL support.".to_string()
                ));
            }
            DatabaseType::MongoDB => r#"from pydantic import BaseModel, Field, ConfigDict
from typing import Optional
from bson import ObjectId

class PyObjectId(ObjectId):
    @classmethod
    def __get_validators__(cls):
        yield cls.validate

    @classmethod
    def validate(cls, v):
        if not ObjectId.is_valid(v):
            raise ValueError("Invalid ObjectId")
        return ObjectId(v)

    @classmethod
    def __get_pydantic_json_schema__(cls, field_schema, handler):
        json_schema = handler(str)
        json_schema.update(type="string")
        return json_schema

class User(BaseModel):
    id: PyObjectId = Field(default_factory=PyObjectId, alias="_id")
    email: str
    hashed_password: str
    full_name: str
    is_active: bool = True

    model_config = ConfigDict(
        populate_by_name=True,
        arbitrary_types_allowed=True,
        json_encoders={ObjectId: str}
    )
"#.to_string(),
            DatabaseType::PostgreSQL => r#"from sqlalchemy import Column, String, Boolean, DateTime
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.sql import func
from pydantic import BaseModel, ConfigDict
from typing import Optional
import uuid
from datetime import datetime

from app.database.connection import Base

# SQLAlchemy model
class UserTable(Base):
    __tablename__ = "users"
    
    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    email = Column(String, unique=True, nullable=False, index=True)
    hashed_password = Column(String, nullable=False)
    full_name = Column(String, nullable=False)
    is_active = Column(Boolean, default=True)
    created_at = Column(DateTime(timezone=True), server_default=func.now())
    updated_at = Column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())

# Pydantic models
class User(BaseModel):
    id: Optional[uuid.UUID] = None
    email: str
    hashed_password: str
    full_name: str
    is_active: bool = True
    created_at: Optional[datetime] = None
    updated_at: Optional[datetime] = None

    model_config = ConfigDict(from_attributes=True)

class UserCreate(BaseModel):
    email: str
    password: str
    full_name: str

class UserResponse(BaseModel):
    id: uuid.UUID
    email: str
    full_name: str
    is_active: bool
    created_at: datetime
    
    model_config = ConfigDict(from_attributes=True)
"#.to_string()
        };

        write_file(base_path.join("app/models/user.py"), &user_model)?;

        // Services
        write_file(base_path.join("app/services/__init__.py"), "")?;
        
        let user_service = match config.database {
            DatabaseType::MySQL => {
                return Err(crate::athena::AthenaError::ValidationError(
                    "MySQL is not supported for FastAPI projects. Use Flask for MySQL support.".to_string()
                ));
            }
            DatabaseType::MongoDB => r#"from typing import Optional, Dict, Any
from bson import ObjectId
from app.models.user import User
from app.database.connection import get_database

class UserService:
    def __init__(self):
        self.db = get_database()
        self.collection = self.db.users

    async def create(self, user_data: Dict[str, Any]) -> User:
        """Create a new user"""
        result = await self.collection.insert_one(user_data)
        user_data["_id"] = result.inserted_id
        return User(**user_data)

    async def get_by_id(self, user_id: str) -> Optional[User]:
        """Get user by ID"""
        user_doc = await self.collection.find_one({"_id": ObjectId(user_id)})
        return User(**user_doc) if user_doc else None

    async def get_by_email(self, email: str) -> Optional[User]:
        """Get user by email"""
        user_doc = await self.collection.find_one({"email": email})
        return User(**user_doc) if user_doc else None

    async def update(self, user_id: str, update_data: Dict[str, Any]) -> Optional[User]:
        """Update user"""
        await self.collection.update_one(
            {"_id": ObjectId(user_id)},
            {"$set": update_data}
        )
        return await self.get_by_id(user_id)

    async def delete(self, user_id: str) -> bool:
        """Delete user"""
        result = await self.collection.delete_one({"_id": ObjectId(user_id)})
        return result.deleted_count > 0
"#.to_string(),
            DatabaseType::PostgreSQL => r#"from typing import Optional, Dict, Any
from sqlalchemy import select, insert, update, delete
from sqlalchemy.ext.asyncio import AsyncSession
import uuid

from app.models.user import User, UserTable
from app.database.connection import get_async_session

class UserService:
    async def create(self, user_data: Dict[str, Any]) -> User:
        """Create a new user"""
        async for session in get_async_session():
            # Create new user
            new_user = UserTable(
                email=user_data["email"],
                hashed_password=user_data["hashed_password"],
                full_name=user_data["full_name"],
                is_active=user_data.get("is_active", True)
            )
            
            session.add(new_user)
            await session.commit()
            await session.refresh(new_user)
            
            return User.from_orm(new_user)

    async def get_by_id(self, user_id: str) -> Optional[User]:
        """Get user by ID"""
        async for session in get_async_session():
            stmt = select(UserTable).where(UserTable.id == uuid.UUID(user_id))
            result = await session.execute(stmt)
            user_row = result.scalar_one_or_none()
            
            return User.from_orm(user_row) if user_row else None

    async def get_by_email(self, email: str) -> Optional[User]:
        """Get user by email"""
        async for session in get_async_session():
            stmt = select(UserTable).where(UserTable.email == email)
            result = await session.execute(stmt)
            user_row = result.scalar_one_or_none()
            
            return User.from_orm(user_row) if user_row else None

    async def update(self, user_id: str, update_data: Dict[str, Any]) -> Optional[User]:
        """Update user"""
        async for session in get_async_session():
            stmt = (
                update(UserTable)
                .where(UserTable.id == uuid.UUID(user_id))
                .values(**update_data)
                .returning(UserTable)
            )
            result = await session.execute(stmt)
            await session.commit()
            user_row = result.scalar_one_or_none()
            
            return User.from_orm(user_row) if user_row else None

    async def delete(self, user_id: str) -> bool:
        """Delete user"""
        async for session in get_async_session():
            stmt = delete(UserTable).where(UserTable.id == uuid.UUID(user_id))
            result = await session.execute(stmt)
            await session.commit()
            
            return result.rowcount > 0
"#.to_string()
        };

        write_file(base_path.join("app/services/user_service.py"), &user_service)?;

        Ok(())
    }

    fn generate_docker_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        if !config.include_docker {
            return Ok(());
        }

        let vars = self.get_template_vars(config);

        // Requirements
        let mut requirements = REQUIREMENTS_TXT.to_string();
        match config.database {
            DatabaseType::MongoDB => {
                requirements = requirements.replace("{{#if mongodb}}", "");
                requirements = requirements.replace("{{/if}}", "");
                requirements = requirements.replace("{{#if postgresql}}", "");
                requirements = requirements.replace("{{/if}}", "");
            }
            DatabaseType::PostgreSQL => {
                requirements = requirements.replace("{{#if postgresql}}", "");
                requirements = requirements.replace("{{/if}}", "");
                requirements = requirements.replace("{{#if mongodb}}", "");
                requirements = requirements.replace("{{/if}}", "");
            }
            DatabaseType::MySQL => {
                return Err(crate::athena::AthenaError::ValidationError(
                    "MySQL is not supported for FastAPI projects. Use Flask for MySQL support.".to_string()
                ));
            }
        }
        write_file(base_path.join("requirements.txt"), &requirements)?;

        // Dockerfile
        write_file(base_path.join("Dockerfile"), DOCKERFILE)?;

        // Docker Compose
        let mut compose_content = DOCKER_COMPOSE_YML.to_string();
        match config.database {
            DatabaseType::MongoDB => {
                compose_content = compose_content.replace("{{#if mongodb}}", "");
                compose_content = compose_content.replace("{{/if}}", "");
                compose_content = compose_content.replace("{{#if postgresql}}", "");
                compose_content = compose_content.replace("{{/if}}", "");
            }
            DatabaseType::PostgreSQL => {
                compose_content = compose_content.replace("{{#if postgresql}}", "");
                compose_content = compose_content.replace("{{/if}}", "");
                compose_content = compose_content.replace("{{#if mongodb}}", "");
                compose_content = compose_content.replace("{{/if}}", "");
            }
            DatabaseType::MySQL => {
                return Err(crate::athena::AthenaError::ValidationError(
                    "MySQL is not supported for FastAPI projects. Use Flask for MySQL support.".to_string()
                ));
            }
        }
        compose_content = replace_template_vars_string(&compose_content, &vars);
        write_file(base_path.join("docker-compose.yml"), &compose_content)?;

        // Nginx configurations
        let nginx_content = replace_template_vars_string(NGINX_CONF, &vars);
        write_file(base_path.join("nginx/nginx.conf"), &nginx_content)?;
        
        let nginx_default_content = replace_template_vars_string(crate::boilerplate::templates::fastapi::NGINX_DEFAULT_CONF, &vars);
        write_file(base_path.join("nginx/conf.d/default.conf"), &nginx_default_content)?;

        // .env template
        let env_template = format!(r#"# Environment Configuration
ENVIRONMENT=development

# Security
SECRET_KEY=your-secret-key-here-change-in-production

# Database{}{}

# Redis
REDIS_URL=redis://localhost:6379

# CORS
ALLOWED_HOSTS=["http://localhost:3000","http://127.0.0.1:3000"]
"#,
            if matches!(config.database, DatabaseType::MongoDB) {
                "\nMONGODB_URL=mongodb://localhost:27017\nDATABASE_NAME=".to_string() + &ProjectNames::new(&config.name).snake_case + "_db"
            } else { "".to_string() },
            if matches!(config.database, DatabaseType::PostgreSQL) {
                "\nDATABASE_URL=postgresql://user:password@localhost/".to_string() + &ProjectNames::new(&config.name).snake_case + "_db\nPOSTGRES_PASSWORD=your-postgres-password"
            } else { "".to_string() }
        );
        write_file(base_path.join(".env.example"), &env_template)?;

        Ok(())
    }

    fn generate_test_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        write_file(base_path.join("tests/__init__.py"), "")?;

        let test_main = r#"import pytest
from fastapi.testclient import TestClient
from app.main import app

client = TestClient(app)

def test_root():
    response = client.get("/")
    assert response.status_code == 200
    assert response.json()["message"] == "{{project_name}} API is running"

def test_health():
    response = client.get("/health")
    assert response.status_code == 200
    assert response.json()["status"] == "healthy"
"#;
        let test_content = replace_template_vars_string(test_main, &vars);
        write_file(base_path.join("tests/test_main.py"), &test_content)?;

        let test_auth = r#"import pytest
from fastapi.testclient import TestClient
from app.main import app

client = TestClient(app)

def test_register_user():
    response = client.post(
        "/api/v1/auth/register",
        json={
            "email": "test@example.com",
            "password": "testpassword123",
            "full_name": "Test User"
        }
    )
    assert response.status_code == 200
    data = response.json()
    assert "access_token" in data
    assert "refresh_token" in data
    assert data["token_type"] == "bearer"

def test_login_user():
    # First register a user
    client.post(
        "/api/v1/auth/register",
        json={
            "email": "login@example.com",
            "password": "testpassword123",
            "full_name": "Login User"
        }
    )
    
    # Then login
    response = client.post(
        "/api/v1/auth/login",
        json={
            "email": "login@example.com",
            "password": "testpassword123"
        }
    )
    assert response.status_code == 200
    data = response.json()
    assert "access_token" in data
    assert "refresh_token" in data

def test_invalid_login():
    response = client.post(
        "/api/v1/auth/login",
        json={
            "email": "nonexistent@example.com",
            "password": "wrongpassword"
        }
    )
    assert response.status_code == 401
"#;
        write_file(base_path.join("tests/test_auth.py"), test_auth)?;

        Ok(())
    }

    fn generate_documentation(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let _vars = self.get_template_vars(config);
        let names = ProjectNames::new(&config.name);

        let readme = format!(r#"# {project_name}

Production-ready FastAPI application with authentication and security features.

## Features

- **FastAPI** - Modern, fast web framework
- **JWT Authentication** - Access & refresh tokens
- **Password Security** - bcrypt hashing
- **{database}** - Database integration
- **Docker** - Containerized deployment
- **Nginx** - Reverse proxy with security headers
- **Tests** - Comprehensive test suite
- **Security** - CORS, rate limiting, input validation

## Quick Start

### Development

```bash
# Install dependencies
pip install -r requirements.txt

# Set up environment
cp .env.example .env
# Edit .env with your configuration

# Run the application
uvicorn app.main:app --host 0.0.0.0 --port 8000 --reload
```

### With Docker

```bash
# Build and run with Docker Compose
docker-compose up --build

# The API will be available at http://localhost
```

## API Documentation

Once running, visit:
- **Swagger UI**: http://localhost:8000/docs
- **ReDoc**: http://localhost:8000/redoc

## API Endpoints

### Authentication
- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/login` - User login
- `POST /api/v1/auth/refresh` - Refresh access token

### Users
- `GET /api/v1/users/me` - Get current user info

### System
- `GET /health` - Health check
- `GET /` - API info

## Testing

```bash
# Run tests
pytest

# Run tests with coverage
pytest --cov=app

# Run specific test
pytest tests/test_auth.py::test_register_user
```

## Project Structure

```
{snake_case}/
├── app/
│   ├── api/                 # API routes
│   ├── core/               # Core functionality
│   ├── database/           # Database connection
│   ├── models/             # Data models
│   ├── schemas/            # Pydantic schemas
│   ├── services/           # Business logic
│   └── main.py            # FastAPI application
├── tests/                  # Test suite
├── nginx/                  # Nginx configuration
├── logs/                   # Application logs
├── requirements.txt        # Python dependencies
├── Dockerfile             # Docker configuration
└── docker-compose.yml     # Docker Compose setup
```

## Configuration

Key environment variables:

```env
ENVIRONMENT=development
SECRET_KEY=your-secret-key-here
{database_config}
REDIS_URL=redis://localhost:6379
ALLOWED_HOSTS=["http://localhost:3000"]
```

## Security Features

- JWT-based authentication with refresh tokens
- Password hashing with bcrypt
- CORS configuration
- Security headers via Nginx
- Rate limiting
- Input validation
- SQL injection prevention

## Deployment

1. Set `ENVIRONMENT=production` in your environment
2. Use strong `SECRET_KEY`
3. Configure your database connection
4. Set up SSL certificates for HTTPS
5. Configure firewall rules

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new features
4. Run the test suite
5. Submit a pull request

Generated with love by Athena CLI
"#,
            project_name = config.name,
            database = match config.database {
                DatabaseType::MySQL => "MySQL",
                DatabaseType::MongoDB => "MongoDB",
                DatabaseType::PostgreSQL => "PostgreSQL",
            },
            snake_case = names.snake_case,
            database_config = match config.database {
                DatabaseType::MySQL => format!("DATABASE_URL=mysql+pymysql://root:password@localhost/{}_db", names.snake_case),
                DatabaseType::MongoDB => format!("MONGODB_URL=mongodb://localhost:27017\nDATABASE_NAME={}_db", names.snake_case),
                DatabaseType::PostgreSQL => format!("DATABASE_URL=postgresql://user:password@localhost/{}_db", names.snake_case),
            }
        );
        
        write_file(base_path.join("README.md"), &readme)?;

        Ok(())
    }
}

impl BoilerplateGenerator for FastAPIGenerator {
    fn validate_config(&self, config: &ProjectConfig) -> BoilerplateResult<()> {
        crate::boilerplate::validate_project_name(&config.name)?;
        crate::boilerplate::check_directory_availability(Path::new(&config.directory))?;
        Ok(())
    }

    fn generate_project(&self, config: &ProjectConfig) -> BoilerplateResult<()> {
        let base_path = Path::new(&config.directory);

        println!("Generating FastAPI project: {}", config.name);
        
        // Create directory structure
        println!("  Creating directory structure...");
        self.create_fastapi_structure(base_path)?;

        // Generate core files
        println!("  Generating core application files...");
        self.generate_core_files(config, base_path)?;

        // Generate API files
        println!("  Generating API endpoints...");
        self.generate_api_files(config, base_path)?;

        // Generate database files
        println!("  💾 Setting up database integration...");
        self.generate_database_files(config, base_path)?;

        // Generate models and services
        println!("  📊 Creating models and services...");
        self.generate_models_and_services(config, base_path)?;

        // Generate Docker files
        if config.include_docker {
            println!("  🐳 Generating Docker configuration...");
            self.generate_docker_files(config, base_path)?;
        }

        // Generate test files
        println!("  🧪 Creating test suite...");
        self.generate_test_files(config, base_path)?;

        // Generate documentation
        println!("  📚 Generating documentation...");
        self.generate_documentation(config, base_path)?;

        println!("FastAPI project '{}' created successfully!", config.name);
        println!("📍 Location: {}", base_path.display());
        
        if config.include_docker {
            println!("\n🔧 Next steps:");
            println!("  cd {}", config.directory);
            println!("  cp .env.example .env  # Edit with your configuration");
            println!("  docker-compose up --build");
        } else {
            println!("\n🔧 Next steps:");
            println!("  cd {}", config.directory);
            println!("  pip install -r requirements.txt");
            println!("  cp .env.example .env  # Edit with your configuration");
            println!("  uvicorn app.main:app --reload");
        }

        Ok(())
    }
}