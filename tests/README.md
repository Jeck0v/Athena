# Athena Integration Tests

This directory contains comprehensive integration tests for the Athena CLI tool.

## 🎯 **Test Philosophy**

Our tests focus on **functionality over format**:
- ✅ **Structural tests** verify logic and behavior
- ✅ **Functional tests** check that features work correctly  
- ✅ **Lightweight approach** easy to maintain and fast to run
- ❌ **No heavy snapshot tests** that break on cosmetic changes

## Test Structure

```
tests/
├── integration/
│   ├── cli_commands_test.rs             # Test all CLI commands
│   ├── docker_compose_generation_test.rs # Full generation test
│   ├── error_handling_test.rs           # Error case testing
│   ├── boilerplate_generation_test.rs   # Init command tests
│   └── structural_tests.rs              # Lightweight YAML structure tests
├── fixtures/
│   ├── valid_simple.ath                # Simple valid .ath file
│   ├── valid_complex_microservices.ath # Complex microservices setup
│   ├── invalid_syntax.ath              # File with syntax errors
│   ├── circular_dependencies.ath       # Circular dependency test
│   └── minimal_valid.ath               # Minimal valid configuration
```

## Running Tests

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
cargo test --test integration_tests boilerplate_generation_test

# Structural tests (lightweight YAML validation)
cargo test --test integration_tests structural_tests
```

### Run Individual Tests
```bash
cargo test --test integration_tests cli_commands_test::test_cli_help
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

### 4. Boilerplate Generation Tests (`boilerplate_generation_test.rs`)
- Tests `athena init` commands
- Tests FastAPI, Flask, and Go project generation
- Tests database configuration options
- Tests Docker file generation
- Tests custom directory options

### 5. Structural Tests (`structural_tests.rs`)
- **Lightweight YAML validation** without heavy snapshots
- Tests **structure and logic** rather than exact formatting
- **Fast and maintainable** - no snapshot file management
- Validates **Docker Compose compliance** and **key functionality**

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

## 💡 **Why Structural Tests?**

### ✅ **Advantages of Our Approach**
- **🚀 Fast execution** - No heavy file comparisons
- **🔧 Easy maintenance** - No snapshot file management
- **📋 Clear intent** - Tests specific functionality, not formatting  
- **💪 Robust** - Don't break on cosmetic changes
- **🔍 Focused** - Test what matters: structure and logic

### ❌ **Why We Avoid Snapshot Tests**
- **🐌 Slow and heavy** - Large files to compare
- **💔 Fragile** - Break on whitespace or comment changes
- **🔄 High maintenance** - Constant `cargo insta review` cycles
- **😵 Opaque failures** - Hard to see what actually matters
- **📁 File bloat** - Many large snapshot files to maintain

## Usage Notes

### Running Structural Tests
Our structural tests are designed to be **fast and reliable**:
```bash
# Run all structural tests - should complete in < 5 seconds
cargo test --test integration_tests structural_tests

# Run specific structural test
cargo test --test integration_tests structural_tests::test_basic_yaml_structure
```

### What Structural Tests Check

**Example: Instead of comparing entire YAML files, we test specific logic:**

```rust
// ✅ Good: Test what matters
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

// ❌ Avoid: Brittle snapshot comparison
// assert_snapshot!("entire_compose_file", yaml_content);
```

**What we verify:**
- 🔍 **YAML structure validity** (version, services, networks)
- 🔍 **Service configuration** (images, ports, environment)  
- 🔍 **Relationships** (dependencies, networks)
- 🔍 **Logic correctness** (restart policies, health checks)
- 🔍 **Docker Compose compliance** (valid format)

### Boilerplate Tests
Some boilerplate generation tests may fail if the actual implementation is not complete. These tests verify:
- Project directory creation
- File structure generation
- Configuration file content
- Database-specific setup

### Coverage Goals
The test suite aims for >80% coverage on critical code paths:
- CLI argument parsing
- .ath file parsing and validation
- Docker Compose generation
- Error handling and reporting
- Project initialization

### CI/CD Integration
To run tests in CI/CD pipelines:
```bash
# Run all tests with verbose output
cargo test --verbose

# Run tests with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out xml

# Run tests in release mode for performance
cargo test --release
```

## Contributing

When adding new tests:
1. Follow the existing naming conventions
2. Add appropriate fixtures for new test cases
3. Update snapshots when output format changes
4. Test both success and failure scenarios
5. Add comprehensive error case testing