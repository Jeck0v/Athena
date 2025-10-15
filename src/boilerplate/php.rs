//! PHP boilerplate generator with Laravel and Symfony support using Clean Architecture

use crate::boilerplate::{BoilerplateGenerator, BoilerplateResult, ProjectConfig};
use crate::boilerplate::utils::{create_directory_structure, write_file, replace_template_vars_string, generate_secret_key, ProjectNames};
use crate::boilerplate::templates::php::*;
use std::path::Path;

/// Simple base64 encoding (without external dependency)
fn base64_encode(input: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let bytes = input.as_bytes();
    
    for chunk in bytes.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }
        
        let b = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
        
        result.push(ALPHABET[((b >> 18) & 63) as usize] as char);
        result.push(ALPHABET[((b >> 12) & 63) as usize] as char);
        result.push(if chunk.len() > 1 { ALPHABET[((b >> 6) & 63) as usize] as char } else { '=' });
        result.push(if chunk.len() > 2 { ALPHABET[(b & 63) as usize] as char } else { '=' });
    }
    
    result
}

pub struct PhpGenerator;

#[derive(Debug, Clone)]
pub enum PhpFramework {
    Laravel,
    Symfony,
}

impl Default for PhpGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl PhpGenerator {
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
            ("app_key", format!("base64:{}", base64_encode(&generate_secret_key()))),
        ]
    }

    fn create_laravel_structure(&self, base_path: &Path) -> BoilerplateResult<()> {
        let directories = vec![
            // Laravel-specific clean architecture structure
            "app/Domain/User/Entities",
            "app/Domain/User/Repositories",
            "app/Domain/User/Services",
            "app/Domain/User/ValueObjects",
            "app/Domain/Auth/Services",
            "app/Application/User/Commands",
            "app/Application/User/Queries",
            "app/Application/User/Handlers",
            "app/Application/Auth/Commands",
            "app/Application/Auth/Handlers",
            "app/Infrastructure/Persistence/Eloquent",
            "app/Infrastructure/Http/Controllers/Api/V1",
            "app/Infrastructure/Http/Middleware",
            "app/Infrastructure/Http/Requests",
            "app/Infrastructure/Http/Resources",
            "app/Infrastructure/Providers",
            "app/Infrastructure/Exceptions",
            "app/Shared/Events",
            "app/Shared/Notifications",
            "app/Shared/Services",
            "config",
            "database/migrations",
            "database/seeders",
            "routes",
            "tests/Unit/Domain",
            "tests/Unit/Application",
            "tests/Feature/Api",
            "tests/Integration",
            "storage/logs",
            "public",
            "resources/lang",
            "docker/nginx",
            "docker/php",
        ];

        create_directory_structure(base_path, &directories)
    }

    fn create_symfony_structure(&self, base_path: &Path) -> BoilerplateResult<()> {
        let directories = vec![
            // Symfony-specific hexagonal architecture structure
            "src/Domain/User/Entity",
            "src/Domain/User/Repository",
            "src/Domain/User/Service",
            "src/Domain/User/ValueObject",
            "src/Domain/Auth/Service",
            "src/Application/User/Command",
            "src/Application/User/Query",
            "src/Application/User/Handler",
            "src/Application/Auth/Command",
            "src/Application/Auth/Handler",
            "src/Infrastructure/Persistence/Doctrine",
            "src/Infrastructure/Http/Controller/Api/V1",
            "src/Infrastructure/Http/EventListener",
            "src/Infrastructure/Security",
            "src/Infrastructure/Serializer",
            "src/Shared/Event",
            "src/Shared/Service",
            "config/packages",
            "config/routes",
            "migrations",
            "tests/Unit/Domain",
            "tests/Unit/Application",
            "tests/Integration/Infrastructure",
            "tests/Functional/Api",
            "var/log",
            "public",
            "translations",
            "docker/nginx",
            "docker/php",
            "templates",
        ];

        create_directory_structure(base_path, &directories)
    }


    fn generate_laravel_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        // Generate Laravel-specific files
        let composer_json = replace_template_vars_string(LARAVEL_COMPOSER_JSON, &vars);
        write_file(base_path.join("composer.json"), &composer_json)?;

        let artisan = replace_template_vars_string(LARAVEL_ARTISAN, &vars);
        write_file(base_path.join("artisan"), &artisan)?;

        // Laravel configuration files
        let app_config = replace_template_vars_string(LARAVEL_APP_CONFIG, &vars);
        write_file(base_path.join("config/app.php"), &app_config)?;

        let database_config = replace_template_vars_string(LARAVEL_DATABASE_CONFIG, &vars);
        write_file(base_path.join("config/database.php"), &database_config)?;

        let auth_config = replace_template_vars_string(LARAVEL_AUTH_CONFIG, &vars);
        write_file(base_path.join("config/auth.php"), &auth_config)?;

        // Clean Architecture - Domain Layer
        self.generate_laravel_domain_layer(config, base_path)?;
        
        // Clean Architecture - Application Layer
        self.generate_laravel_application_layer(config, base_path)?;
        
        // Clean Architecture - Infrastructure Layer
        self.generate_laravel_infrastructure_layer(config, base_path)?;

        Ok(())
    }

    fn generate_laravel_domain_layer(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        // User Entity
        let user_entity = replace_template_vars_string(LARAVEL_USER_ENTITY, &vars);
        write_file(base_path.join("app/Domain/User/Entities/User.php"), &user_entity)?;

        // User Repository Interface
        let user_repository = replace_template_vars_string(LARAVEL_USER_REPOSITORY, &vars);
        write_file(base_path.join("app/Domain/User/Repositories/UserRepositoryInterface.php"), &user_repository)?;

        // User Service
        let user_service = replace_template_vars_string(LARAVEL_USER_SERVICE, &vars);
        write_file(base_path.join("app/Domain/User/Services/UserService.php"), &user_service)?;

        // Auth Service
        let auth_service = replace_template_vars_string(LARAVEL_AUTH_SERVICE, &vars);
        write_file(base_path.join("app/Domain/Auth/Services/AuthService.php"), &auth_service)?;

        Ok(())
    }

    fn generate_laravel_application_layer(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        // User Commands
        let create_user_command = replace_template_vars_string(LARAVEL_CREATE_USER_COMMAND, &vars);
        write_file(base_path.join("app/Application/User/Commands/CreateUserCommand.php"), &create_user_command)?;

        // User Queries
        let get_user_query = replace_template_vars_string(LARAVEL_GET_USER_QUERY, &vars);
        write_file(base_path.join("app/Application/User/Queries/GetUserQuery.php"), &get_user_query)?;

        // User Handlers
        let user_handler = replace_template_vars_string(LARAVEL_USER_HANDLER, &vars);
        write_file(base_path.join("app/Application/User/Handlers/UserHandler.php"), &user_handler)?;

        // Auth Commands & Handlers
        let login_command = replace_template_vars_string(LARAVEL_LOGIN_COMMAND, &vars);
        write_file(base_path.join("app/Application/Auth/Commands/LoginCommand.php"), &login_command)?;

        let auth_handler = replace_template_vars_string(LARAVEL_AUTH_HANDLER, &vars);
        write_file(base_path.join("app/Application/Auth/Handlers/AuthHandler.php"), &auth_handler)?;

        Ok(())
    }

    fn generate_laravel_infrastructure_layer(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        // Eloquent Repository Implementation
        let eloquent_user_repository = replace_template_vars_string(LARAVEL_ELOQUENT_USER_REPOSITORY, &vars);
        write_file(base_path.join("app/Infrastructure/Persistence/Eloquent/EloquentUserRepository.php"), &eloquent_user_repository)?;

        // API Controllers
        let user_controller = replace_template_vars_string(LARAVEL_USER_CONTROLLER, &vars);
        write_file(base_path.join("app/Infrastructure/Http/Controllers/Api/V1/UserController.php"), &user_controller)?;

        let auth_controller = replace_template_vars_string(LARAVEL_AUTH_CONTROLLER, &vars);
        write_file(base_path.join("app/Infrastructure/Http/Controllers/Api/V1/AuthController.php"), &auth_controller)?;

        // HTTP Requests
        let register_request = replace_template_vars_string(LARAVEL_REGISTER_REQUEST, &vars);
        write_file(base_path.join("app/Infrastructure/Http/Requests/RegisterRequest.php"), &register_request)?;

        let login_request = replace_template_vars_string(LARAVEL_LOGIN_REQUEST, &vars);
        write_file(base_path.join("app/Infrastructure/Http/Requests/LoginRequest.php"), &login_request)?;

        // API Routes
        let api_routes = replace_template_vars_string(LARAVEL_API_ROUTES, &vars);
        write_file(base_path.join("routes/api.php"), &api_routes)?;

        // Service Provider
        let app_service_provider = replace_template_vars_string(LARAVEL_APP_SERVICE_PROVIDER, &vars);
        write_file(base_path.join("app/Infrastructure/Providers/AppServiceProvider.php"), &app_service_provider)?;

        Ok(())
    }

    fn generate_symfony_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        // Generate Symfony-specific files
        let composer_json = replace_template_vars_string(SYMFONY_COMPOSER_JSON, &vars);
        write_file(base_path.join("composer.json"), &composer_json)?;

        // Symfony configuration
        let services_yaml = replace_template_vars_string(SYMFONY_SERVICES_YAML, &vars);
        write_file(base_path.join("config/services.yaml"), &services_yaml)?;

        let doctrine_yaml = replace_template_vars_string(SYMFONY_DOCTRINE_CONFIG, &vars);
        write_file(base_path.join("config/packages/doctrine.yaml"), &doctrine_yaml)?;

        let security_yaml = replace_template_vars_string(SYMFONY_SECURITY_CONFIG, &vars);
        write_file(base_path.join("config/packages/security.yaml"), &security_yaml)?;

        // Hexagonal Architecture - Domain Layer
        self.generate_symfony_domain_layer(config, base_path)?;
        
        // Hexagonal Architecture - Application Layer
        self.generate_symfony_application_layer(config, base_path)?;
        
        // Hexagonal Architecture - Infrastructure Layer
        self.generate_symfony_infrastructure_layer(config, base_path)?;

        Ok(())
    }

    fn generate_symfony_domain_layer(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        // User Entity
        let user_entity = replace_template_vars_string(SYMFONY_USER_ENTITY, &vars);
        write_file(base_path.join("src/Domain/User/Entity/User.php"), &user_entity)?;

        // User Repository Interface
        let user_repository = replace_template_vars_string(SYMFONY_USER_REPOSITORY, &vars);
        write_file(base_path.join("src/Domain/User/Repository/UserRepositoryInterface.php"), &user_repository)?;

        // User Service
        let user_service = replace_template_vars_string(SYMFONY_USER_SERVICE, &vars);
        write_file(base_path.join("src/Domain/User/Service/UserService.php"), &user_service)?;

        // Auth Service
        let auth_service = replace_template_vars_string(SYMFONY_AUTH_SERVICE, &vars);
        write_file(base_path.join("src/Domain/Auth/Service/AuthService.php"), &auth_service)?;

        Ok(())
    }

    fn generate_symfony_application_layer(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        // User Commands
        let create_user_command = replace_template_vars_string(SYMFONY_CREATE_USER_COMMAND, &vars);
        write_file(base_path.join("src/Application/User/Command/CreateUserCommand.php"), &create_user_command)?;

        // User Queries
        let get_user_query = replace_template_vars_string(SYMFONY_GET_USER_QUERY, &vars);
        write_file(base_path.join("src/Application/User/Query/GetUserQuery.php"), &get_user_query)?;

        // User Handlers
        let user_handler = replace_template_vars_string(SYMFONY_USER_HANDLER, &vars);
        write_file(base_path.join("src/Application/User/Handler/UserHandler.php"), &user_handler)?;

        // Auth Commands & Handlers
        let login_command = replace_template_vars_string(SYMFONY_LOGIN_COMMAND, &vars);
        write_file(base_path.join("src/Application/Auth/Command/LoginCommand.php"), &login_command)?;

        let auth_handler = replace_template_vars_string(SYMFONY_AUTH_HANDLER, &vars);
        write_file(base_path.join("src/Application/Auth/Handler/AuthHandler.php"), &auth_handler)?;

        Ok(())
    }

    fn generate_symfony_infrastructure_layer(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        // Doctrine Repository Implementation
        let doctrine_user_repository = replace_template_vars_string(SYMFONY_DOCTRINE_USER_REPOSITORY, &vars);
        write_file(base_path.join("src/Infrastructure/Persistence/Doctrine/DoctrineUserRepository.php"), &doctrine_user_repository)?;

        // API Controllers
        let user_controller = replace_template_vars_string(SYMFONY_USER_CONTROLLER, &vars);
        write_file(base_path.join("src/Infrastructure/Http/Controller/Api/V1/UserController.php"), &user_controller)?;

        let auth_controller = replace_template_vars_string(SYMFONY_AUTH_CONTROLLER, &vars);
        write_file(base_path.join("src/Infrastructure/Http/Controller/Api/V1/AuthController.php"), &auth_controller)?;

        // API Routes
        let api_routes = replace_template_vars_string(SYMFONY_API_ROUTES, &vars);
        write_file(base_path.join("config/routes/api.yaml"), &api_routes)?;

        Ok(())
    }

    fn generate_docker_files(&self, config: &ProjectConfig, base_path: &Path, framework: &PhpFramework) -> BoilerplateResult<()> {
        if !config.include_docker {
            return Ok(());
        }

        let vars = self.get_template_vars(config);

        // Optimized PHP Dockerfile
        let dockerfile_content = replace_template_vars_string(PHP_OPTIMIZED_DOCKERFILE, &vars);
        write_file(base_path.join("docker/php/Dockerfile"), &dockerfile_content)?;

        // Nginx Dockerfile
        let nginx_dockerfile = replace_template_vars_string(PHP_NGINX_DOCKERFILE, &vars);
        write_file(base_path.join("docker/nginx/Dockerfile"), &nginx_dockerfile)?;

        // Docker Compose (production-ready)
        let docker_compose = match framework {
            PhpFramework::Laravel => replace_template_vars_string(LARAVEL_DOCKER_COMPOSE, &vars),
            PhpFramework::Symfony => replace_template_vars_string(SYMFONY_DOCKER_COMPOSE, &vars),
        };
        write_file(base_path.join("docker-compose.yml"), &docker_compose)?;

        // Docker Compose Development Override
        let docker_compose_dev = replace_template_vars_string(PHP_DOCKER_COMPOSE_DEV, &vars);
        write_file(base_path.join("docker-compose.dev.yml"), &docker_compose_dev)?;

        // Environment files
        let env_docker = replace_template_vars_string(PHP_ENV_DOCKER, &vars);
        write_file(base_path.join(".env.docker.example"), &env_docker)?;

        let env_example = match framework {
            PhpFramework::Laravel => replace_template_vars_string(LARAVEL_ENV_EXAMPLE, &vars),
            PhpFramework::Symfony => replace_template_vars_string(SYMFONY_ENV_EXAMPLE, &vars),
        };
        write_file(base_path.join(".env.example"), &env_example)?;

        // Nginx configuration
        let nginx_conf = replace_template_vars_string(PHP_NGINX_CONF, &vars);
        write_file(base_path.join("docker/nginx/nginx.conf"), &nginx_conf)?;

        let nginx_default_conf = match framework {
            PhpFramework::Laravel => replace_template_vars_string(LARAVEL_NGINX_DEFAULT_CONF, &vars),
            PhpFramework::Symfony => replace_template_vars_string(SYMFONY_NGINX_DEFAULT_CONF, &vars),
        };
        write_file(base_path.join("docker/nginx/default.conf"), &nginx_default_conf)?;

        // PHP-FPM configuration
        let php_fpm_conf = replace_template_vars_string(PHP_FPM_CONF, &vars);
        write_file(base_path.join("docker/php/php-fpm.conf"), &php_fpm_conf)?;

        Ok(())
    }

    fn generate_test_files(&self, config: &ProjectConfig, base_path: &Path, framework: &PhpFramework) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        match framework {
            PhpFramework::Laravel => {
                let phpunit_xml = replace_template_vars_string(LARAVEL_PHPUNIT_XML, &vars);
                write_file(base_path.join("phpunit.xml"), &phpunit_xml)?;

                let feature_test = replace_template_vars_string(LARAVEL_AUTH_FEATURE_TEST, &vars);
                write_file(base_path.join("tests/Feature/Api/AuthTest.php"), &feature_test)?;

                let unit_test = replace_template_vars_string(LARAVEL_USER_UNIT_TEST, &vars);
                write_file(base_path.join("tests/Unit/Domain/UserTest.php"), &unit_test)?;
            }
            PhpFramework::Symfony => {
                let phpunit_xml = replace_template_vars_string(SYMFONY_PHPUNIT_XML, &vars);
                write_file(base_path.join("phpunit.xml.dist"), &phpunit_xml)?;

                let functional_test = replace_template_vars_string(SYMFONY_AUTH_FUNCTIONAL_TEST, &vars);
                write_file(base_path.join("tests/Functional/Api/AuthTest.php"), &functional_test)?;

                let unit_test = replace_template_vars_string(SYMFONY_USER_UNIT_TEST, &vars);
                write_file(base_path.join("tests/Unit/Domain/UserTest.php"), &unit_test)?;
            }
        }

        Ok(())
    }

    fn generate_documentation(&self, config: &ProjectConfig, base_path: &Path, framework: &PhpFramework) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        let readme = match framework {
            PhpFramework::Laravel => replace_template_vars_string(LARAVEL_README, &vars),
            PhpFramework::Symfony => replace_template_vars_string(SYMFONY_README, &vars),
        };
        
        write_file(base_path.join("README.md"), &readme)?;

        Ok(())
    }

    pub fn generate_laravel_project(&self, config: &ProjectConfig) -> BoilerplateResult<()> {
        let base_path = Path::new(&config.directory);
        let framework = PhpFramework::Laravel;

        println!("Generating Laravel project with Clean Architecture: {}", config.name);
        
        // Create directory structure
        println!("  📁 Creating clean architecture structure...");
        self.create_laravel_structure(base_path)?;

        // Generate Laravel files
        println!("  ⚡ Generating Laravel application files...");
        self.generate_laravel_files(config, base_path)?;

        // Generate Docker files
        if config.include_docker {
            println!("  🐳 Generating Docker configuration...");
            self.generate_docker_files(config, base_path, &framework)?;
        }

        // Generate test files
        println!("  🧪 Creating test suite...");
        self.generate_test_files(config, base_path, &framework)?;

        // Generate documentation
        println!("  📚 Generating documentation...");
        self.generate_documentation(config, base_path, &framework)?;

        println!("Laravel project '{}' created successfully!", config.name);
        println!("📍 Location: {}", base_path.display());
        
        if config.include_docker {
            println!("\n🔧 Next steps:");
            println!("  cd {}", config.directory);
            println!("  cp .env.example .env  # Edit with your configuration");
            println!("  docker-compose up --build");
            println!("  docker-compose exec app composer install");
            println!("  docker-compose exec app php artisan migrate");
        } else {
            println!("\n🔧 Next steps:");
            println!("  cd {}", config.directory);
            println!("  composer install");
            println!("  cp .env.example .env  # Edit with your configuration");
            println!("  php artisan key:generate");
            println!("  php artisan migrate");
            println!("  php artisan serve");
        }

        Ok(())
    }

    pub fn generate_symfony_project(&self, config: &ProjectConfig) -> BoilerplateResult<()> {
        let base_path = Path::new(&config.directory);
        let framework = PhpFramework::Symfony;

        println!("Generating Symfony project with Hexagonal Architecture: {}", config.name);
        
        // Create directory structure
        println!("  📁 Creating hexagonal architecture structure...");
        self.create_symfony_structure(base_path)?;

        // Generate Symfony files
        println!("  🎼 Generating Symfony application files...");
        self.generate_symfony_files(config, base_path)?;

        // Generate Docker files
        if config.include_docker {
            println!("  🐳 Generating Docker configuration...");
            self.generate_docker_files(config, base_path, &framework)?;
        }

        // Generate test files
        println!("  🧪 Creating test suite...");
        self.generate_test_files(config, base_path, &framework)?;

        // Generate documentation
        println!("  📚 Generating documentation...");
        self.generate_documentation(config, base_path, &framework)?;

        println!("Symfony project '{}' created successfully!", config.name);
        println!("📍 Location: {}", base_path.display());
        
        if config.include_docker {
            println!("\n🔧 Next steps:");
            println!("  cd {}", config.directory);
            println!("  cp .env.example .env  # Edit with your configuration");
            println!("  docker-compose up --build");
            println!("  docker-compose exec app composer install");
            println!("  docker-compose exec app php bin/console doctrine:migrations:migrate");
        } else {
            println!("\n🔧 Next steps:");
            println!("  cd {}", config.directory);
            println!("  composer install");
            println!("  cp .env.example .env  # Edit with your configuration");
            println!("  php bin/console doctrine:migrations:migrate");
            println!("  symfony server:start");
        }

        Ok(())
    }
}

impl BoilerplateGenerator for PhpGenerator {
    fn validate_config(&self, config: &ProjectConfig) -> BoilerplateResult<()> {
        crate::boilerplate::validate_project_name(&config.name)?;
        crate::boilerplate::check_directory_availability(Path::new(&config.directory))?;
        Ok(())
    }

    fn generate_project(&self, config: &ProjectConfig) -> BoilerplateResult<()> {
        // Default to Laravel, but this can be extended to support framework selection
        self.generate_laravel_project(config)
    }
}