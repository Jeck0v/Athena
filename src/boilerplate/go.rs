//! Go boilerplate generator with production-ready features

use crate::boilerplate::{BoilerplateGenerator, BoilerplateResult, ProjectConfig, DatabaseType};
use crate::boilerplate::utils::{create_directory_structure, write_file, replace_template_vars_string, generate_secret_key, ProjectNames};
use crate::boilerplate::templates::go::*;
use std::path::Path;

pub struct GoGenerator;

impl Default for GoGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl GoGenerator {
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
            ("module_name", format!("github.com/yourusername/{}", names.kebab_case)),
        ]
    }

    fn create_go_structure(&self, base_path: &Path) -> BoilerplateResult<()> {
        let directories = vec![
            "cmd",
            "internal/config",
            "internal/database",
            "internal/handlers",
            "internal/middleware",
            "internal/models",
            "internal/routes",
            "internal/services",
            "internal/utils",
            "pkg/auth",
            "pkg/errors",
            "pkg/logger",
            "tests",
            "tests/integration",
            "tests/unit",
            "deployments",
            "scripts",
            "docs",
        ];

        create_directory_structure(base_path, &directories)
    }

    fn generate_main_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        // Main application
        let main_content = replace_template_vars_string(MAIN_GO, &vars);
        write_file(base_path.join("cmd/main.go"), &main_content)?;

        // Go module
        let go_mod_content = replace_template_vars_string(GO_MOD, &vars);
        write_file(base_path.join("go.mod"), &go_mod_content)?;

        Ok(())
    }

    fn generate_config_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        let config_go = r#"package config

import (
	"os"
	"strconv"
	"strings"
)

type Config struct {
	// Application
	ProjectName string
	Version     string
	Environment string
	Port        string

	// Security
	JWTSecret                string
	AccessTokenExpireMinutes int
	RefreshTokenExpireDays   int

	// Database{{#if mongodb}}
	MongoURL      string
	DatabaseName  string{{/if}}{{#if postgresql}}
	DatabaseURL   string{{/if}}

	// Redis
	RedisURL string

	// CORS
	AllowedOrigins []string

	// Rate Limiting
	RateLimitRPS int
}

func Load() *Config {
	return &Config{
		ProjectName: getEnv("PROJECT_NAME", "{{project_name}}"),
		Version:     getEnv("VERSION", "1.0.0"),
		Environment: getEnv("ENVIRONMENT", "development"),
		Port:        getEnv("PORT", "8080"),

		JWTSecret:                getEnv("JWT_SECRET", "{{secret_key}}"),
		AccessTokenExpireMinutes: getEnvAsInt("ACCESS_TOKEN_EXPIRE_MINUTES", 30),
		RefreshTokenExpireDays:   getEnvAsInt("REFRESH_TOKEN_EXPIRE_DAYS", 7),
{{#if mongodb}}
		MongoURL:     getEnv("MONGO_URL", "mongodb://localhost:27017"),
		DatabaseName: getEnv("DATABASE_NAME", "{{snake_case}}_db"),{{/if}}{{#if postgresql}}
		DatabaseURL:  getEnv("DATABASE_URL", "postgres://user:password@localhost/{{snake_case}}_db?sslmode=disable"),{{/if}}

		RedisURL: getEnv("REDIS_URL", "redis://localhost:6379"),

		AllowedOrigins: strings.Split(getEnv("ALLOWED_ORIGINS", "http://localhost:3000,http://127.0.0.1:3000"), ","),
		RateLimitRPS:   getEnvAsInt("RATE_LIMIT_RPS", 10),
	}
}

func getEnv(key, fallback string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return fallback
}

func getEnvAsInt(key string, fallback int) int {
	if value := os.Getenv(key); value != "" {
		if intValue, err := strconv.Atoi(value); err == nil {
			return intValue
		}
	}
	return fallback
}
"#;
        
        let mut config_content = config_go.to_string();
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
                return Err(crate::athena::AthenaError::validation_error_simple(
                    "MySQL is not supported for Go projects. Use Flask for MySQL support.".to_string()
                ));
            }
        }
        config_content = replace_template_vars_string(&config_content, &vars);
        write_file(base_path.join("internal/config/config.go"), &config_content)?;

        Ok(())
    }

    fn generate_database_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        match config.database {
            DatabaseType::MongoDB => {
                let mongodb_go = r#"package database

import (
	"context"
	"log"
	"time"

	"{{module_name}}/internal/config"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

type Database struct {
	Client   *mongo.Client
	Database *mongo.Database
}

func Connect(cfg *config.Config) (*Database, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	// Connect to MongoDB
	client, err := mongo.Connect(ctx, options.Client().ApplyURI(cfg.MongoURL))
	if err != nil {
		return nil, err
	}

	// Ping the database
	if err = client.Ping(ctx, nil); err != nil {
		return nil, err
	}

	database := client.Database(cfg.DatabaseName)
	
	log.Println("Successfully connected to MongoDB")
	
	return &Database{
		Client:   client,
		Database: database,
	}, nil
}

func Close(db *Database) {
	if db.Client != nil {
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		
		if err := db.Client.Disconnect(ctx); err != nil {
			log.Printf("Error disconnecting from MongoDB: %v", err)
		} else {
			log.Println("Disconnected from MongoDB")
		}
	}
}

func (db *Database) GetCollection(name string) *mongo.Collection {
	return db.Database.Collection(name)
}
"#;
                let mongodb_content = replace_template_vars_string(mongodb_go, &vars);
                write_file(base_path.join("internal/database/mongodb.go"), &mongodb_content)?;
            }
            DatabaseType::PostgreSQL => {
                let postgres_go = r#"package database

import (
	"database/sql"
	"log"

	"{{module_name}}/internal/config"
	_ "github.com/lib/pq"
	"github.com/jmoiron/sqlx"
)

type Database struct {
	DB *sqlx.DB
}

func Connect(cfg *config.Config) (*Database, error) {
	// Connect to PostgreSQL
	db, err := sqlx.Connect("postgres", cfg.DatabaseURL)
	if err != nil {
		return nil, err
	}

	// Test the connection
	if err = db.Ping(); err != nil {
		return nil, err
	}

	// Set connection pool settings
	db.SetMaxOpenConns(25)
	db.SetMaxIdleConns(5)

	log.Println("Successfully connected to PostgreSQL")
	
	// Run migrations
	if err = runMigrations(db); err != nil {
		log.Printf("Warning: Migration failed: %v", err)
	}

	return &Database{DB: db}, nil
}

func Close(db *Database) {
	if db.DB != nil {
		if err := db.DB.Close(); err != nil {
			log.Printf("Error closing database: %v", err)
		} else {
			log.Println("Database connection closed")
		}
	}
}

func runMigrations(db *sqlx.DB) error {
	// Create users table
	query := `
	CREATE TABLE IF NOT EXISTS users (
		id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
		email VARCHAR(255) UNIQUE NOT NULL,
		hashed_password VARCHAR(255) NOT NULL,
		full_name VARCHAR(255) NOT NULL,
		is_active BOOLEAN DEFAULT TRUE,
		created_at TIMESTAMP DEFAULT NOW(),
		updated_at TIMESTAMP DEFAULT NOW()
	);
	
	CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
	`
	
	_, err := db.Exec(query)
	if err != nil {
		return err
	}
	
	log.Println("Database migrations completed successfully")
	return nil
}
"#;
                let postgres_content = replace_template_vars_string(postgres_go, &vars);
                write_file(base_path.join("internal/database/postgres.go"), &postgres_content)?;
            }
            DatabaseType::MySQL => {
                return Err(crate::athena::AthenaError::validation_error_simple(
                    "MySQL is not supported for Go projects. Use Flask for MySQL support.".to_string()
                ));
            }
        }

        Ok(())
    }

    fn generate_auth_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        let auth_go = r#"package auth

import (
	"errors"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"golang.org/x/crypto/bcrypt"
)

type Claims struct {
	UserID string `json:"user_id"`
	Email  string `json:"email"`
	Type   string `json:"type"` // "access" or "refresh"
	jwt.RegisteredClaims
}

type TokenPair struct {
	AccessToken  string `json:"access_token"`
	RefreshToken string `json:"refresh_token"`
	TokenType    string `json:"token_type"`
}

func HashPassword(password string) (string, error) {
	bytes, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	return string(bytes), err
}

func CheckPassword(hashedPassword, password string) error {
	return bcrypt.CompareHashAndPassword([]byte(hashedPassword), []byte(password))
}

func GenerateTokenPair(userID, email, jwtSecret string, accessExpireMinutes, refreshExpireDays int) (*TokenPair, error) {
	// Generate access token
	accessToken, err := generateToken(userID, email, "access", jwtSecret, time.Duration(accessExpireMinutes)*time.Minute)
	if err != nil {
		return nil, err
	}

	// Generate refresh token
	refreshToken, err := generateToken(userID, email, "refresh", jwtSecret, time.Duration(refreshExpireDays)*24*time.Hour)
	if err != nil {
		return nil, err
	}

	return &TokenPair{
		AccessToken:  accessToken,
		RefreshToken: refreshToken,
		TokenType:    "bearer",
	}, nil
}

func generateToken(userID, email, tokenType, jwtSecret string, expiration time.Duration) (string, error) {
	claims := Claims{
		UserID: userID,
		Email:  email,
		Type:   tokenType,
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(expiration)),
			IssuedAt:  jwt.NewNumericDate(time.Now()),
		},
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return token.SignedString([]byte(jwtSecret))
}

func VerifyToken(tokenString, jwtSecret, expectedType string) (*Claims, error) {
	token, err := jwt.ParseWithClaims(tokenString, &Claims{}, func(token *jwt.Token) (interface{}, error) {
		return []byte(jwtSecret), nil
	})

	if err != nil {
		return nil, err
	}

	if claims, ok := token.Claims.(*Claims); ok && token.Valid {
		if claims.Type != expectedType {
			return nil, errors.New("invalid token type")
		}
		return claims, nil
	}

	return nil, errors.New("invalid token")
}
"#;

        let auth_content = replace_template_vars_string(auth_go, &vars);
        write_file(base_path.join("pkg/auth/auth.go"), &auth_content)?;

        Ok(())
    }

    fn generate_models_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let _vars = self.get_template_vars(config);

        let models_go = match config.database {
            DatabaseType::MySQL => {
                return Err(crate::athena::AthenaError::validation_error_simple(
                    "MySQL is not supported for Go projects. Use Flask for MySQL support.".to_string()
                ));
            }
            DatabaseType::MongoDB => r#"package models

import (
	"time"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

type User struct {
	ID             primitive.ObjectID `bson:"_id,omitempty" json:"id"`
	Email          string             `bson:"email" json:"email"`
	HashedPassword string             `bson:"hashed_password" json:"-"`
	FullName       string             `bson:"full_name" json:"full_name"`
	IsActive       bool               `bson:"is_active" json:"is_active"`
	CreatedAt      time.Time          `bson:"created_at" json:"created_at"`
	UpdatedAt      time.Time          `bson:"updated_at" json:"updated_at"`
}

type LoginRequest struct {
	Email    string `json:"email" binding:"required,email"`
	Password string `json:"password" binding:"required,min=6"`
}

type RegisterRequest struct {
	Email    string `json:"email" binding:"required,email"`
	Password string `json:"password" binding:"required,min=6"`
	FullName string `json:"full_name" binding:"required"`
}

type UserResponse struct {
	ID       string    `json:"id"`
	Email    string    `json:"email"`
	FullName string    `json:"full_name"`
	IsActive bool      `json:"is_active"`
	CreatedAt time.Time `json:"created_at"`
}

func (u *User) ToResponse() *UserResponse {
	return &UserResponse{
		ID:       u.ID.Hex(),
		Email:    u.Email,
		FullName: u.FullName,
		IsActive: u.IsActive,
		CreatedAt: u.CreatedAt,
	}
}
"#.to_string(),
            DatabaseType::PostgreSQL => r#"package models

import (
	"time"
	"github.com/google/uuid"
)

type User struct {
	ID             uuid.UUID `db:"id" json:"id"`
	Email          string    `db:"email" json:"email"`
	HashedPassword string    `db:"hashed_password" json:"-"`
	FullName       string    `db:"full_name" json:"full_name"`
	IsActive       bool      `db:"is_active" json:"is_active"`
	CreatedAt      time.Time `db:"created_at" json:"created_at"`
	UpdatedAt      time.Time `db:"updated_at" json:"updated_at"`
}

type LoginRequest struct {
	Email    string `json:"email" binding:"required,email"`
	Password string `json:"password" binding:"required,min=6"`
}

type RegisterRequest struct {
	Email    string `json:"email" binding:"required,email"`
	Password string `json:"password" binding:"required,min=6"`
	FullName string `json:"full_name" binding:"required"`
}

type UserResponse struct {
	ID       string    `json:"id"`
	Email    string    `json:"email"`
	FullName string    `json:"full_name"`
	IsActive bool      `json:"is_active"`
	CreatedAt time.Time `json:"created_at"`
}

func (u *User) ToResponse() *UserResponse {
	return &UserResponse{
		ID:       u.ID.String(),
		Email:    u.Email,
		FullName: u.FullName,
		IsActive: u.IsActive,
		CreatedAt: u.CreatedAt,
	}
}
"#.to_string()
        };

        write_file(base_path.join("internal/models/user.go"), &models_go)?;

        Ok(())
    }

    fn generate_middleware_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        let cors_go = r#"package middleware

import (
	"{{module_name}}/internal/config"
	"github.com/gin-gonic/gin"
)

func CORS() gin.HandlerFunc {
	return gin.HandlerFunc(func(c *gin.Context) {
		c.Writer.Header().Set("Access-Control-Allow-Origin", "*")
		c.Writer.Header().Set("Access-Control-Allow-Credentials", "true")
		c.Writer.Header().Set("Access-Control-Allow-Headers", "Content-Type, Content-Length, Accept-Encoding, X-CSRF-Token, Authorization, accept, origin, Cache-Control, X-Requested-With")
		c.Writer.Header().Set("Access-Control-Allow-Methods", "POST, OPTIONS, GET, PUT, DELETE")

		if c.Request.Method == "OPTIONS" {
			c.AbortWithStatus(204)
			return
		}

		c.Next()
	})
}
"#;

        let security_go = r#"package middleware

import (
	"github.com/gin-gonic/gin"
)

func Security() gin.HandlerFunc {
	return gin.HandlerFunc(func(c *gin.Context) {
		// Security headers
		c.Writer.Header().Set("X-Frame-Options", "DENY")
		c.Writer.Header().Set("X-Content-Type-Options", "nosniff")
		c.Writer.Header().Set("X-XSS-Protection", "1; mode=block")
		c.Writer.Header().Set("Referrer-Policy", "strict-origin-when-cross-origin")
		c.Writer.Header().Set("Content-Security-Policy", "default-src 'self'")

		c.Next()
	})
}
"#;

        let auth_middleware_go = r#"package middleware

import (
	"net/http"
	"strings"

	"{{module_name}}/internal/config"
	"{{module_name}}/pkg/auth"
	"github.com/gin-gonic/gin"
)

func AuthMiddleware(cfg *config.Config) gin.HandlerFunc {
	return gin.HandlerFunc(func(c *gin.Context) {
		authHeader := c.GetHeader("Authorization")
		if authHeader == "" {
			c.JSON(http.StatusUnauthorized, gin.H{"error": "Authorization header required"})
			c.Abort()
			return
		}

		// Check Bearer token format
		bearerToken := strings.Split(authHeader, " ")
		if len(bearerToken) != 2 || bearerToken[0] != "Bearer" {
			c.JSON(http.StatusUnauthorized, gin.H{"error": "Invalid authorization header format"})
			c.Abort()
			return
		}

		// Verify token
		claims, err := auth.VerifyToken(bearerToken[1], cfg.JWTSecret, "access")
		if err != nil {
			c.JSON(http.StatusUnauthorized, gin.H{"error": "Invalid token"})
			c.Abort()
			return
		}

		// Set user info in context
		c.Set("user_id", claims.UserID)
		c.Set("user_email", claims.Email)
		c.Next()
	})
}
"#;

        let cors_content = replace_template_vars_string(cors_go, &vars);
        let auth_middleware_content = replace_template_vars_string(auth_middleware_go, &vars);

        write_file(base_path.join("internal/middleware/cors.go"), &cors_content)?;
        write_file(base_path.join("internal/middleware/security.go"), security_go)?;
        write_file(base_path.join("internal/middleware/auth.go"), &auth_middleware_content)?;

        Ok(())
    }

    fn generate_handlers_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        let handlers_go = r#"package handlers

import (
	"{{module_name}}/internal/config"
	"{{module_name}}/internal/database"
	"{{module_name}}/internal/services"
)

type Handlers struct {
	Auth   *AuthHandler
	User   *UserHandler
	Health *HealthHandler
}

func New(db *database.Database, cfg *config.Config) *Handlers {
	userService := services.NewUserService(db)

	return &Handlers{
		Auth:   NewAuthHandler(userService, cfg),
		User:   NewUserHandler(userService),
		Health: NewHealthHandler(),
	}
}
"#;

        let auth_handler_go = r#"package handlers

import (
	"net/http"

	"{{module_name}}/internal/config"
	"{{module_name}}/internal/models"
	"{{module_name}}/internal/services"
	"{{module_name}}/pkg/auth"
	"github.com/gin-gonic/gin"
)

type AuthHandler struct {
	userService *services.UserService
	config      *config.Config
}

func NewAuthHandler(userService *services.UserService, cfg *config.Config) *AuthHandler {
	return &AuthHandler{
		userService: userService,
		config:      cfg,
	}
}

func (h *AuthHandler) Register(c *gin.Context) {
	var req models.RegisterRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// Check if user already exists
	existingUser, _ := h.userService.GetByEmail(req.Email)
	if existingUser != nil {
		c.JSON(http.StatusConflict, gin.H{"error": "User already exists"})
		return
	}

	// Hash password
	hashedPassword, err := auth.HashPassword(req.Password)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to process password"})
		return
	}

	// Create user
	user, err := h.userService.Create(&models.User{
		Email:          req.Email,
		HashedPassword: hashedPassword,
		FullName:       req.FullName,
		IsActive:       true,
	})
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to create user"})
		return
	}

	// Generate tokens
	tokens, err := auth.GenerateTokenPair(
		user.ID.String(),
		user.Email,
		h.config.JWTSecret,
		h.config.AccessTokenExpireMinutes,
		h.config.RefreshTokenExpireDays,
	)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to generate tokens"})
		return
	}

	c.JSON(http.StatusOK, tokens)
}

func (h *AuthHandler) Login(c *gin.Context) {
	var req models.LoginRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// Find user
	user, err := h.userService.GetByEmail(req.Email)
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Invalid credentials"})
		return
	}

	// Check password
	if err := auth.CheckPassword(user.HashedPassword, req.Password); err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Invalid credentials"})
		return
	}

	// Generate tokens
	tokens, err := auth.GenerateTokenPair(
		user.ID.String(),
		user.Email,
		h.config.JWTSecret,
		h.config.AccessTokenExpireMinutes,
		h.config.RefreshTokenExpireDays,
	)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to generate tokens"})
		return
	}

	c.JSON(http.StatusOK, tokens)
}

type RefreshRequest struct {
	RefreshToken string `json:"refresh_token" binding:"required"`
}

func (h *AuthHandler) RefreshToken(c *gin.Context) {
	var req RefreshRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// Verify refresh token
	claims, err := auth.VerifyToken(req.RefreshToken, h.config.JWTSecret, "refresh")
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Invalid refresh token"})
		return
	}

	// Get user
	user, err := h.userService.GetByID(claims.UserID)
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User not found"})
		return
	}

	// Generate new tokens
	tokens, err := auth.GenerateTokenPair(
		user.ID.String(),
		user.Email,
		h.config.JWTSecret,
		h.config.AccessTokenExpireMinutes,
		h.config.RefreshTokenExpireDays,
	)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to generate tokens"})
		return
	}

	c.JSON(http.StatusOK, tokens)
}
"#;

        let user_handler_go = r#"package handlers

import (
	"net/http"

	"{{module_name}}/internal/services"
	"github.com/gin-gonic/gin"
)

type UserHandler struct {
	userService *services.UserService
}

func NewUserHandler(userService *services.UserService) *UserHandler {
	return &UserHandler{
		userService: userService,
	}
}

func (h *UserHandler) GetMe(c *gin.Context) {
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "User ID not found"})
		return
	}

	user, err := h.userService.GetByID(userID.(string))
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "User not found"})
		return
	}

	c.JSON(http.StatusOK, user.ToResponse())
}
"#;

        let health_handler_go = r#"package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
)

type HealthHandler struct{}

func NewHealthHandler() *HealthHandler {
	return &HealthHandler{}
}

func (h *HealthHandler) Health(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{
		"status":  "healthy",
		"service": "{{project_name}} API",
		"version": "1.0.0",
	})
}
"#;

        let handlers_content = replace_template_vars_string(handlers_go, &vars);
        let auth_handler_content = replace_template_vars_string(auth_handler_go, &vars);
        let user_handler_content = replace_template_vars_string(user_handler_go, &vars);
        let health_handler_content = replace_template_vars_string(health_handler_go, &vars);

        write_file(base_path.join("internal/handlers/handlers.go"), &handlers_content)?;
        write_file(base_path.join("internal/handlers/auth.go"), &auth_handler_content)?;
        write_file(base_path.join("internal/handlers/user.go"), &user_handler_content)?;
        write_file(base_path.join("internal/handlers/health.go"), &health_handler_content)?;

        Ok(())
    }

    fn generate_services_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        let user_service_go = match config.database {
            DatabaseType::MySQL => {
                return Err(crate::athena::AthenaError::validation_error_simple(
                    "MySQL is not supported for Go projects. Use Flask for MySQL support.".to_string()
                ));
            }
            DatabaseType::MongoDB => r#"package services

import (
	"context"
	"time"

	"{{module_name}}/internal/database"
	"{{module_name}}/internal/models"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo"
)

type UserService struct {
	collection *mongo.Collection
}

func NewUserService(db *database.Database) *UserService {
	return &UserService{
		collection: db.GetCollection("users"),
	}
}

func (s *UserService) Create(user *models.User) (*models.User, error) {
	user.ID = primitive.NewObjectID()
	user.CreatedAt = time.Now()
	user.UpdatedAt = time.Now()

	_, err := s.collection.InsertOne(context.Background(), user)
	if err != nil {
		return nil, err
	}

	return user, nil
}

func (s *UserService) GetByID(id string) (*models.User, error) {
	objectID, err := primitive.ObjectIDFromHex(id)
	if err != nil {
		return nil, err
	}

	var user models.User
	err = s.collection.FindOne(context.Background(), bson.M{"_id": objectID}).Decode(&user)
	if err != nil {
		return nil, err
	}

	return &user, nil
}

func (s *UserService) GetByEmail(email string) (*models.User, error) {
	var user models.User
	err := s.collection.FindOne(context.Background(), bson.M{"email": email}).Decode(&user)
	if err != nil {
		return nil, err
	}

	return &user, nil
}

func (s *UserService) Update(id string, updateData bson.M) (*models.User, error) {
	objectID, err := primitive.ObjectIDFromHex(id)
	if err != nil {
		return nil, err
	}

	updateData["updated_at"] = time.Now()

	_, err = s.collection.UpdateOne(
		context.Background(),
		bson.M{"_id": objectID},
		bson.M{"$set": updateData},
	)
	if err != nil {
		return nil, err
	}

	return s.GetByID(id)
}

func (s *UserService) Delete(id string) error {
	objectID, err := primitive.ObjectIDFromHex(id)
	if err != nil {
		return err
	}

	_, err = s.collection.DeleteOne(context.Background(), bson.M{"_id": objectID})
	return err
}
"#.to_string(),
            DatabaseType::PostgreSQL => r#"package services

import (
	"time"

	"{{module_name}}/internal/database"
	"{{module_name}}/internal/models"
	"github.com/google/uuid"
)

type UserService struct {
	db *database.Database
}

func NewUserService(db *database.Database) *UserService {
	return &UserService{db: db}
}

func (s *UserService) Create(user *models.User) (*models.User, error) {
	user.ID = uuid.New()
	user.CreatedAt = time.Now()
	user.UpdatedAt = time.Now()

	query := `
		INSERT INTO users (id, email, hashed_password, full_name, is_active, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, $6, $7)
		RETURNING id, email, hashed_password, full_name, is_active, created_at, updated_at
	`
	
	err := s.db.DB.Get(user, query,
		user.ID, user.Email, user.HashedPassword, user.FullName,
		user.IsActive, user.CreatedAt, user.UpdatedAt)
	
	if err != nil {
		return nil, err
	}

	return user, nil
}

func (s *UserService) GetByID(id string) (*models.User, error) {
	userID, err := uuid.Parse(id)
	if err != nil {
		return nil, err
	}

	var user models.User
	query := "SELECT * FROM users WHERE id = $1"
	err = s.db.DB.Get(&user, query, userID)
	if err != nil {
		return nil, err
	}

	return &user, nil
}

func (s *UserService) GetByEmail(email string) (*models.User, error) {
	var user models.User
	query := "SELECT * FROM users WHERE email = $1"
	err := s.db.DB.Get(&user, query, email)
	if err != nil {
		return nil, err
	}

	return &user, nil
}

func (s *UserService) Update(id string, user *models.User) (*models.User, error) {
	userID, err := uuid.Parse(id)
	if err != nil {
		return nil, err
	}

	user.UpdatedAt = time.Now()

	query := `
		UPDATE users 
		SET email = $2, hashed_password = $3, full_name = $4, is_active = $5, updated_at = $6
		WHERE id = $1
		RETURNING id, email, hashed_password, full_name, is_active, created_at, updated_at
	`
	
	err = s.db.DB.Get(user, query, userID, user.Email, user.HashedPassword, 
		user.FullName, user.IsActive, user.UpdatedAt)
	
	if err != nil {
		return nil, err
	}

	return user, nil
}

func (s *UserService) Delete(id string) error {
	userID, err := uuid.Parse(id)
	if err != nil {
		return err
	}

	query := "DELETE FROM users WHERE id = $1"
	_, err = s.db.DB.Exec(query, userID)
	return err
}
"#.to_string()
        };

        let user_service_content = replace_template_vars_string(&user_service_go, &vars);
        write_file(base_path.join("internal/services/user.go"), &user_service_content)?;

        Ok(())
    }

    fn generate_routes_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        let routes_go = r#"package routes

import (
	"{{module_name}}/internal/config"
	"{{module_name}}/internal/handlers"
	"{{module_name}}/internal/middleware"
	"github.com/gin-gonic/gin"
)

func Setup(router *gin.Engine, h *handlers.Handlers) {
	cfg := config.Load()

	// Health check
	router.GET("/health", h.Health.Health)

	// Root endpoint
	router.GET("/", func(c *gin.Context) {
		c.JSON(200, gin.H{
			"message": "{{project_name}} API is running",
			"version": "1.0.0",
		})
	})

	// API v1 routes
	api := router.Group("/api/v1")
	{
		// Auth routes (no middleware)
		auth := api.Group("/auth")
		{
			auth.POST("/register", h.Auth.Register)
			auth.POST("/login", h.Auth.Login)
			auth.POST("/refresh", h.Auth.RefreshToken)
		}

		// Protected routes
		protected := api.Group("/")
		protected.Use(middleware.AuthMiddleware(cfg))
		{
			// User routes
			users := protected.Group("/users")
			{
				users.GET("/me", h.User.GetMe)
			}
		}
	}
}
"#;

        let routes_content = replace_template_vars_string(routes_go, &vars);
        write_file(base_path.join("internal/routes/routes.go"), &routes_content)?;

        Ok(())
    }

    fn generate_docker_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        if !config.include_docker {
            return Ok(());
        }

        let vars = self.get_template_vars(config);

        // Dockerfile
        let dockerfile_content = replace_template_vars_string(DOCKERFILE_GO, &vars);
        write_file(base_path.join("Dockerfile"), &dockerfile_content)?;

        // Docker Compose
        let docker_compose = r#"services:
  {{kebab_case}}-api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - ENVIRONMENT=production{{#if mongodb}}
      - MONGO_URL=mongodb://mongo:27017
      - DATABASE_NAME={{snake_case}}_db{{/if}}{{#if postgresql}}
      - DATABASE_URL=postgres://postgres:${POSTGRES_PASSWORD}@postgres:5432/{{snake_case}}_db?sslmode=disable{{/if}}
      - REDIS_URL=redis://redis:6379
      - JWT_SECRET=${JWT_SECRET}
    depends_on:{{#if mongodb}}
      - mongo{{/if}}{{#if postgresql}}
      - postgres{{/if}}
      - redis
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
{{/if}}{{#if postgresql}}
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

volumes:{{#if mongodb}}
  mongo_data:{{/if}}{{#if postgresql}}
  postgres_data:{{/if}}
  redis_data:

networks:
  {{kebab_case}}-network:
    driver: bridge
"#;

        let mut compose_content = docker_compose.to_string();
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
                return Err(crate::athena::AthenaError::validation_error_simple(
                    "MySQL is not supported for Go projects. Use Flask for MySQL support.".to_string()
                ));
            }
        }
        compose_content = replace_template_vars_string(&compose_content, &vars);
        write_file(base_path.join("docker-compose.yml"), &compose_content)?;

        // .env template
        let env_template = format!(r#"# Environment Configuration
ENVIRONMENT=development
PORT=8080

# Security
JWT_SECRET=your-jwt-secret-here-change-in-production
ACCESS_TOKEN_EXPIRE_MINUTES=30
REFRESH_TOKEN_EXPIRE_DAYS=7

# Database{}{}

# Redis
REDIS_URL=redis://localhost:6379

# CORS
ALLOWED_ORIGINS=http://localhost:3000,http://127.0.0.1:3000

# Rate Limiting
RATE_LIMIT_RPS=10
"#,
            if matches!(config.database, DatabaseType::MongoDB) {
                "\nMONGO_URL=mongodb://localhost:27017\nDATABASE_NAME=".to_string() + &ProjectNames::new(&config.name).snake_case + "_db"
            } else { "".to_string() },
            if matches!(config.database, DatabaseType::PostgreSQL) {
                "\nDATABASE_URL=postgres://user:password@localhost/".to_string() + &ProjectNames::new(&config.name).snake_case + "_db?sslmode=disable\nPOSTGRES_PASSWORD=your-postgres-password"
            } else { "".to_string() }
        );
        
        write_file(base_path.join(".env.example"), &env_template)?;

        Ok(())
    }

    fn generate_test_files(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let vars = self.get_template_vars(config);

        let test_main_go = r#"package main

import (
	"os"
	"testing"
)

func TestMain(m *testing.M) {
	// Setup test environment
	os.Setenv("ENVIRONMENT", "test")
	os.Setenv("JWT_SECRET", "test-secret-key")
	
	// Run tests
	code := m.Run()
	
	// Cleanup
	os.Exit(code)
}
"#;

        let test_auth_go = r#"package main

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"{{module_name}}/internal/config"
	"{{module_name}}/internal/database"
	"{{module_name}}/internal/handlers"
	"{{module_name}}/internal/routes"
	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)

func setupRouter() *gin.Engine {
	gin.SetMode(gin.TestMode)
	
	cfg := config.Load()
	db, _ := database.Connect(cfg)
	h := handlers.New(db, cfg)
	
	router := gin.New()
	routes.Setup(router, h)
	
	return router
}

func TestRegisterUser(t *testing.T) {
	router := setupRouter()

	user := map[string]string{
		"email":     "test@example.com",
		"password":  "testpassword123",
		"full_name": "Test User",
	}

	jsonData, _ := json.Marshal(user)
	req, _ := http.NewRequest("POST", "/api/v1/auth/register", bytes.NewBuffer(jsonData))
	req.Header.Set("Content-Type", "application/json")

	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var response map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Contains(t, response, "access_token")
	assert.Contains(t, response, "refresh_token")
	assert.Equal(t, "bearer", response["token_type"])
}

func TestLoginUser(t *testing.T) {
	router := setupRouter()

	// First register a user
	registerUser := map[string]string{
		"email":     "login@example.com",
		"password":  "testpassword123",
		"full_name": "Login User",
	}

	jsonData, _ := json.Marshal(registerUser)
	req, _ := http.NewRequest("POST", "/api/v1/auth/register", bytes.NewBuffer(jsonData))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)

	// Then login
	loginUser := map[string]string{
		"email":    "login@example.com",
		"password": "testpassword123",
	}

	jsonData, _ = json.Marshal(loginUser)
	req, _ = http.NewRequest("POST", "/api/v1/auth/login", bytes.NewBuffer(jsonData))
	req.Header.Set("Content-Type", "application/json")

	w = httptest.NewRecorder()
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var response map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Contains(t, response, "access_token")
	assert.Contains(t, response, "refresh_token")
}

func TestHealth(t *testing.T) {
	router := setupRouter()

	req, _ := http.NewRequest("GET", "/health", nil)
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	var response map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Equal(t, "healthy", response["status"])
}
"#;

        write_file(base_path.join("tests/main_test.go"), test_main_go)?;
        
        let test_auth_content = replace_template_vars_string(test_auth_go, &vars);
        write_file(base_path.join("tests/auth_test.go"), &test_auth_content)?;

        Ok(())
    }

    fn generate_documentation(&self, config: &ProjectConfig, base_path: &Path) -> BoilerplateResult<()> {
        let _vars = self.get_template_vars(config);
        let names = ProjectNames::new(&config.name);

        let readme = format!(r#"# {project_name}

Production-ready Go API with Gin framework, JWT authentication, and comprehensive security features.

## Features

- **Gin Framework** - High-performance HTTP web framework
- **JWT Authentication** - Access & refresh token system
- **Password Security** - bcrypt hashing
- **{database}** - Database integration with migrations
- **Middleware** - CORS, security headers, rate limiting
- **Docker** - Containerized deployment
- **Tests** - Comprehensive test suite
- **Graceful Shutdown** - Proper application lifecycle management

## Quick Start

### Development

```bash
# Install dependencies
go mod download

# Set up environment
cp .env.example .env
# Edit .env with your configuration

# Run the application
go run cmd/main.go
```

### With Docker

```bash
# Build and run with Docker Compose
docker-compose up --build

# The API will be available at http://localhost:8080
```

## API Documentation

### Authentication
- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/login` - User login  
- `POST /api/v1/auth/refresh` - Refresh access token

### Users (Protected)
- `GET /api/v1/users/me` - Get current user info

### System
- `GET /health` - Health check
- `GET /` - API info

## Testing

```bash
# Run all tests
go test ./...

# Run tests with verbose output
go test -v ./...

# Run specific test
go test -v ./tests -run TestRegisterUser

# Run tests with coverage
go test -cover ./...
```

## Project Structure

```
{snake_case}/
├── cmd/                    # Application entrypoints
├── internal/               # Private application code
│   ├── config/            # Configuration
│   ├── database/          # Database connection
│   ├── handlers/          # HTTP handlers
│   ├── middleware/        # HTTP middleware
│   ├── models/            # Data models
│   ├── routes/            # Route definitions
│   └── services/          # Business logic
├── pkg/                   # Public library code
│   ├── auth/              # Authentication utilities
│   ├── errors/            # Error handling
│   └── logger/            # Logging utilities
├── tests/                 # Test files
├── deployments/           # Deployment configurations
├── go.mod                 # Go modules
└── Dockerfile            # Docker configuration
```

## Configuration

Key environment variables:

```env
ENVIRONMENT=development
PORT=8080
JWT_SECRET=your-jwt-secret-here
ACCESS_TOKEN_EXPIRE_MINUTES=30
REFRESH_TOKEN_EXPIRE_DAYS=7
{database_config}
REDIS_URL=redis://localhost:6379
ALLOWED_ORIGINS=http://localhost:3000
```

## Security Features

- JWT-based authentication with refresh tokens
- Password hashing with bcrypt
- Security headers middleware
- CORS configuration
- Rate limiting middleware
- Input validation
- SQL injection prevention
- Graceful error handling

## Database

This project uses {database} for data persistence.

{database_instructions}

## Deployment

1. Set `ENVIRONMENT=production` 
2. Use strong `JWT_SECRET`
3. Configure your database connection
4. Set up load balancing if needed
5. Configure monitoring and logging

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
                DatabaseType::MySQL => format!("DATABASE_URL=mysql://user:password@localhost/{}_db", names.snake_case),
                DatabaseType::MongoDB => format!("MONGO_URL=mongodb://localhost:27017\nDATABASE_NAME={}_db", names.snake_case),
                DatabaseType::PostgreSQL => format!("DATABASE_URL=postgres://user:password@localhost/{}_db?sslmode=disable", names.snake_case),
            },
            database_instructions = match config.database {
                DatabaseType::MySQL => r#"
### MySQL Setup

For MySQL support in Go projects, please use Flask with MySQL option instead: `athena init flask project --with-mysql`
"#,
                DatabaseType::MongoDB => r#"
### MongoDB Setup

The application automatically connects to MongoDB using the configured URL. Collections are created automatically when first accessed."#,
                DatabaseType::PostgreSQL => r#"
### PostgreSQL Setup

The application includes automatic database migrations that create the necessary tables on startup. Ensure your PostgreSQL instance is running and accessible."#,
            }
        );
        
        write_file(base_path.join("README.md"), &readme)?;

        Ok(())
    }
}

impl BoilerplateGenerator for GoGenerator {
    fn validate_config(&self, config: &ProjectConfig) -> BoilerplateResult<()> {
        crate::boilerplate::validate_project_name(&config.name)?;
        crate::boilerplate::check_directory_availability(Path::new(&config.directory))?;
        Ok(())
    }

    fn generate_project(&self, config: &ProjectConfig) -> BoilerplateResult<()> {
        let base_path = Path::new(&config.directory);

        println!("Generating Go project: {}", config.name);
        
        // Create directory structure
        println!("  Creating directory structure...");
        self.create_go_structure(base_path)?;

        // Generate main files
        println!("  Generating main application files...");
        self.generate_main_files(config, base_path)?;

        // Generate config files
        println!("  🔧 Setting up configuration...");
        self.generate_config_files(config, base_path)?;

        // Generate database files
        println!("  💾 Setting up database integration...");
        self.generate_database_files(config, base_path)?;

        // Generate auth files
        println!("  🔐 Creating authentication system...");
        self.generate_auth_files(config, base_path)?;

        // Generate models
        println!("  📊 Creating data models...");
        self.generate_models_files(config, base_path)?;

        // Generate middleware
        println!("  Setting up middleware...");
        self.generate_middleware_files(config, base_path)?;

        // Generate handlers
        println!("  Creating HTTP handlers...");
        self.generate_handlers_files(config, base_path)?;

        // Generate services
        println!("  Setting up business logic services...");
        self.generate_services_files(config, base_path)?;

        // Generate routes
        println!("  Configuring routes...");
        self.generate_routes_files(config, base_path)?;

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

        println!("Go project '{}' created successfully!", config.name);
        println!("📍 Location: {}", base_path.display());
        
        if config.include_docker {
            println!("\n🔧 Next steps:");
            println!("  cd {}", config.directory);
            println!("  cp .env.example .env  # Edit with your configuration");  
            println!("  docker-compose up --build");
        } else {
            println!("\n🔧 Next steps:");
            println!("  cd {}", config.directory);
            println!("  go mod download");
            println!("  cp .env.example .env  # Edit with your configuration");
            println!("  go run cmd/main.go");
        }

        Ok(())
    }
}