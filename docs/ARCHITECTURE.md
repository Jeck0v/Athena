# Architecture

## Core Components

```
athena/
├── src/
│   ├── cli/                    # Command-line interface
│   │   ├── args.rs            # Argument parsing
│   │   ├── commands.rs        # Command implementations  
│   │   └── utils.rs           # CLI utilities
│   ├── athena/                # Core functionality
│   │   ├── parser/            # DSL parsing
│   │   │   ├── grammar.pest   # COBOL-inspired grammar
│   │   │   ├── ast.rs         # Abstract syntax tree
│   │   │   ├── parser.rs      # Parser implementation
│   │   │   └── optimized_parser.rs # Performance optimizations
│   │   ├── generator/         # Docker Compose generation
│   │   │   ├── compose.rs     # Main generator
│   │   │   └── defaults.rs    # Intelligent defaults engine
│   │   └── error.rs           # Typed error handling
│   └── main.rs                # Application entrypoint
├── docs/                      # Documentation
├── tests/                     # Comprehensive test suite
│   ├── integration/           # Integration tests organized by functionality
│   │   ├── cli_commands_test.rs      # CLI command tests
│   │   ├── docker_compose_generation_test.rs # YAML generation tests
│   │   ├── error_handling_test.rs    # Error scenario tests
│   │   └── structural/              # Lightweight structural tests
│   │       ├── basic_structure.rs   # YAML structure validation
│   │       ├── service_configuration.rs # Service config tests
│   │       ├── networking.rs        # Network and dependency tests
│   │       ├── policies.rs          # Restart and health check tests
│   │       ├── formatting.rs        # YAML validity tests
│   │       └── complex_scenarios.rs # Microservices scenarios
│   └── fixtures/              # Test .ath files and configurations
└── examples/                  # Example configurations
```

## Performance Features
- **Fast parsing** using Pest grammar (<1ms)
- **Topological sorting** for dependency resolution
- **Iterative validation** preventing stack overflow
- **Memory-efficient** AST representation
- **Optimized YAML generation** (<2ms)

## Security Features
- **Input validation** at parser level
- **No code injection** in generated YAML
- **Safe file handling** with proper error propagation
- **Secure defaults** in generated configurations

## Project Standards
- **Rust 2021 Edition** with latest stable compiler
- **Error handling** using `thiserror` for typed errors
- **CLI framework** using `clap` v4 with derive macros
- **Parsing** using `pest` for grammar-based parsing
- **YAML generation** using `serde_yaml` for safe serialization
- **Testing** using comprehensive integration tests with GitHub Actions CI/CD
- **Test structure** organized by functionality (structural, CLI, error handling)
- **Lightweight testing** approach focusing on logic over format