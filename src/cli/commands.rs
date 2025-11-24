use std::fs;

use crate::athena::{generate_docker_compose, parse_athena_file, AthenaError, AthenaResult};
use crate::cli::args::Commands;
use crate::cli::utils::{auto_detect_ath_file, should_be_verbose};

pub fn execute_command(command: Option<Commands>, verbose: bool) -> AthenaResult<()> {
    match command {
        // Magic command - no argument, auto-detect and build
        None => {
            if verbose {
                println!("Magic mode: Auto-detecting and building...");
            }
            execute_build(None, None, false, true) // verbose=true by default in magic mode
        }
        Some(Commands::Build {
            input,
            output,
            validate_only,
            quiet,
        }) => {
            let verbose = should_be_verbose(verbose, quiet);
            execute_build(input, output, validate_only, verbose)
        }

        Some(Commands::Validate { input }) => execute_validate(input, verbose),

        Some(Commands::Info {
            examples,
            directives,
        }) => execute_info(examples, directives),
    }
}

fn execute_build(
    input: Option<std::path::PathBuf>,
    output: Option<std::path::PathBuf>,
    validate_only: bool,
    verbose: bool,
) -> AthenaResult<()> {
    // Auto-detection of the .ath file
    let input = auto_detect_ath_file(input)?;
    if verbose {
        println!("Reading Athena file: {}", input.display());
    }

    // Read and parse the input file
    let content = fs::read_to_string(&input).map_err(AthenaError::IoError)?;

    // Automatic validation (always done)
    if verbose {
        println!("Validating syntax...");
    }

    let athena_file = parse_athena_file(&content)?;

    if verbose {
        println!(
            "Successfully parsed Athena file with {} services",
            athena_file.services.services.len()
        );
    }

    if validate_only {
        println!("Athena file is valid");
        return Ok(());
    }

    // Generate docker-compose.yml (includes validation)
    let compose_yaml = generate_docker_compose(&athena_file)?;

    // Determine output file
    let output_path = output.unwrap_or_else(|| "docker-compose.yml".into());

    // Write output
    fs::write(&output_path, &compose_yaml).map_err(AthenaError::IoError)?;

    println!(
        "Generated docker-compose.yml at: {}",
        output_path.display()
    );

    if verbose {
        println!("Project details:");
        println!("   • Project name: {}", athena_file.get_project_name());
        println!("   • Network name: {}", athena_file.get_network_name());
        println!("   • Services: {}", athena_file.services.services.len());

        for service in &athena_file.services.services {
            println!(
                "     - {} ({})",
                service.name,
                service.image.as_deref().unwrap_or("no image")
            );
        }
    }

    Ok(())
}


fn execute_validate(input: Option<std::path::PathBuf>, verbose: bool) -> AthenaResult<()> {
    // Auto-detection of the .ath file
    let input = auto_detect_ath_file(input)?;
    if verbose {
        println!("Validating Athena file: {}", input.display());
    }

    let content = fs::read_to_string(&input).map_err(AthenaError::IoError)?;

    let athena_file = parse_athena_file(&content)?;

    println!("✓ Athena file is valid");

    if verbose {
        println!("Project name: {}", athena_file.get_project_name());
        println!("Services found: {}", athena_file.services.services.len());

        for service in &athena_file.services.services {
            println!(
                "  - {}: {}",
                service.name,
                service.image.as_deref().unwrap_or("no image")
            );
        }
    }

    Ok(())
}

fn execute_info(examples: bool, directives: bool) -> AthenaResult<()> {
    if examples {
        show_examples();
    } else if directives {
        show_directives();
    } else {
        show_general_info();
    }

    Ok(())
}

fn show_general_info() {
    println!("Athena DSL - Docker Compose Generator");
    println!("====================================");
    println!();
    println!("Athena uses a COBOL-inspired DSL to generate Docker Compose files.");
    println!("The syntax is designed to be readable and maintainable.");
    println!();
    println!("Basic structure:");
    println!("  DEPLOYMENT-ID project_name");
    println!("  VERSION-ID 1.0.0");
    println!();
    println!("  ENVIRONMENT SECTION");
    println!("  NETWORK-NAME project_network");
    println!();
    println!("  SERVICES SECTION");
    println!("  SERVICE service_name");
    println!("    IMAGE-ID image:tag");
    println!("    PORT-MAPPING host_port TO container_port");
    println!("    ENV-VARIABLE {{{{VARIABLE_NAME}}}}");
    println!("    COMMAND \"command string\"");
    println!("  END SERVICE");
    println!();
    println!("Use 'athena info --examples' to see complete examples");
    println!("Use 'athena info --directives' to see all available directives");
}

fn show_examples() {
    println!("Athena DSL Examples");
    println!("==================");
    println!();
    println!("Example 1: Simple web application");
    println!("---------------------------------");
    println!(
        r#"DEPLOYMENT-ID WEB_APP
VERSION-ID 1.0.0

ENVIRONMENT SECTION
NETWORK-NAME web_app_network

SERVICES SECTION

SERVICE backend
IMAGE-ID python:3.11-slim
PORT-MAPPING 8000 TO 8000
ENV-VARIABLE {{DATABASE_URL}}
ENV-VARIABLE {{SECRET_KEY}}
COMMAND "uvicorn app.main:app --host 0.0.0.0 --port 8000"
DEPENDS-ON db
HEALTH-CHECK "curl -f http://localhost:8000/health || exit 1"
RESTART-POLICY unless-stopped
END SERVICE

SERVICE db
IMAGE-ID postgres:15
PORT-MAPPING 5432 TO 5432
ENV-VARIABLE {{POSTGRES_USER}}
ENV-VARIABLE {{POSTGRES_PASSWORD}}
ENV-VARIABLE {{POSTGRES_DB}}
VOLUME-MAPPING "./data" TO "/var/lib/postgresql/data"
RESTART-POLICY unless-stopped
END SERVICE
"#
    );

    println!();
    println!("Example 2: Microservices with resources");
    println!("---------------------------------------");
    println!(
        r#"DEPLOYMENT-ID MICROSERVICES
VERSION-ID 2.1.0

ENVIRONMENT SECTION
NETWORK-NAME microservices_net

SERVICES SECTION

SERVICE api
IMAGE-ID node:18-alpine
PORT-MAPPING 3000 TO 3000
ENV-VARIABLE {{NODE_ENV}}
COMMAND "npm start"
RESOURCE-LIMITS CPU "0.5" MEMORY "512M"
END SERVICE

SERVICE redis
IMAGE-ID redis:7-alpine
PORT-MAPPING 6379 TO 6379
VOLUME-MAPPING "./redis-data" TO "/data" (rw)
END SERVICE
"#
    );
}

fn show_directives() {
    println!("Athena DSL Directives Reference");
    println!("==============================");
    println!();

    println!("FILE STRUCTURE");
    println!("  DEPLOYMENT-ID <name>     - Project identifier");
    println!("  VERSION-ID <version>     - Project version (optional)");
    println!();

    println!("ENVIRONMENT SECTION");
    println!("  NETWORK-NAME <name>      - Docker network name");
    println!("  VOLUME <name>            - Define named volume");
    println!("  SECRET <name> <value>    - Define secret value");
    println!();

    println!("SERVICE DIRECTIVES");
    println!("  SERVICE <name> ... END SERVICE - Service definition block");
    println!("  IMAGE-ID <image:tag>            - Docker image");
    println!("  PORT-MAPPING <host> TO <container> [(tcp|udp)] - Port mapping");
    println!("  ENV-VARIABLE {{VAR_NAME}}       - Environment variable template");
    println!("  COMMAND <command>               - Override container command");
    println!("  VOLUME-MAPPING <host> TO <container> [(ro|rw)] - Volume mount");
    println!("  DEPENDS-ON <service>            - Service dependency");
    println!("  HEALTH-CHECK <command>          - Health check command");
    println!("  RESTART-POLICY (always|unless-stopped|on-failure|no)");
    println!("  RESOURCE-LIMITS CPU <limit> MEMORY <limit> - Resource constraints");
    println!();

    println!("EXAMPLES");
    println!("  PORT-MAPPING 8080 TO 80 (tcp)");
    println!("  ENV-VARIABLE {{DATABASE_URL}}");
    println!("  VOLUME-MAPPING \"./data\" TO \"/app/data\" (rw)");
    println!("  RESOURCE-LIMITS CPU \"0.5\" MEMORY \"1G\"");
}
