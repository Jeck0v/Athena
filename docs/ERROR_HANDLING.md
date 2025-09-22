# Enhanced Error Handling System

Athena features a comprehensive error handling system designed to provide clear, actionable feedback with precise location information and automatic suggestions for common issues.

## Key Features

### Line & Column Precision
- **Exact error location** with line and column numbers
- **Visual context** showing the problematic code
- **Error highlighting** pointing to the exact issue

### Intelligent Suggestions
- **Automatic recommendations** for fixing common errors
- **Context-aware suggestions** based on error type
- **Alternative solutions** when multiple fixes are possible

### Advanced Validation
- **Port conflict detection** with alternative suggestions
- **Service reference validation** with available options
- **Circular dependency detection** with clear explanations

##  Parse Error Examples

### Missing END SERVICE Statement

**Input (.ath file):**
```athena
DEPLOYMENT-ID MY_PROJECT

SERVICES SECTION

SERVICE backend
IMAGE-ID "nginx:alpine"
PORT-MAPPING 8080 TO 80
# Missing END SERVICE statement
```

**Enhanced Error Output:**
```
Error: Parse error at line 8, column 1: Missing 'END SERVICE' statement
   |
 8 | # Missing END SERVICE statement
   | ^ Error here

Suggestion: Each SERVICE block must be closed with 'END SERVICE'
```

### Invalid Port Mapping Format

**Input (.ath file):**
```athena
DEPLOYMENT-ID MY_PROJECT

SERVICES SECTION

SERVICE backend
IMAGE-ID "nginx:alpine"
PORT-MAPPING 8080 INVALID_FORMAT 80
END SERVICE
```

**Enhanced Error Output:**
```
Error: Parse error at line 7, column 20: Invalid port mapping format
   |
 7 | PORT-MAPPING 8080 : 80
   |                   ^ Error here

Suggestion: Use PORT-MAPPING <host_port> TO <container_port> format, e.g., PORT-MAPPING 8080 TO 80
```

### Invalid Environment Variable Format

**Input (.ath file):**
```athena
DEPLOYMENT-ID MY_PROJECT

SERVICES SECTION

SERVICE backend
IMAGE-ID "nginx:alpine"
ENV-VARIABLE INVALID_VAR_FORMAT
END SERVICE
```

**Enhanced Error Output:**
```
Error: Parse error at line 7, column 1: Invalid environment variable format
   |
 7 | ENV-VARIABLE INVALID_VAR_FORMAT
   | ^ Error here

Suggestion: Use ENV-VARIABLE {{VAR_NAME}} for templates or ENV-VARIABLE "literal_value" for literals (not recommended)
```

## Validation Error Examples

### Port Conflicts Detection

**Input (.ath file):**
```athena
DEPLOYMENT-ID PORT_CONFLICT_DEMO

SERVICES SECTION

SERVICE frontend
IMAGE-ID "nginx:alpine"
PORT-MAPPING 8080 TO 80
END SERVICE

SERVICE backend
IMAGE-ID "node:18-alpine"
PORT-MAPPING 8080 TO 3000  # Conflict: same host port!
END SERVICE

SERVICE api
IMAGE-ID "python:3.11-alpine"
PORT-MAPPING 8080 TO 8000  # Another conflict!
END SERVICE
```

**Enhanced Error Output:**
```
Error: Validation error: Port conflict detected! Host port 8080 is used by multiple services: frontend, backend, api
Affected services: frontend, backend, api

Suggestion: Use different host ports, e.g., 8080, 8081, 8082
```

### Service Reference Validation

**Input (.ath file):**
```athena
DEPLOYMENT-ID REFERENCE_DEMO

SERVICES SECTION

SERVICE frontend
IMAGE-ID "nginx:alpine"
PORT-MAPPING 8080 TO 80
DEPENDS-ON nonexistent_backend  # Invalid reference!
END SERVICE

SERVICE database
IMAGE-ID "postgres:15"
PORT-MAPPING 5432 TO 5432
END SERVICE
```

**Enhanced Error Output:**
```
Error: Validation error: Service 'frontend' depends on 'nonexistent_backend' which doesn't exist
Affected services: frontend, nonexistent_backend

Suggestion: Available services: database, frontend. Check the service name in your DEPENDS-ON declaration
```

### Circular Dependency Detection

**Input (.ath file):**
```athena
DEPLOYMENT-ID CIRCULAR_DEMO

SERVICES SECTION

SERVICE service_a
IMAGE-ID "nginx:alpine"
DEPENDS-ON service_b
END SERVICE

SERVICE service_b
IMAGE-ID "node:18-alpine"
DEPENDS-ON service_c
END SERVICE

SERVICE service_c
IMAGE-ID "postgres:15"
DEPENDS-ON service_a  # Creates circular dependency!
END SERVICE
```

**Enhanced Error Output:**
```
Error: Validation error: Circular dependency detected involving service 'service_a'
Affected services: service_a

Suggestion: Check the DEPENDS-ON declarations in your .ath file and remove circular dependencies
```

## Error Categories

### 1. Syntax Errors (Parse Errors)
- **Missing keywords** (END SERVICE, DEPLOYMENT-ID)
- **Invalid formats** (port mappings, environment variables)
- **Malformed structures** (unclosed blocks, missing sections)

### 2. Validation Errors
- **Port conflicts** between services
- **Invalid service references** in dependencies
- **Circular dependencies** in service chains
- **Missing required configurations**

### 3. Configuration Errors
- **Invalid restart policies**
- **Malformed resource limits**
- **Incorrect volume mappings**

## Error Resolution Process

### 1. **Immediate Feedback**
Athena stops processing as soon as an error is detected, preventing cascading issues.

### 2. **Precise Location**
Line and column numbers help you jump directly to the problematic code in your editor.

### 3. **Context Visualization**
Visual representation shows exactly where the error occurs within the line.

### 4. **Actionable Suggestions**
Each error includes specific recommendations for fixing the issue.

### 5. **Related Information**
For validation errors, Athena shows affected services and available alternatives.

## Best Practices

### Writing Error-Free .ath Files

1. **Always close SERVICE blocks** with `END SERVICE`
2. **Use consistent formatting** for port mappings: `PORT-MAPPING <host> TO <container>`
3. **Template variables** should use double braces: `{{VARIABLE_NAME}}`
4. **Check service names** in DEPENDS-ON declarations
5. **Avoid port conflicts** by using unique host ports

### Debugging Tips

1. **Start simple** - build your .ath file incrementally
2. **Use validation mode** with `athena validate file.ath`
3. **Check dependencies** - ensure referenced services exist
4. **Review port mappings** - each service needs unique host ports
5. **Test incrementally** - add services one by one

##  Error Handling Architecture

The enhanced error system uses:

- **Structured error types** with rich context information
- **Location tracking** throughout the parsing pipeline
- **Suggestion engine** that provides context-aware recommendations
- **Validation pipeline** that catches logical errors before generation
- **User-friendly formatting** with visual indicators and clear language

This system ensures that debugging .ath files is fast, intuitive, and educational, helping users learn the DSL syntax and best practices through clear error messages.
