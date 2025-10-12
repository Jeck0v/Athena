use athena::athena::parser::parser::parse_athena_file;
use athena::athena::generator::compose::generate_docker_compose;

#[test]
fn test_swarm_replicas_parsing() {
    let input = r#"
        DEPLOYMENT-ID SWARM_TEST
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        REPLICAS 3
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_ok());

    let athena_file = result.unwrap();
    assert_eq!(athena_file.services.services.len(), 1);
    
    let service = &athena_file.services.services[0];
    assert_eq!(service.name, "web");
    assert!(service.swarm_config.is_some());
    
    let swarm_config = service.swarm_config.as_ref().unwrap();
    assert_eq!(swarm_config.replicas, Some(3));
}

#[test]
fn test_swarm_update_config_parsing() {
    let input = r#"
        DEPLOYMENT-ID SWARM_UPDATE_TEST
        
        SERVICES SECTION
        
        SERVICE api
        IMAGE-ID python:3.11
        REPLICAS 2
        UPDATE-CONFIG PARALLELISM 1 DELAY 30s FAILURE-ACTION ROLLBACK
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_ok());

    let athena_file = result.unwrap();
    let service = &athena_file.services.services[0];
    let swarm_config = service.swarm_config.as_ref().unwrap();
    
    assert_eq!(swarm_config.replicas, Some(2));
    assert!(swarm_config.update_config.is_some());
    
    let update_config = swarm_config.update_config.as_ref().unwrap();
    assert_eq!(update_config.parallelism, Some(1));
    assert_eq!(update_config.delay, Some("30s".to_string()));
    assert!(update_config.failure_action.is_some());
}

#[test]
fn test_swarm_labels_parsing() {
    let input = r#"
        DEPLOYMENT-ID SWARM_LABELS_TEST
        
        SERVICES SECTION
        
        SERVICE app
        IMAGE-ID node:18
        SWARM-LABELS environment="production" tier="backend"
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_ok());

    let athena_file = result.unwrap();
    let service = &athena_file.services.services[0];
    let swarm_config = service.swarm_config.as_ref().unwrap();
    
    assert!(swarm_config.labels.is_some());
    let labels = swarm_config.labels.as_ref().unwrap();
    assert_eq!(labels.get("environment"), Some(&"production".to_string()));
    assert_eq!(labels.get("tier"), Some(&"backend".to_string()));
    assert_eq!(labels.len(), 2);
}

#[test]
fn test_overlay_network_parsing() {
    let input = r#"
        DEPLOYMENT-ID OVERLAY_TEST
        
        ENVIRONMENT SECTION
        NETWORK-NAME overlay_net DRIVER OVERLAY ATTACHABLE TRUE ENCRYPTED FALSE
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_ok());

    let athena_file = result.unwrap();
    assert!(athena_file.environment.is_some());
    
    let env = athena_file.environment.as_ref().unwrap();
    assert_eq!(env.networks.len(), 1);
    
    let network = &env.networks[0];
    assert_eq!(network.name, "overlay_net");
    assert!(network.driver.is_some());
    assert_eq!(network.attachable, Some(true));
    assert_eq!(network.encrypted, Some(false));
}

#[test]
fn test_complete_swarm_compose_generation() {
    let input = r#"
        DEPLOYMENT-ID COMPLETE_SWARM
        VERSION-ID 2.0.0
        
        ENVIRONMENT SECTION
        NETWORK-NAME swarm_overlay DRIVER OVERLAY ATTACHABLE TRUE
        
        SERVICES SECTION
        
        SERVICE frontend
        BUILD-ARGS NODE_ENV="production"
        PORT-MAPPING 80 TO 3000
        REPLICAS 2
        UPDATE-CONFIG PARALLELISM 1 DELAY 10s
        SWARM-LABELS tier="frontend" env="prod"
        DEPENDS-ON backend
        END SERVICE
        
        SERVICE backend
        IMAGE-ID python:3.11-slim
        REPLICAS 3
        SWARM-LABELS tier="backend" env="prod"
        RESOURCE-LIMITS CPU "0.5" MEMORY "512M"
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_ok());

    let athena_file = result.unwrap();
    let compose_result = generate_docker_compose(&athena_file);
    assert!(compose_result.is_ok());

    let yaml = compose_result.unwrap();
    
    // Verify Swarm-specific configurations in generated YAML
    assert!(yaml.contains("replicas: 2"));
    assert!(yaml.contains("replicas: 3"));
    assert!(yaml.contains("driver: overlay"));
    assert!(yaml.contains("attachable: true"));
    assert!(yaml.contains("parallelism: 1"));
    assert!(yaml.contains("delay: 10s"));
    assert!(yaml.contains("tier: frontend"));
    assert!(yaml.contains("tier: backend"));
    assert!(yaml.contains("env: prod"));
}

#[test]
fn test_mixed_compose_and_swarm_features() {
    let input = r#"
        DEPLOYMENT-ID MIXED_TEST
        
        SERVICES SECTION
        
        SERVICE standard_service
        IMAGE-ID alpine:latest
        PORT-MAPPING 8080 TO 80
        END SERVICE
        
        SERVICE swarm_service
        IMAGE-ID nginx:alpine
        REPLICAS 5
        UPDATE-CONFIG PARALLELISM 2 DELAY 5s FAILURE-ACTION PAUSE
        SWARM-LABELS scaling="auto" priority="high"
        RESOURCE-LIMITS CPU "1.0" MEMORY "1024M"
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_ok());

    let athena_file = result.unwrap();
    assert_eq!(athena_file.services.services.len(), 2);
    
    let standard_service = &athena_file.services.services[0];
    assert!(standard_service.swarm_config.is_none());
    
    let swarm_service = &athena_file.services.services[1];
    assert!(swarm_service.swarm_config.is_some());
    
    let swarm_config = swarm_service.swarm_config.as_ref().unwrap();
    assert_eq!(swarm_config.replicas, Some(5));
    assert!(swarm_config.update_config.is_some());
    assert!(swarm_config.labels.is_some());
}

// ========== ERROR HANDLING TESTS ==========

#[test]
fn test_invalid_replica_negative_number() {
    let input = r#"
        DEPLOYMENT-ID INVALID_REPLICAS_TEST
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        REPLICAS -5
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_err());
    
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Invalid replicas number") || error_msg.contains("Parse error"));
}

#[test]
fn test_invalid_replica_extremely_large_number() {
    let input = r#"
        DEPLOYMENT-ID LARGE_REPLICAS_TEST
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        REPLICAS 999999999999999999999
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_err());
    
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Invalid replicas number") || error_msg.contains("Parse error"));
}

#[test]
fn test_invalid_replica_zero() {
    let input = r#"
        DEPLOYMENT-ID ZERO_REPLICAS_TEST
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        REPLICAS 0
        END SERVICE
    "#;

    // Zero replicas should parse but might be logically invalid for deployment
    let result = parse_athena_file(input);
    assert!(result.is_ok());
    
    let athena_file = result.unwrap();
    let service = &athena_file.services.services[0];
    let swarm_config = service.swarm_config.as_ref().unwrap();
    assert_eq!(swarm_config.replicas, Some(0));
}

#[test]
fn test_invalid_replica_non_numeric() {
    let input = r#"
        DEPLOYMENT-ID NON_NUMERIC_REPLICAS_TEST
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        REPLICAS abc
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_err());
    
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Invalid replicas number") || error_msg.contains("Parse error"));
}

#[test]
fn test_swarm_labels_without_quotes_should_work() {
    // Actually, this should work - the parser is flexible
    let input = r#"
        DEPLOYMENT-ID FLEXIBLE_LABELS_TEST
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        SWARM-LABELS environment=production tier=backend
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_ok());
    
    let athena_file = result.unwrap();
    let service = &athena_file.services.services[0];
    let swarm_config = service.swarm_config.as_ref().unwrap();
    let labels = swarm_config.labels.as_ref().unwrap();
    assert_eq!(labels.get("environment"), Some(&"production".to_string()));
    assert_eq!(labels.get("tier"), Some(&"backend".to_string()));
}

#[test]
fn test_invalid_swarm_labels_malformed_missing_value() {
    let input = r#"
        DEPLOYMENT-ID MALFORMED_LABELS_TEST
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        SWARM-LABELS environment="production" tier=
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_err());
    
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Parse error"));
}

#[test]
fn test_empty_swarm_labels() {
    let input = r#"
        DEPLOYMENT-ID EMPTY_LABELS_TEST
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        SWARM-LABELS
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_err());
    
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("must contain at least one key=value pair") || error_msg.contains("Parse error"));
}

#[test]
fn test_invalid_update_config_negative_parallelism() {
    let input = r#"
        DEPLOYMENT-ID INVALID_UPDATE_CONFIG_TEST
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        UPDATE-CONFIG PARALLELISM -1 DELAY 10s
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_err());
    
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Invalid parallelism number") || error_msg.contains("Parse error"));
}

#[test]
fn test_invalid_update_config_invalid_delay_format() {
    let input = r#"
        DEPLOYMENT-ID INVALID_DELAY_TEST
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        UPDATE-CONFIG PARALLELISM 1 DELAY invalid_time
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_err());
    
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Parse error"));
}

#[test]
fn test_invalid_failure_action() {
    let input = r#"
        DEPLOYMENT-ID INVALID_FAILURE_ACTION_TEST
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        UPDATE-CONFIG PARALLELISM 1 DELAY 10s FAILURE-ACTION INVALID_ACTION
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_err());
    
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Parse error"));
}

#[test]
fn test_invalid_max_failure_ratio() {
    let input = r#"
        DEPLOYMENT-ID INVALID_RATIO_TEST
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        UPDATE-CONFIG PARALLELISM 1 MAX-FAILURE-RATIO 1.5
        END SERVICE
    "#;

    // Ratio > 1.0 should parse but might be logically invalid
    let result = parse_athena_file(input);
    assert!(result.is_ok());
    
    let athena_file = result.unwrap();
    let service = &athena_file.services.services[0];
    let swarm_config = service.swarm_config.as_ref().unwrap();
    let update_config = swarm_config.update_config.as_ref().unwrap();
    assert_eq!(update_config.max_failure_ratio, Some(1.5));
}

#[test]
fn test_invalid_network_driver() {
    let input = r#"
        DEPLOYMENT-ID INVALID_DRIVER_TEST
        
        ENVIRONMENT SECTION
        NETWORK-NAME test_net DRIVER INVALID_DRIVER
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_err());
    
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Parse error"));
}

#[test]
fn test_invalid_boolean_values() {
    let input = r#"
        DEPLOYMENT-ID INVALID_BOOLEAN_TEST
        
        ENVIRONMENT SECTION
        NETWORK-NAME test_net DRIVER OVERLAY ATTACHABLE MAYBE
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_err());
    
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Parse error"));
}

#[test]
fn test_swarm_config_without_service_name() {
    let input = r#"
        DEPLOYMENT-ID MISSING_SERVICE_NAME_TEST
        
        SERVICES SECTION
        
        SERVICE
        REPLICAS 3
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_err());
    
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Parse error") || error_msg.contains("Missing service name"));
}

#[test]
fn test_conflicting_swarm_and_compose_features() {
    // This should parse successfully but could show warnings or conflicts
    let input = r#"
        DEPLOYMENT-ID CONFLICT_TEST
        
        SERVICES SECTION
        
        SERVICE web
        IMAGE-ID nginx:alpine
        PORT-MAPPING 80 TO 80
        REPLICAS 3
        UPDATE-CONFIG PARALLELISM 1 DELAY 10s
        RESTART-POLICY always
        END SERVICE
    "#;

    let result = parse_athena_file(input);
    assert!(result.is_ok());
    
    // Verify both Compose and Swarm features are present
    let athena_file = result.unwrap();
    let service = &athena_file.services.services[0];
    assert!(!service.ports.is_empty()); // Compose feature
    assert!(service.swarm_config.is_some()); // Swarm feature
    assert!(service.restart.is_some()); // Compose feature
}