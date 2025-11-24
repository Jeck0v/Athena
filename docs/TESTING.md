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
├── integration_tests.rs                # Main integration test entry point
├── integration/
│   ├── mod.rs                          # Module declarations and utilities
│   ├── cli_commands_test.rs            # Test all CLI commands and options
│   ├── docker_compose_generation_test.rs # Full Docker Compose generation tests
│   ├── error_handling_test.rs          # Error case testing and validation
│   ├── enhanced_error_handling_test.rs # Advanced error scenarios with suggestions
│   ├── build_args_cli_tests.rs         # Dockerfile integration and BUILD-ARGS tests
│   ├── swarm_features_test.rs          # Docker Swarm support and error handling
│   └── structural/                     # Organized structural tests (lightweight)
│       ├── mod.rs                      # Common utilities and module declarations
│       ├── basic_structure.rs          # Basic YAML structure validation
│       ├── service_configuration.rs    # Service config (env vars, ports, volumes)
│       ├── networking.rs               # Networks and service dependencies
│       ├── policies.rs                 # Restart policies and health checks
│       ├── formatting.rs               # YAML validity and formatting tests
│       ├── comments.rs                 # Comment parsing and edge cases
│       └── complex_scenarios.rs        # Complex microservices scenarios
├── fixtures/
│   ├── valid_simple.ath                # Simple valid .ath file (3 services)
│   ├── valid_complex_microservices.ath # Complex microservices setup
│   ├── minimal_valid.ath               # Minimal valid configuration
│   ├── invalid_syntax.ath              # File with syntax errors
│   ├── circular_dependencies.ath       # Circular dependency test cases
│   ├── port_conflicts.ath              # Port conflict scenarios
│   ├── comments_test.ath               # Comment parsing test cases
│   ├── build_args_basic.ath            # Basic BUILD-ARGS examples
│   ├── build_args_complex.ath          # Complex BUILD-ARGS scenarios
│   ├── build_args_invalid.ath          # Invalid BUILD-ARGS for error testing
│   ├── build_args_mixed_valid_invalid.ath # Mixed valid/invalid BUILD-ARGS
│   ├── build_args_multiple_services.ath # Multiple services with BUILD-ARGS
│   ├── build_args_with_image.ath       # BUILD-ARGS with IMAGE-ID precedence
│   ├── swarm_basic.ath                 # Basic Docker Swarm features
│   ├── swarm_advanced.ath              # Advanced Swarm scenarios
│   ├── swarm_errors.ath                # Swarm error testing base
│   └── mixed_features.ath              # Mixed Compose + Swarm features
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

# Enhanced error handling tests
cargo test --test integration_tests enhanced_error_handling_test

# Build args CLI tests
cargo test --test integration_tests build_args_cli_tests

# Docker Swarm feature tests
cargo test --test integration_tests swarm_features_test

# All structural tests (lightweight YAML validation)
cargo test --test integration_tests structural

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
- Tests port conflict detection during generation
- Tests successful generation with different ports
- Validates YAML syntax and structure

### 3. Error Handling Tests (`error_handling_test.rs`)
- Tests file not found scenarios
- Tests invalid syntax handling
- Tests circular dependency detection
- Tests port conflict detection with detailed error messages
- Tests port conflict suggestions for resolution
- Tests mixed port mapping scenarios
- Tests malformed configuration errors
- Tests permission and access errors
- Validates error message quality

### 4. Enhanced Error Handling Tests (`enhanced_error_handling_test.rs`)
- Advanced error scenarios with intelligent suggestions
- Tests enhanced port conflict detection with multiple services
- Validates service reference error messages with suggestions
- Tests validation error improvements
- Comprehensive error message quality assurance

### 5. Build Args CLI Tests (`build_args_cli_tests.rs`)
- Tests Dockerfile integration and validation
- BUILD-ARGS parsing and generation
- Dockerfile ARG validation against Athena BUILD-ARGS
- Error handling for missing Dockerfiles
- Intelligent similarity suggestions for mismatched ARG names
- Tests precedence of IMAGE-ID over BUILD-ARGS

### 6. Docker Swarm Feature Tests (`swarm_features_test.rs`)
- Comprehensive Docker Swarm support testing
- REPLICAS directive validation and error handling
- UPDATE-CONFIG options testing (PARALLELISM, DELAY, FAILURE-ACTION)
- SWARM-LABELS parsing with flexible syntax support
- Overlay network configuration testing
- Complete integration tests with Swarm + Compose features
- 13 dedicated error handling tests for edge cases

### 7. Structural Tests (`structural/`)
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
- `comments.rs`: Comment parsing, multi-line comments, and edge cases
- `complex_scenarios.rs`: Complex microservices architecture tests

## Test Fixtures

### Valid Test Files
- **`valid_simple.ath`**: Basic 3-service setup (web, app, database)
- **`valid_complex_microservices.ath`**: Complex microservices architecture
- **`minimal_valid.ath`**: Minimal valid configuration

### Invalid Test Files
- **`invalid_syntax.ath`**: Contains various syntax errors
- **`circular_dependencies.ath`**: Services with circular dependencies
- **`port_conflicts.ath`**: Port conflict scenarios for error testing

### BUILD-ARGS Test Files
- **`build_args_basic.ath`**: Basic BUILD-ARGS examples
- **`build_args_complex.ath`**: Complex BUILD-ARGS scenarios with multiple services
- **`build_args_invalid.ath`**: Invalid BUILD-ARGS for error testing
- **`build_args_mixed_valid_invalid.ath`**: Mixed valid/invalid BUILD-ARGS scenarios
- **`build_args_multiple_services.ath`**: Multiple services with BUILD-ARGS
- **`build_args_with_image.ath`**: BUILD-ARGS with IMAGE-ID precedence testing

### Comment Test Files
- **`comments_test.ath`**: Comment parsing test cases including multi-line and edge cases

### Docker Swarm Test Files
- **`swarm_basic.ath`**: Basic Docker Swarm features (REPLICAS, UPDATE-CONFIG, SWARM-LABELS)
- **`swarm_advanced.ath`**: Advanced Swarm scenarios with all options and complex architectures
- **`swarm_errors.ath`**: Base fixture for Swarm error testing scenarios
- **`mixed_features.ath`**: Mixed Docker Compose and Swarm features in same deployment

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

### Test Performance & Statistics

**Current test suite:**
- **Total tests**: 103 integration tests
- **CLI tests**: 13 tests (command parsing, help, validation)
- **Docker Compose generation**: 11 tests (YAML generation, validation, port conflict detection)
- **Error handling**: 21 tests (comprehensive error scenarios including port conflicts)
- **Enhanced error handling**: 6 tests (advanced error scenarios with suggestions)
- **Build args CLI**: 8 tests (Dockerfile integration and validation)
- **Swarm features**: 21 tests (Docker Swarm support with comprehensive error handling)
- **Structural tests**: 23 tests (organized in 6 categories including comments)
- **Execution time**: < 1 second for structural tests
- **Test organization**: Modular structure for easy maintenance

**Test breakdown by category:**

**Structural tests:**
- `basic_structure.rs`: 2 tests (YAML structure, service count validation)
- `service_configuration.rs`: 4 tests (env vars, ports, volumes, service settings)
- `networking.rs`: 2 tests (network configuration, service dependencies)
- `policies.rs`: 2 tests (restart policies, health check configurations)
- `formatting.rs`: 2 tests (YAML validity, readable output formatting)
- `comments.rs`: 11 tests (comment parsing, edge cases, multi-line comments)
- `complex_scenarios.rs`: 1 test (complex microservices architecture)

## Port Conflict Detection Tests

### Overview
As of the latest update, Athena includes comprehensive port conflict detection that prevents Docker Compose generation when multiple services attempt to use the same host port.

### Test Coverage

#### In `docker_compose_generation_test.rs`:
- **`test_port_conflict_prevention_in_generation`**: Verifies that Docker Compose generation fails when multiple services use the same host port (e.g., two services both mapping to port 8080)
- **`test_successful_generation_with_different_ports`**: Confirms that generation succeeds when services use different host ports (e.g., one service uses 8080, another uses 8081)

#### In `error_handling_test.rs`:
- **`test_port_conflict_detection`**: Tests basic port conflict detection using the fixture file
- **`test_port_conflict_suggestions`**: Verifies that the error message includes helpful port suggestions (e.g., "Consider using different ports like: 3000, 3001, 3002")
- **`test_no_port_conflicts_different_ports`**: Ensures no false positives when services use different ports
- **`test_port_conflict_with_mixed_mappings`**: Tests scenarios where services have multiple port mappings with conflicts

### Key Features Tested
1. **Conflict Detection**: Identifies when multiple services use the same host port
2. **Detailed Error Messages**: Provides clear information about which services are conflicting
3. **Port Suggestions**: Automatically generates alternative port suggestions
4. **Mixed Mappings**: Handles services with multiple port mappings correctly
5. **Integration with Generation**: Prevents invalid Docker Compose file creation

### Example Test Scenarios
```rust
// Conflict scenario - should fail
SERVICE service1
PORT-MAPPING 8080 TO 80
END SERVICE

SERVICE service2
PORT-MAPPING 8080 TO 8000  // Conflict on host port 8080
END SERVICE

// Valid scenario - should succeed
SERVICE service1
PORT-MAPPING 8080 TO 80
END SERVICE

SERVICE service2
PORT-MAPPING 8081 TO 8000  // Different host port
END SERVICE
```

## Docker Swarm Feature Tests (**NEW**)

### Overview
As of December 2025, Athena includes comprehensive Docker Swarm support with extensive error handling tests covering all edge cases and invalid configurations.

### Test Coverage (`swarm_features_test.rs`)

#### Success Scenarios (8 tests):
- **`test_swarm_replicas_parsing`**: Validates REPLICAS directive parsing
- **`test_swarm_update_config_parsing`**: Tests UPDATE-CONFIG with all options
- **`test_swarm_labels_parsing`**: Verifies SWARM-LABELS functionality
- **`test_overlay_network_parsing`**: Tests overlay network configuration
- **`test_complete_swarm_compose_generation`**: Full integration test
- **`test_mixed_compose_and_swarm_features`**: Mixed mode compatibility
- **`test_swarm_labels_without_quotes_should_work`**: Flexible label parsing
- **`test_conflicting_swarm_and_compose_features`**: Feature coexistence

#### Error Handling Tests (13 tests):
**Replica Validation:**
- **`test_invalid_replica_negative_number`**: Catches `REPLICAS -5`
- **`test_invalid_replica_extremely_large_number`**: Catches overflow numbers
- **`test_invalid_replica_non_numeric`**: Catches `REPLICAS abc`
- **`test_invalid_replica_zero`**: Allows zero replicas (edge case)

**UPDATE-CONFIG Validation:**
- **`test_invalid_update_config_negative_parallelism`**: Catches negative values
- **`test_invalid_update_config_invalid_delay_format`**: Validates time formats
- **`test_invalid_failure_action`**: Catches invalid failure actions
- **`test_invalid_max_failure_ratio`**: Validates ratio bounds

**SWARM-LABELS Validation:**
- **`test_invalid_swarm_labels_malformed_missing_value`**: Catches missing values
- **`test_empty_swarm_labels`**: Catches empty label directives

**Network and Service Validation:**
- **`test_invalid_network_driver`**: Catches invalid network drivers
- **`test_invalid_boolean_values`**: Validates boolean parameters
- **`test_swarm_config_without_service_name`**: Catches missing service names

### Key Features Tested
1. **Comprehensive Error Detection**: All invalid configurations are caught
2. **Detailed Error Messages**: Specific error messages with line/column info
3. **Edge Case Handling**: Zero replicas, large numbers, malformed syntax
4. **Flexible Parsing**: Supports both quoted and unquoted label values
5. **Integration Testing**: Full Docker Compose generation with Swarm features

### Example Test Scenarios
```rust
// Replica validation tests
fn test_invalid_replica_negative_number() {
    let input = r#"
        SERVICE web
        IMAGE-ID nginx:alpine
        REPLICAS -5    // Should fail
        END SERVICE
    "#;
    assert!(parse_athena_file(input).is_err());
}

// Label validation tests
fn test_invalid_swarm_labels_malformed() {
    let input = r#"
        SERVICE web
        SWARM-LABELS environment="prod" tier=  // Missing value
        END SERVICE
    "#;
    assert!(parse_athena_file(input).is_err());
}
```

### Coverage Goals
The test suite aims for >80% coverage on critical code paths:
- CLI argument parsing
- .ath file parsing and validation
- Docker Compose generation
- Docker Swarm configuration generation
- Port conflict detection and validation
- Swarm-specific directive validation
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
