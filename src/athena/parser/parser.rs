use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

use crate::athena::error::{AthenaError, AthenaResult, EnhancedParseError};
use super::ast::*;

#[derive(Parser)]
#[grammar = "athena/parser/grammar.pest"]
pub struct AthenaParser;

pub fn parse_athena_file(input: &str) -> AthenaResult<AthenaFile> {
    let pairs = AthenaParser::parse(Rule::athena_file, input)
        .map_err(|e| {
            // Extract location information from Pest error
            let (line, column) = match &e.location {
                pest::error::InputLocation::Pos(pos) => {
                    let line_col = pest::Position::new(input, *pos)
                        .map(|p| (p.line_col().0, p.line_col().1))
                        .unwrap_or((1, 1));
                    line_col
                }
                pest::error::InputLocation::Span((start, _)) => {
                    let line_col = pest::Position::new(input, *start)
                        .map(|p| (p.line_col().0, p.line_col().1))
                        .unwrap_or((1, 1));
                    line_col
                }
            };
            
            // Create enhanced error with context and suggestions
            let enhanced_error = create_enhanced_parse_error(&e, line, column, input);
            AthenaError::parse_error_enhanced(enhanced_error)
        })?;

    let mut athena_file = AthenaFile::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::athena_file => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::deployment_section => {
                            athena_file.deployment = Some(parse_deployment_section(inner_pair)?);
                        }
                        Rule::environment_section => {
                            athena_file.environment = Some(parse_environment_section(inner_pair)?);
                        }
                        Rule::services_section => {
                            athena_file.services = parse_services_section(inner_pair)?;
                        }
                        Rule::EOI => {} // End of input
                        _ => return Err(AthenaError::ParseError(
                            EnhancedParseError::new(format!("Unexpected rule: {:?}", inner_pair.as_rule()))
                        )),
                    }
                }
            }
            _ => return Err(AthenaError::ParseError(
                EnhancedParseError::new("Expected athena_file rule".to_string())
            )),
        }
    }

    Ok(athena_file)
}

fn parse_deployment_section(pair: pest::iterators::Pair<Rule>) -> AthenaResult<DeploymentSection> {
    let mut deployment_id = None;
    let mut version_id = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::deployment_id => {
                if let Some(id_pair) = inner_pair.into_inner().next() {
                    deployment_id = Some(id_pair.as_str().to_string());
                }
            }
            Rule::version_id => {
                if let Some(version_pair) = inner_pair.into_inner().next() {
                    version_id = Some(version_pair.as_str().to_string());
                }
            }
            _ => {}
        }
    }

    let deployment_id = deployment_id.ok_or_else(|| 
        AthenaError::ParseError(EnhancedParseError::new("Missing deployment ID".to_string()))
    )?;

    Ok(DeploymentSection {
        deployment_id,
        version_id,
    })
}

fn parse_environment_section(pair: pest::iterators::Pair<Rule>) -> AthenaResult<EnvironmentSection> {
    let mut network_name = None;
    let mut volumes = Vec::new();
    let mut secrets = HashMap::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::environment_item => {
                for item_pair in inner_pair.into_inner() {
                    match item_pair.as_rule() {
                        Rule::network_name => {
                            if let Some(name_pair) = item_pair.into_inner().next() {
                                network_name = Some(name_pair.as_str().to_string());
                            }
                        }
                        Rule::volume_def => {
                            volumes.push(parse_volume_definition(item_pair)?);
                        }
                        Rule::secret_def => {
                            let mut inner = item_pair.into_inner();
                            if let (Some(key), Some(value)) = (inner.next(), inner.next()) {
                                secrets.insert(
                                    key.as_str().to_string(),
                                    clean_string_value(value.as_str())
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(EnvironmentSection {
        network_name,
        volumes,
        secrets,
    })
}

fn parse_volume_definition(pair: pest::iterators::Pair<Rule>) -> AthenaResult<VolumeDefinition> {
    let mut name = None;
    let mut options = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier => {
                name = Some(inner_pair.as_str().to_string());
            }
            Rule::volume_options => {
                for option_pair in inner_pair.into_inner() {
                    if let Rule::volume_option = option_pair.as_rule() {
                        options.push(option_pair.as_str().to_string());
                    }
                }
            }
            _ => {}
        }
    }

    let name = name.ok_or_else(|| 
        AthenaError::ParseError(EnhancedParseError::new("Missing volume name".to_string()))
    )?;

    Ok(VolumeDefinition { name, options })
}

fn parse_services_section(pair: pest::iterators::Pair<Rule>) -> AthenaResult<ServicesSection> {
    let mut services = Vec::new();

    for inner_pair in pair.into_inner() {
        if let Rule::service = inner_pair.as_rule() {
            services.push(parse_service(inner_pair)?);
        }
    }

    Ok(ServicesSection { services })
}

fn parse_service(pair: pest::iterators::Pair<Rule>) -> AthenaResult<Service> {
    let mut service_name = None;
    let mut service = Service::new(String::new());

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::service_name => {
                service_name = Some(inner_pair.as_str().to_string());
            }
            Rule::service_items => {
                for item_pair in inner_pair.into_inner() {
                    parse_service_item(item_pair, &mut service)?;
                }
            }
            _ => {}
        }
    }

    let service_name = service_name.ok_or_else(|| 
        AthenaError::ParseError(EnhancedParseError::new("Missing service name".to_string()))
    )?;

    service.name = service_name;
    Ok(service)
}

fn parse_service_item(pair: pest::iterators::Pair<Rule>, service: &mut Service) -> AthenaResult<()> {
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::image_id => {
                if let Some(image_pair) = inner_pair.into_inner().next() {
                    service.image = Some(clean_string_value(image_pair.as_str()));
                }
            }
            Rule::port_mapping => {
                service.ports.push(parse_port_mapping(inner_pair)?);
            }
            Rule::env_variable => {
                service.environment.push(parse_env_variable(inner_pair)?);
            }
            Rule::command_line => {
                if let Some(cmd_pair) = inner_pair.into_inner().next() {
                    service.command = Some(clean_string_value(cmd_pair.as_str()));
                }
            }
            Rule::volume_mapping => {
                service.volumes.push(parse_volume_mapping(inner_pair)?);
            }
            Rule::depends_on => {
                if let Some(dep_pair) = inner_pair.into_inner().next() {
                    service.depends_on.push(dep_pair.as_str().to_string());
                }
            }
            Rule::health_check => {
                if let Some(health_pair) = inner_pair.into_inner().next() {
                    service.health_check = Some(clean_string_value(health_pair.as_str()));
                }
            }
            Rule::restart_policy => {
                service.restart = Some(parse_restart_policy(inner_pair)?);
            }
            Rule::resource_limits => {
                service.resources = Some(parse_resource_limits(inner_pair)?);
            }
            Rule::build_args => {
                service.build_args = Some(parse_build_args(inner_pair)?);
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_port_mapping(pair: pest::iterators::Pair<Rule>) -> AthenaResult<PortMapping> {
    let mut inner = pair.into_inner();
    let host_port = inner.next()
        .ok_or_else(|| AthenaError::ParseError(EnhancedParseError::new("Missing host port".to_string())))?
        .as_str()
        .parse::<u16>()
        .map_err(|_| AthenaError::ParseError(EnhancedParseError::new("Invalid host port".to_string())))?;

    let container_port = inner.next()
        .ok_or_else(|| AthenaError::ParseError(EnhancedParseError::new("Missing container port".to_string())))?
        .as_str()
        .parse::<u16>()
        .map_err(|_| AthenaError::ParseError(EnhancedParseError::new("Invalid container port".to_string())))?;

    let mut protocol = Protocol::Tcp;
    if let Some(protocol_pair) = inner.next() {
        if let Rule::port_protocol = protocol_pair.as_rule() {
            let proto_str = protocol_pair.into_inner().next()
                .ok_or_else(|| AthenaError::ParseError(EnhancedParseError::new("Missing protocol".to_string())))?
                .as_str();
            protocol = match proto_str {
                "tcp" => Protocol::Tcp,
                "udp" => Protocol::Udp,
                _ => Protocol::Tcp,
            };
        }
    }

    Ok(PortMapping {
        host_port,
        container_port,
        protocol,
    })
}

fn parse_env_variable(pair: pest::iterators::Pair<Rule>) -> AthenaResult<EnvironmentVariable> {
    let inner = pair.into_inner().next()
        .ok_or_else(|| AthenaError::ParseError(EnhancedParseError::new("Missing environment variable".to_string())))?;

    match inner.as_rule() {
        Rule::template_var => {
            let var_name = inner.as_str().trim_start_matches("{{").trim_end_matches("}}").to_string();
            Ok(EnvironmentVariable::Template(var_name))
        }
        Rule::string_value => {
            Ok(EnvironmentVariable::Literal(clean_string_value(inner.as_str())))
        }
        _ => Err(AthenaError::ParseError(EnhancedParseError::new("Invalid environment variable".to_string())))
    }
}

fn parse_volume_mapping(pair: pest::iterators::Pair<Rule>) -> AthenaResult<VolumeMapping> {
    let mut inner = pair.into_inner();
    let host_path = clean_string_value(
        inner.next()
            .ok_or_else(|| AthenaError::ParseError(EnhancedParseError::new("Missing host path".to_string())))?
            .as_str()
    );

    let container_path = clean_string_value(
        inner.next()
            .ok_or_else(|| AthenaError::ParseError(EnhancedParseError::new("Missing container path".to_string())))?
            .as_str()
    );

    let mut options = Vec::new();
    if let Some(options_pair) = inner.next() {
        if let Rule::volume_options = options_pair.as_rule() {
            for option_pair in options_pair.into_inner() {
                if let Rule::volume_option = option_pair.as_rule() {
                    options.push(option_pair.as_str().to_string());
                }
            }
        }
    }

    Ok(VolumeMapping {
        host_path,
        container_path,
        options,
    })
}

fn parse_restart_policy(pair: pest::iterators::Pair<Rule>) -> AthenaResult<RestartPolicy> {
    let mut inner = pair.into_inner();
    let policy_str = inner.next()
        .ok_or_else(|| AthenaError::ParseError(EnhancedParseError::new("Missing restart policy".to_string())))?
        .as_str();

    match policy_str {
        "always" => Ok(RestartPolicy::Always),
        "unless-stopped" => Ok(RestartPolicy::UnlessStopped),
        "on-failure" => Ok(RestartPolicy::OnFailure),
        "no" => Ok(RestartPolicy::No),
        _ => Err(AthenaError::ParseError(EnhancedParseError::new(format!("Invalid restart policy: {}", policy_str))))
    }
}

fn parse_resource_limits(pair: pest::iterators::Pair<Rule>) -> AthenaResult<ResourceLimits> {
    let inner_pairs: Vec<_> = pair.into_inner().collect();
    
    // The grammar expects: "CPU" string_value "MEMORY" string_value
    // But pest parses only the string values, skipping keywords
    if inner_pairs.len() != 2 {
        return Err(AthenaError::ParseError(EnhancedParseError::new(
            format!("Expected 2 values for resource limits, got {}", inner_pairs.len())
        )));
    }
    
    // First string_value is CPU, second is MEMORY
    let cpu = clean_string_value(inner_pairs[0].as_str());
    let memory = clean_string_value(inner_pairs[1].as_str());

    Ok(ResourceLimits { cpu, memory })
}

fn parse_build_args(pair: pest::iterators::Pair<Rule>) -> AthenaResult<HashMap<String, String>> {
    let mut build_args = HashMap::new();
    
    for inner_pair in pair.into_inner() {
        if let Rule::build_arg_pair = inner_pair.as_rule() {
            let mut arg_parts = inner_pair.into_inner();
            
            let key = arg_parts.next()
                .ok_or_else(|| AthenaError::ParseError(EnhancedParseError::new("Missing build arg key".to_string())))?
                .as_str().to_string();
            
            let value = arg_parts.next()
                .ok_or_else(|| AthenaError::ParseError(EnhancedParseError::new("Missing build arg value".to_string())))?
                .as_str();
            
            build_args.insert(key, clean_string_value(value));
        }
    }
    
    if build_args.is_empty() {
        return Err(AthenaError::ParseError(EnhancedParseError::new(
            "BUILD-ARGS must contain at least one key=value pair".to_string()
        )));
    }
    
    Ok(build_args)
}

fn clean_string_value(input: &str) -> String {
    if input.starts_with('"') && input.ends_with('"') {
        input[1..input.len()-1].to_string()
    } else {
        input.to_string()
    }
}

fn create_enhanced_parse_error(
    pest_error: &pest::error::Error<Rule>,
    line: usize,
    column: usize,
    file_content: &str,
) -> EnhancedParseError {
    let base_message = format!("{}", pest_error);
    
    // Extract meaningful error message from Pest error
    let (clean_message, suggestion) = match &pest_error.variant {
        pest::error::ErrorVariant::ParsingError { 
            positives, 
            negatives: _ 
        } => {
            if positives.contains(&Rule::athena_file) {
                (
                    "Invalid file structure".to_string(),
                    Some("Expected DEPLOYMENT-ID followed by SERVICES SECTION".to_string())
                )
            } else if positives.contains(&Rule::service) {
                (
                    "Expected service definition".to_string(),
                    Some("Service blocks must start with 'SERVICE <name>' and end with 'END SERVICE'".to_string())
                )
            } else if positives.contains(&Rule::service_name) {
                (
                    "Missing service name".to_string(),
                    Some("Add a service name after 'SERVICE', e.g., 'SERVICE backend'".to_string())
                )
            } else if positives.contains(&Rule::image_id) {
                (
                    "Invalid IMAGE-ID format".to_string(),
                    Some("Use IMAGE-ID \"image:tag\" format, e.g., IMAGE-ID \"nginx:alpine\"".to_string())
                )
            } else if positives.contains(&Rule::port_mapping) {
                (
                    "Invalid port mapping format".to_string(),
                    Some("Use PORT-MAPPING <host_port> TO <container_port> format, e.g., PORT-MAPPING 8080 TO 80".to_string())
                )
            } else if positives.contains(&Rule::env_variable) {
                (
                    "Invalid environment variable format".to_string(),
                    Some("Use ENV-VARIABLE {{VAR_NAME}} for templates or ENV-VARIABLE \"literal_value\" for literals".to_string())
                )
            } else if positives.contains(&Rule::restart_policy) {
                (
                    "Invalid restart policy".to_string(),
                    Some("Valid restart policies: always, unless-stopped, on-failure, no".to_string())
                )
            } else if positives.contains(&Rule::resource_limits) {
                (
                    "Invalid resource limits format".to_string(),
                    Some("Use RESOURCE-LIMITS CPU \"0.5\" MEMORY \"512M\" format".to_string())
                )
            } else if positives.contains(&Rule::build_args) {
                (
                    "Invalid BUILD-ARGS format".to_string(),
                    Some("Use BUILD-ARGS KEY=\"value\" KEY2=\"value2\" format, e.g., BUILD-ARGS NODE_VERSION=\"20\" BUILD_ENV=\"production\"".to_string())
                )
            } else {
                // Check for unclosed comment errors
                if file_content.contains("/*") && file_content.matches("/*").count() != file_content.matches("*/").count() {
                    (
                        "Unclosed multi-line comment".to_string(),
                        Some("Multi-line comments must be closed with '*/'. Each '/*' must have a matching '*/'".to_string())
                    )
                }
                // Check for common missing END SERVICE error
                else if base_message.contains("end of input") || base_message.contains("EOI") {
                    (
                        "Missing 'END SERVICE' statement".to_string(),
                        Some("Each SERVICE block must be closed with 'END SERVICE'".to_string())
                    )
                } else {
                    (
                        extract_clean_message(&base_message),
                        generate_generic_suggestion(&positives)
                    )
                }
            }
        }
        pest::error::ErrorVariant::CustomError { message } => {
            (message.clone(), None)
        }
    };
    
    EnhancedParseError::new(clean_message)
        .with_location(line, column)
        .with_file_content(file_content.to_string())
        .with_suggestion(suggestion.unwrap_or_else(|| "Check the syntax in your .ath file".to_string()))
}

fn extract_clean_message(pest_message: &str) -> String {
    // Remove technical Pest details and make message user-friendly
    if pest_message.contains("expected") {
        let parts: Vec<&str> = pest_message.split(" --> ").collect();
        if let Some(first_part) = parts.first() {
            return first_part.trim().to_string();
        }
    }
    
    pest_message.to_string()
}

fn generate_generic_suggestion(expected_rules: &[Rule]) -> Option<String> {
    if expected_rules.is_empty() {
        return None;
    }
    
    let suggestions: Vec<String> = expected_rules.iter().filter_map(|rule| {
        match rule {
            Rule::deployment_id => Some("Add DEPLOYMENT-ID <project_name>".to_string()),
            Rule::services_section => Some("Add SERVICES SECTION block".to_string()),
            Rule::service => Some("Define services with SERVICE <name> ... END SERVICE".to_string()),
            Rule::image_id => Some("Add IMAGE-ID \"image:tag\"".to_string()),
            Rule::port_mapping => Some("Add PORT-MAPPING <host_port> TO <container_port>".to_string()),
            Rule::env_variable => Some("Add ENV-VARIABLE {{VAR_NAME}}".to_string()),
            _ => None,
        }
    }).collect();
    
    if suggestions.is_empty() {
        None
    } else {
        Some(format!("Try: {}", suggestions.join(" or ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let input = r#"
            DEPLOYMENT-ID TEST_PROJECT
            VERSION-ID 1.0.0

            SERVICES SECTION

            SERVICE backend
            IMAGE-ID "python:3.11-slim"
            PORT-MAPPING 8000 TO 8000
            ENV-VARIABLE {{DATABASE_URL}}
            COMMAND "uvicorn app.main:app --host 0.0.0.0"
            END SERVICE
        "#;

        let result = parse_athena_file(input);
        assert!(result.is_ok());

        let athena_file = result.unwrap();
        assert!(athena_file.deployment.is_some());
        assert_eq!(athena_file.services.services.len(), 1);
        
        let service = &athena_file.services.services[0];
        assert_eq!(service.name, "backend");
        assert_eq!(service.image, Some("python:3.11-slim".to_string()));
        assert_eq!(service.ports.len(), 1);
        assert_eq!(service.environment.len(), 1);
    }

    #[test]
    fn test_resource_limits_parsing() {
        let input = r#"RESOURCE-LIMITS CPU "0.5" MEMORY "512M""#;
        
        match AthenaParser::parse(Rule::resource_limits, input) {
            Ok(mut pairs) => {
                if let Some(pair) = pairs.next() {
                    println!("Parsing resource limits:");
                    for inner in pair.into_inner() {
                        println!("  Rule: {:?}, Value: '{}'", inner.as_rule(), inner.as_str());
                    }
                    
                    // Test the actual parsing function
                    match AthenaParser::parse(Rule::resource_limits, input) {
                        Ok(mut pairs) => {
                            let result = parse_resource_limits(pairs.next().unwrap());
                            println!("Parse result: {:?}", result);
                        }
                        Err(e) => println!("Parse error: {:?}", e),
                    }
                } else {
                    println!("No pairs found");
                }
            }
            Err(e) => {
                println!("Grammar parse error: {:?}", e);
            }
        }
    }

    #[test]
    fn test_build_args_parsing() {
        let input = r#"
            DEPLOYMENT-ID TEST_PROJECT
            
            SERVICES SECTION
            
            SERVICE api
            BUILD-ARGS BUILD_ENV="production" NODE_VERSION="20"
            ENV-VARIABLE {{API_KEY}}
            END SERVICE
        "#;

        let result = parse_athena_file(input);
        assert!(result.is_ok());

        let athena_file = result.unwrap();
        assert_eq!(athena_file.services.services.len(), 1);
        
        let service = &athena_file.services.services[0];
        assert_eq!(service.name, "api");
        assert!(service.build_args.is_some());
        
        let build_args = service.build_args.as_ref().unwrap();
        assert_eq!(build_args.get("BUILD_ENV"), Some(&"production".to_string()));
        assert_eq!(build_args.get("NODE_VERSION"), Some(&"20".to_string()));
        assert_eq!(build_args.len(), 2);
    }

    #[test]
    fn test_build_args_single_pair() {
        let input = r#"BUILD-ARGS NODE_ENV="development""#;
        
        match AthenaParser::parse(Rule::build_args, input) {
            Ok(mut pairs) => {
                if let Some(pair) = pairs.next() {
                    let result = parse_build_args(pair);
                    assert!(result.is_ok());
                    
                    let build_args = result.unwrap();
                    assert_eq!(build_args.get("NODE_ENV"), Some(&"development".to_string()));
                    assert_eq!(build_args.len(), 1);
                }
            }
            Err(e) => panic!("Parse error: {:?}", e),
        }
    }

    #[test]
    fn test_build_args_multiple_pairs() {
        let input = r#"BUILD-ARGS ENV="prod" VERSION="1.2.3" DEBUG="false""#;
        
        match AthenaParser::parse(Rule::build_args, input) {
            Ok(mut pairs) => {
                if let Some(pair) = pairs.next() {
                    let result = parse_build_args(pair);
                    assert!(result.is_ok());
                    
                    let build_args = result.unwrap();
                    assert_eq!(build_args.get("ENV"), Some(&"prod".to_string()));
                    assert_eq!(build_args.get("VERSION"), Some(&"1.2.3".to_string()));
                    assert_eq!(build_args.get("DEBUG"), Some(&"false".to_string()));
                    assert_eq!(build_args.len(), 3);
                }
            }
            Err(e) => panic!("Parse error: {:?}", e),
        }
    }
}