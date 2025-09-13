//! Flask boilerplate generator with production-ready features

use crate::boilerplate::{BoilerplateGenerator, BoilerplateResult, ProjectConfig, DatabaseType};
use crate::boilerplate::utils::{create_directory_structure, write_file, replace_template_vars_string, generate_secret_key, ProjectNames};
use crate::boilerplate::templates::flask::*;
use std::path::Path;

pub struct FlaskGenerator;

impl FlaskGenerator {
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

    fn create_flask_structure(&self, base_path: &Path) -> BoilerplateResult<()> {
        let directories = vec![
            "app",
            "app/api",
            "app/api/v1",
            "app/core",
            "app/database",
            "app/models",
            "app/schemas",
            "app/services",
            "app/utils",
            "tests",
            "tests/api",
            "migrations",
            "logs",
            "nginx",
            "nginx/conf.d",
            "scripts",
            "instance",
        ];

        create_directory_structure(base_path, &directories)
    }

    fn generate_core_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);
        
        // Main application factory
        let app_init = replace_template_vars_string(APP_INIT_PY, &vars);
        write_file(base_path.join("app/__init__.py"), &app_init)?;

        // Configuration
        let config_content = replace_template_vars_string(CONFIG_PY, &vars);
        write_file(base_path.join("app/core/config.py"), &config_content)?;
        write_file(base_path.join("app/core/__init__.py"), "")?;

        // Extensions (Flask extensions initialization)
        let extensions_content = replace_template_vars_string(EXTENSIONS_PY, &vars);
        write_file(base_path.join("app/core/extensions.py"), &extensions_content)?;

        // Security module
        let security_content = replace_template_vars_string(SECURITY_PY, &vars);
        write_file(base_path.join("app/core/security.py"), &security_content)?;

        // Main application entry point
        let main_content = replace_template_vars_string(MAIN_PY, &vars);
        write_file(base_path.join("run.py"), &main_content)?;

        Ok(())
    }

    fn generate_api_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        // API init files
        write_file(base_path.join("app/api/__init__.py"), "")?;
        write_file(base_path.join("app/api/v1/__init__.py"), "")?;

        // Health endpoint
        let health_content = replace_template_vars_string(HEALTH_BP, &vars);
        write_file(base_path.join("app/api/health.py"), &health_content)?;

        // Auth endpoints
        let auth_content = replace_template_vars_string(AUTH_BP, &vars);
        write_file(base_path.join("app/api/v1/auth.py"), &auth_content)?;

        // Users endpoints  
        let users_content = replace_template_vars_string(USERS_BP, &vars);
        write_file(base_path.join("app/api/v1/users.py"), &users_content)?;

        Ok(())
    }

    fn generate_database_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        write_file(base_path.join("app/database/__init__.py"), "")?;

        // Choose database connection based on config
        match config.database {
            DatabaseType::MySQL => {
                let mysql_connection = replace_template_vars_string(MYSQL_CONNECTION, &vars);
                write_file(base_path.join("app/database/connection.py"), &mysql_connection)?;
            }
            DatabaseType::PostgreSQL => {
                let postgres_connection = replace_template_vars_string(POSTGRES_CONNECTION, &vars);
                write_file(base_path.join("app/database/connection.py"), &postgres_connection)?;
            }
            _ => {
                return Err(crate::athena::AthenaError::ValidationError(
                    "Unsupported database type for Flask".to_string()
                ));
            }
        }

        Ok(())
    }

    fn generate_models_and_services(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        // Models
        write_file(base_path.join("app/models/__init__.py"), "")?;
        
        // User model - choose based on database type
        let user_model = match config.database {
            DatabaseType::MySQL => replace_template_vars_string(USER_MODEL_MYSQL, &vars),
            DatabaseType::PostgreSQL => replace_template_vars_string(USER_MODEL, &vars),
            _ => return Err(crate::athena::AthenaError::ValidationError(
                "Unsupported database type for Flask".to_string()
            )),
        };
        write_file(base_path.join("app/models/user.py"), &user_model)?;

        // Schemas (Marshmallow for serialization)
        write_file(base_path.join("app/schemas/__init__.py"), "")?;
        let user_schema = replace_template_vars_string(USER_SCHEMA, &vars);
        write_file(base_path.join("app/schemas/user.py"), &user_schema)?;

        // Services
        write_file(base_path.join("app/services/__init__.py"), "")?;
        let user_service = replace_template_vars_string(USER_SERVICE, &vars);
        write_file(base_path.join("app/services/user_service.py"), &user_service)?;

        // Utils
        write_file(base_path.join("app/utils/__init__.py"), "")?;
        let decorators = replace_template_vars_string(DECORATORS, &vars);
        write_file(base_path.join("app/utils/decorators.py"), &decorators)?;

        Ok(())
    }

    fn generate_docker_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        if !config.include_docker {
            return Ok(());
        }

        let vars = self.get_template_vars(config);

        // Requirements - choose based on database type
        let requirements = match config.database {
            DatabaseType::MySQL => REQUIREMENTS_TXT_MYSQL,
            DatabaseType::PostgreSQL => REQUIREMENTS_TXT,
            _ => return Err(crate::athena::AthenaError::ValidationError(
                "Unsupported database type for Flask".to_string()
            )),
        };
        write_file(base_path.join("requirements.txt"), requirements)?;

        // Dockerfile - choose based on database type
        let dockerfile_template = match config.database {
            DatabaseType::MySQL => DOCKERFILE_MYSQL,
            DatabaseType::PostgreSQL => DOCKERFILE,
            _ => return Err(crate::athena::AthenaError::ValidationError(
                "Unsupported database type for Flask".to_string()
            )),
        };
        let dockerfile_content = replace_template_vars_string(dockerfile_template, &vars);
        write_file(base_path.join("Dockerfile"), &dockerfile_content)?;

        // Docker Compose - choose based on database type
        let compose_template = match config.database {
            DatabaseType::MySQL => DOCKER_COMPOSE_YML_MYSQL,
            DatabaseType::PostgreSQL => DOCKER_COMPOSE_YML,
            _ => return Err(crate::athena::AthenaError::ValidationError(
                "Unsupported database type for Flask".to_string()
            )),
        };
        let compose_content = replace_template_vars_string(compose_template, &vars);
        write_file(base_path.join("docker-compose.yml"), &compose_content)?;

        // Nginx configurations
        let nginx_content = replace_template_vars_string(NGINX_CONF, &vars);
        write_file(base_path.join("nginx/nginx.conf"), &nginx_content)?;
        
        let nginx_default_content = replace_template_vars_string(NGINX_DEFAULT_CONF, &vars);
        write_file(base_path.join("nginx/conf.d/default.conf"), &nginx_default_content)?;

        // .env template - choose based on database type
        let names = ProjectNames::new(&config.name);
        let env_template = match config.database {
            DatabaseType::MySQL => format!(r#"# Environment Configuration
FLASK_ENV=development
FLASK_DEBUG=1

# Security
SECRET_KEY=your-secret-key-here-change-in-production

# Database
DATABASE_URL=mysql+pymysql://root:yourpassword@localhost/{}_db
MYSQL_ROOT_PASSWORD=your-mysql-root-password
MYSQL_PASSWORD=your-mysql-password

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET_KEY=your-jwt-secret-key-here
JWT_ACCESS_TOKEN_EXPIRES=3600
JWT_REFRESH_TOKEN_EXPIRES=2592000

# CORS
ALLOWED_ORIGINS=http://localhost:3000,http://127.0.0.1:3000
"#, names.snake_case),
            DatabaseType::PostgreSQL => format!(r#"# Environment Configuration
FLASK_ENV=development
FLASK_DEBUG=1

# Security
SECRET_KEY=your-secret-key-here-change-in-production

# Database
DATABASE_URL=postgresql://user:password@localhost/{}_db
POSTGRES_PASSWORD=your-postgres-password

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET_KEY=your-jwt-secret-key-here
JWT_ACCESS_TOKEN_EXPIRES=3600
JWT_REFRESH_TOKEN_EXPIRES=2592000

# CORS
ALLOWED_ORIGINS=http://localhost:3000,http://127.0.0.1:3000
"#, names.snake_case),
            _ => return Err(crate::athena::AthenaError::ValidationError(
                "Unsupported database type for Flask".to_string()
            )),
        };
        write_file(base_path.join(".env.example"), &env_template)?;

        Ok(())
    }

    fn generate_test_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        write_file(base_path.join("tests/__init__.py"), "")?;

        // Test configuration
        let test_config = replace_template_vars_string(TEST_CONFIG, &vars);
        write_file(base_path.join("tests/conftest.py"), &test_config)?;

        // Main tests
        let test_main = replace_template_vars_string(TEST_MAIN, &vars);
        write_file(base_path.join("tests/test_main.py"), &test_main)?;

        // Auth tests
        let test_auth = replace_template_vars_string(TEST_AUTH, &vars);
        write_file(base_path.join("tests/test_auth.py"), &test_auth)?;

        Ok(())
    }

    fn generate_documentation(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let names = ProjectNames::new(&config.name);

        let database_name = match config.database {
            DatabaseType::MySQL => "MySQL",
            DatabaseType::PostgreSQL => "PostgreSQL",
            _ => "Database",
        };

        let database_config = match config.database {
            DatabaseType::MySQL => format!("DATABASE_URL=mysql+pymysql://root:password@localhost/{}_db\nMYSQL_ROOT_PASSWORD=your-mysql-password", names.snake_case),
            DatabaseType::PostgreSQL => format!("DATABASE_URL=postgresql://user:password@localhost/{}_db\nPOSTGRES_PASSWORD=your-postgres-password", names.snake_case),
            _ => "DATABASE_URL=your-database-url".to_string(),
        };

        let readme = format!(r#"# {project_name}

Production-ready Flask application with authentication, {database_name}, and Nginx.

## Features

- **Flask** - Micro web framework with flexibility
- **JWT Authentication** - Access & refresh tokens with Flask-JWT-Extended
- **Password Security** - Werkzeug password hashing
- **{database_name}** - Robust relational database with SQLAlchemy ORM
- **Docker** - Containerized deployment with multi-stage builds
- **Nginx** - Reverse proxy with security headers
- **Tests** - Comprehensive test suite with pytest
- **Security** - CORS, rate limiting, input validation
- **Migrations** - Database migrations with Flask-Migrate

## Quick Start

### Development

```bash
# Install dependencies
pip install -r requirements.txt

# Set up environment
cp .env.example .env
# Edit .env with your configuration

# Initialize database
flask db init
flask db migrate -m "Initial migration"
flask db upgrade

# Run the application
python run.py
```

### With Docker

```bash
# Build and run with Docker Compose
docker-compose up --build

# The API will be available at http://localhost
```

## API Documentation

Once running, the API will be available at:
- **Base URL**: http://localhost:5000
- **Health Check**: http://localhost:5000/health

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
│   │   └── v1/             # API version 1
│   ├── core/               # Core functionality
│   ├── database/           # Database connection
│   ├── models/             # SQLAlchemy models
│   ├── schemas/            # Marshmallow schemas
│   ├── services/           # Business logic
│   ├── utils/              # Utilities and decorators
│   └── __init__.py        # Flask application factory
├── tests/                  # Test suite
├── migrations/             # Database migrations
├── nginx/                  # Nginx configuration
├── logs/                   # Application logs
├── instance/               # Instance-specific files
├── requirements.txt        # Python dependencies
├── Dockerfile             # Docker configuration
├── docker-compose.yml     # Docker Compose setup
└── run.py                 # Application entry point
```

## Configuration

Key environment variables:

```env
FLASK_ENV=development
FLASK_DEBUG=1
SECRET_KEY=your-secret-key-here
{database_config}
REDIS_URL=redis://localhost:6379
JWT_SECRET_KEY=your-jwt-secret-key-here
ALLOWED_ORIGINS=http://localhost:3000
```

## Security Features

- JWT-based authentication with refresh tokens
- Password hashing with Werkzeug
- CORS configuration
- Security headers via Nginx
- Rate limiting with Flask-Limiter
- Input validation with Marshmallow
- SQL injection prevention with SQLAlchemy ORM

## Database Migrations

```bash
# Create a new migration
flask db migrate -m "Description of changes"

# Apply migrations
flask db upgrade

# Downgrade migrations
flask db downgrade
```

## Deployment

1. Set `FLASK_ENV=production` and `FLASK_DEBUG=0`
2. Use a strong `SECRET_KEY` and `JWT_SECRET_KEY`
3. Configure your PostgreSQL database connection
4. Set up SSL certificates for HTTPS
5. Configure firewall rules
6. Use a WSGI server like Gunicorn in production

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new features
4. Run the test suite
5. Submit a pull request

Generated with love by Athena CLI
"#,
            project_name = config.name,
            snake_case = names.snake_case
        );
        
        write_file(base_path.join("README.md"), &readme)?;

        Ok(())
    }
}

impl BoilerplateGenerator for FlaskGenerator {
    fn validate_config(&self, config: &ProjectConfig) -> BoilerplateResult<()> {
        crate::boilerplate::validate_project_name(&config.name)?;
        crate::boilerplate::check_directory_availability(Path::new(&config.directory))?;
        Ok(())
    }

    fn generate_project(&self, config: &ProjectConfig) -> BoilerplateResult<()> {
        let base_path = Path::new(&config.directory);

        println!("Generating Flask project: {}", config.name);
        
        // Create directory structure
        println!("  Creating directory structure...");
        self.create_flask_structure(base_path)?;

        // Generate core files
        println!("  Generating core application files...");
        self.generate_core_files(config, base_path)?;

        // Generate API files
        println!("  Generating API endpoints...");
        self.generate_api_files(config, base_path)?;

        // Generate database files
        let db_name = match config.database {
            DatabaseType::MySQL => "MySQL",
            DatabaseType::PostgreSQL => "PostgreSQL", 
            _ => "database",
        };
        println!("  💾 Setting up {} integration...", db_name);
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

        println!("Flask project '{}' created successfully!", config.name);
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
            println!("  flask db init && flask db migrate -m 'Initial migration' && flask db upgrade");
            println!("  python run.py");
        }

        Ok(())
    }
}