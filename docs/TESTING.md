# Athena Integration Tests

This directory contains comprehensive integration tests for the Athena CLI tool.

## Test Philosophy

Our tests focus on **functionality over format**:
- **Structural tests** verify logic and behavior
- **Functional tests** check that features work correctly
- **Lightweight approach** easy to maintain and fast to run
- **No heavy snapshot tests** that break on cosmetic changes

## Test Structure

```
tests/
├── integration/
│   ├── cli_commands_test.rs             # Test all CLI commands
│   ├── docker_compose_generation_test.rs # Full generation test
│   ├── error_handling_test.rs           # Error case testing
│   ├── boilerplate/                     # Modular boilerplate tests
│   │   ├── mod.rs                       # Common utilities and shared functions
│   │   ├── fastapi_tests.rs             # FastAPI project generation tests
│   │   ├── flask_tests.rs               # Flask project generation tests
│   │   ├── go_tests.rs                  # Go project generation tests
│   │   └── common_tests.rs              # Common init command tests
│   └── structural/                      # Organized structural tests
│       ├── mod.rs                       # Common utilities and module declarations
│       ├── basic_structure.rs           # Basic YAML structure validation
│       ├── service_configuration.rs     # Service config (env vars, ports, volumes)
│       ├── networking.rs                # Networks and service dependencies
│       ├── policies.rs                  # Restart policies and health checks
│       ├── formatting.rs                # YAML validity and formatting tests
│       └── complex_scenarios.rs         # Complex microservices scenarios
├── fixtures/
│   ├── valid_simple.ath                # Simple valid .ath file
│   ├── valid_complex_microservices.ath # Complex microservices setup
│   ├── invalid_syntax.ath              # File with syntax errors
│   ├── circular_dependencies.ath       # Circular dependency test
│   └── minimal_valid.ath               # Minimal valid configuration
```

## Running Tests

### Quick Start
```bash
# Run all tests
cargo test

# Run only integration tests
cargo test --test integration_tests

# Run structural tests (fastest, most common)
cargo test --test integration_tests structural

# Run with verbose output to see individual test names
cargo test --test integration_tests structural --verbose
```

### Run All Tests
```bash
cargo test
```

### Run Integration Tests Only
```bash
cargo test --test integration_tests
```

### Run Specific Test Categories
```bash
# CLI command tests
cargo test --test integration_tests cli_commands_test

# Docker Compose generation tests
cargo test --test integration_tests docker_compose_generation_test

# Error handling tests
cargo test --test integration_tests error_handling_test

# Boilerplate generation tests
cargo test --test integration_tests boilerplate

# All structural tests (lightweight YAML validation)
cargo test --test integration_tests structural

# Specific boilerplate test categories
cargo test --test integration_tests boilerplate::fastapi_tests
cargo test --test integration_tests boilerplate::flask_tests
cargo test --test integration_tests boilerplate::go_tests
cargo test --test integration_tests boilerplate::common_tests

# Specific structural test categories
cargo test --test integration_tests structural::basic_structure
cargo test --test integration_tests structural::service_configuration
cargo test --test integration_tests structural::networking
cargo test --test integration_tests structural::policies
cargo test --test integration_tests structural::formatting
cargo test --test integration_tests structural::complex_scenarios
```

### Run Individual Tests
```bash
# Run a specific test function
cargo test --test integration_tests cli_commands_test::test_cli_help

# Run a specific structural test
cargo test --test integration_tests structural::basic_structure::test_basic_yaml_structure

# Run with verbose output to see test names
cargo test --test integration_tests structural --verbose
```

## Test Categories

### 1. CLI Commands Tests (`cli_commands_test.rs`)
- Tests all CLI commands and options
- Validates help text and command parsing
- Tests file input/output handling
- Covers verbose/quiet modes
- Tests auto-detection features

### 2. Docker Compose Generation Tests (`docker_compose_generation_test.rs`)
- Tests YAML generation from .ath files
- Validates Docker Compose structure
- Tests environment variable templating
- Tests port mappings, volume mounts
- Tests health checks and resource limits
- Validates YAML syntax and structure

### 3. Error Handling Tests (`error_handling_test.rs`)
- Tests file not found scenarios
- Tests invalid syntax handling
- Tests circular dependency detection
- Tests malformed configuration errors
- Tests permission and access errors
- Validates error message quality

### 4. Boilerplate Generation Tests (`boilerplate/`)
- **Modular organization** by framework for better maintainability
- **FastAPI tests** (`fastapi_tests.rs`): Basic init, PostgreSQL/MongoDB options, Docker/no-Docker modes
- **Flask tests** (`flask_tests.rs`): Basic init, MySQL integration
- **Go tests** (`go_tests.rs`): Gin, Echo, and Fiber framework support
- **Common tests** (`common_tests.rs`): Error handling, help commands, project validation
- Tests project structure generation, configuration files, and dependency setup

### 5. Structural Tests (`structural/`)
- **Organized by functional categories** for better maintainability
- **Lightweight YAML validation** without heavy snapshots
- Tests **structure and logic** rather than exact formatting
- **Fast and maintainable** - no snapshot file management
- Validates **Docker Compose compliance** and **key functionality**

**Test categories:**
- `basic_structure.rs`: Basic YAML structure and service count validation
- `service_configuration.rs`: Environment variables, ports, volumes, and service settings
- `networking.rs`: Network configuration and service dependencies
- `policies.rs`: Restart policies and health check configurations
- `formatting.rs`: YAML validity and readable output formatting
- `complex_scenarios.rs`: Complex microservices architecture tests

## Test Fixtures

### Valid Test Files
- **`valid_simple.ath`**: Basic 3-service setup (web, app, database)
- **`valid_complex_microservices.ath`**: Complex microservices architecture
- **`minimal_valid.ath`**: Minimal valid configuration

### Invalid Test Files
- **`invalid_syntax.ath`**: Contains various syntax errors
- **`circular_dependencies.ath`**: Services with circular dependencies

## Dependencies

The integration tests use several lightweight dependencies:
- **`assert_cmd`**: CLI testing framework
- **`predicates`**: Assertions for command output
- **`tempfile`**: Temporary file/directory management
- **`serial_test`**: Sequential test execution (for file system tests)
- **`pretty_assertions`**: Better assertion output
- **`serde_yaml`**: YAML parsing for structural validation

## Why Structural Tests?

### Advantages of Our Approach
- **Fast execution** - No heavy file comparisons
- **Easy maintenance** - No snapshot file management
- **Clear intent** - Tests specific functionality, not formatting
- **Robust** - Don't break on cosmetic changes
- **Focused** - Test what matters: structure and logic

### Why We Avoid Snapshot Tests
- **Slow and heavy** - Large files to compare
- **Fragile** - Break on whitespace or comment changes
- **High maintenance** - Constant `cargo insta review` cycles
- **Opaque failures** - Hard to see what actually matters
- **File bloat** - Many large snapshot files to maintain

## Usage Notes

### Running Structural Tests
Our structural tests are designed to be **fast and reliable**:
```bash
# Run all structural tests - should complete in < 1 second
cargo test --test integration_tests structural

# Run specific structural test category
cargo test --test integration_tests structural::basic_structure

# Run specific structural test function
cargo test --test integration_tests structural::basic_structure::test_basic_yaml_structure
```

### What Structural Tests Check

**Example: Instead of comparing entire YAML files, we test specific logic:**

```rust
// Good: Test what matters
#[test]
fn test_service_configuration_structure() {
    let parsed = run_athena_build_and_parse(&ath_file);
    let services = parsed["services"].as_mapping().unwrap();

    // Test specific functionality
    assert!(services.contains_key("web"), "Should have web service");
    assert_eq!(services["web"]["image"], "nginx:alpine");
    assert!(services["web"]["ports"].is_sequence());
    assert!(services["web"]["environment"].is_sequence());
}

// Avoid: Brittle snapshot comparison
// assert_snapshot!("entire_compose_file", yaml_content);
```

**What we verify:**
- **YAML structure validity** (services, networks, volumes)
- **Service configuration** (images, ports, environment variables)
- **Relationships** (dependencies, networks)
- **Logic correctness** (restart policies, health checks)
- **Docker Compose compliance** (valid modern format)

### Boilerplate Tests
Modular boilerplate tests organized by framework, each verifying:
- **Project directory creation** and proper file structure
- **Framework-specific configurations** (dependencies, settings)
- **Database integration** (PostgreSQL, MongoDB, MySQL setup)
- **Docker configuration** generation and optional exclusion
- **Custom directory** and project name handling
- **Error scenarios** and validation

### Test Performance & Statistics

**Current test suite:**
- **Total tests**: 67 integration tests
- **CLI tests**: 13 tests (command parsing, help, validation)
- **Docker Compose generation**: 9 tests (YAML generation and validation)
- **Error handling**: 18 tests (comprehensive error scenarios)
- **Boilerplate generation**: 14 tests (modular by framework)
- **Structural tests**: 13 tests (organized in 6 categories)
- **Execution time**: < 1 second for structural tests
- **Test organization**: Modular structure for easy maintenance

**Test breakdown by category:**

**Structural tests:**
- `basic_structure.rs`: 2 tests
- `service_configuration.rs`: 4 tests
- `networking.rs`: 2 tests
- `policies.rs`: 2 tests
- `formatting.rs`: 2 tests
- `complex_scenarios.rs`: 1 test

**Boilerplate tests:**
- `fastapi_tests.rs`: 6 tests (basic, PostgreSQL, MongoDB, no-Docker, custom directory, help)
- `flask_tests.rs`: 2 tests (basic, MySQL integration)
- `go_tests.rs`: 3 tests (Gin, Echo, Fiber frameworks)
- `common_tests.rs`: 3 tests (error handling, validation, help commands)

### Coverage Goals
The test suite aims for >80% coverage on critical code paths:
- CLI argument parsing
- .ath file parsing and validation
- Docker Compose generation
- Error handling and reporting
- Project initialization


## Contributing

When adding new tests:
1. **Follow naming conventions**: Use descriptive test function names starting with `test_`
2. **Add fixtures**: Create appropriate `.ath` test files in `tests/fixtures/` for new scenarios
3. **Choose the right category**: Place structural tests in the appropriate category file
4. **Test both scenarios**: Include success and failure cases
5. **Update documentation**: Update this README when adding new test categories
6. **Keep tests focused**: Each test should verify one specific aspect of functionality

### Adding Structural Tests
For new structural tests, place them in the appropriate category:
- `basic_structure.rs`: YAML structure and service count validation
- `service_configuration.rs`: Service settings (env vars, ports, volumes)
- `networking.rs`: Network configuration and dependencies
- `policies.rs`: Restart policies and health checks
- `formatting.rs`: YAML validity and output formatting
- `complex_scenarios.rs`: Multi-service architecture tests

### Adding Boilerplate Tests
For new boilerplate tests, place them in the appropriate framework file:
- `fastapi_tests.rs`: FastAPI-specific features and configurations
- `flask_tests.rs`: Flask-specific features and database integrations
- `go_tests.rs`: Go framework-specific tests (Gin, Echo, Fiber)
- `common_tests.rs`: Cross-framework functionality (validation, help, errors)
