# Development Guide

## Development Commands
```bash
# Development cycle with tests
make dev

# Install and run demo
make demo

# Check installation
make check-install

# Clean build artifacts
make clean
```

## Running Tests
```bash
# All tests
cargo test

# Integration tests only
cargo test --test integration_tests

# Structural tests (fastest, most common)
cargo test --test integration_tests structural

# Specific test categories
cargo test --test integration_tests cli_commands_test
cargo test --test integration_tests docker_compose_generation_test
cargo test --test integration_tests error_handling_test

# Run with verbose output
cargo test --test integration_tests structural --verbose
```

## Contributing
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make changes with appropriate tests
4. Run the test suite: `cargo test --test integration_tests`
5. Ensure code quality: `cargo clippy` and `cargo fmt`
6. All tests must pass in CI (GitHub Actions workflow)
7. Commit changes (`git commit -m 'Add amazing feature'`)
8. Push to branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

## Test Requirements
- Add structural tests for Docker Compose generation changes
- Add CLI tests for new command options
- All integration tests must pass on Ubuntu latest via GitHub Actions

## Code Quality
- Run `cargo clippy` for linting
- Run `cargo fmt` for formatting
- Ensure all tests pass with `cargo test`
- Follow Rust best practices and idioms