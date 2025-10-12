use std::fs;
use std::path::Path;
use std::collections::HashSet;
use crate::athena::error::{AthenaResult, AthenaError};

#[derive(Debug, Clone, PartialEq)]
pub struct DockerfileArg {
    pub name: String,
    pub default_value: Option<String>,
    pub line_number: usize,
}

#[derive(Debug, Clone)]
pub struct DockerfileAnalysis {
    pub args: Vec<DockerfileArg>,
    pub dockerfile_path: String,
    #[allow(dead_code)]
    pub content: String,
}

/// Parse Dockerfile and extract ARG declarations
pub fn analyze_dockerfile<P: AsRef<Path>>(dockerfile_path: P) -> AthenaResult<DockerfileAnalysis> {
    let path_str = dockerfile_path.as_ref().to_string_lossy().to_string();
    
    // Check if file exists
    if !dockerfile_path.as_ref().exists() {
        return Err(AthenaError::config_error(format!(
            "Dockerfile not found: {}. CREATE-ARGS validation requires a Dockerfile to be present.",
            path_str
        )));
    }

    // Read file content
    let content = fs::read_to_string(&dockerfile_path)
        .map_err(|e| AthenaError::config_error(format!(
            "Failed to read Dockerfile '{}': {}",
            path_str, e
        )))?;

    // Parse ARG declarations
    let args = extract_arg_declarations(&content)?;

    Ok(DockerfileAnalysis {
        args,
        dockerfile_path: path_str,
        content,
    })
}

/// Extract ARG declarations from Dockerfile content
fn extract_arg_declarations(content: &str) -> AthenaResult<Vec<DockerfileArg>> {
    let mut args = Vec::new();
    
    for (line_number, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        
        // Look for ARG instructions
        if let Some(arg_line) = trimmed.strip_prefix("ARG ") {
            if let Some(dockerfile_arg) = parse_arg_line(arg_line, line_number + 1)? {
                args.push(dockerfile_arg);
            }
        }
        // Handle multi-line ARG (less common but possible)
        else if trimmed.eq_ignore_ascii_case("ARG") {
            // This would be a syntax error in Dockerfile, but we can handle it gracefully
            continue;
        }
    }

    Ok(args)
}

/// Parse a single ARG line from Dockerfile
fn parse_arg_line(arg_line: &str, line_number: usize) -> AthenaResult<Option<DockerfileArg>> {
    let arg_line = arg_line.trim();
    
    // Handle ARG NAME=default_value
    if let Some(eq_pos) = arg_line.find('=') {
        let name = arg_line[..eq_pos].trim().to_string();
        let default_value = arg_line[eq_pos + 1..].trim();
        
        // Validate argument name
        if !is_valid_arg_name(&name) {
            return Err(AthenaError::config_error(format!(
                "Invalid ARG name '{}' at line {} in Dockerfile",
                name, line_number
            )));
        }
        
        // Remove quotes if present
        let cleaned_default = clean_dockerfile_value(default_value);
        
        Ok(Some(DockerfileArg {
            name,
            default_value: Some(cleaned_default),
            line_number,
        }))
    }
    // Handle ARG NAME (no default value)
    else {
        let name = arg_line.trim().to_string();
        
        // Validate argument name
        if !is_valid_arg_name(&name) {
            return Err(AthenaError::config_error(format!(
                "Invalid ARG name '{}' at line {} in Dockerfile",
                name, line_number
            )));
        }
        
        Ok(Some(DockerfileArg {
            name,
            default_value: None,
            line_number,
        }))
    }
}

/// Validate if an ARG name follows Docker conventions
fn is_valid_arg_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    
    // Docker ARG names can contain alphanumeric characters and underscores
    // and must start with a letter or underscore
    let chars: Vec<char> = name.chars().collect();
    
    // Check first character
    if !chars[0].is_alphabetic() && chars[0] != '_' {
        return false;
    }
    
    // Check remaining characters
    for &c in &chars[1..] {
        if !c.is_alphanumeric() && c != '_' {
            return false;
        }
    }
    
    true
}

/// Clean default values by removing surrounding quotes
fn clean_dockerfile_value(value: &str) -> String {
    let trimmed = value.trim();
    
    // Remove double quotes
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        return trimmed[1..trimmed.len()-1].to_string();
    }
    
    // Remove single quotes  
    if trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2 {
        return trimmed[1..trimmed.len()-1].to_string();
    }
    
    trimmed.to_string()
}

/// Validate BUILD-ARGS against Dockerfile ARGs
pub fn validate_build_args_against_dockerfile(
    build_args: &std::collections::HashMap<String, String>,
    dockerfile_analysis: &DockerfileAnalysis,
) -> AthenaResult<Vec<String>> {
    let mut warnings = Vec::new();
    
    // Create a set of available ARGs for quick lookup
    let available_args: HashSet<String> = dockerfile_analysis
        .args
        .iter()
        .map(|arg| arg.name.clone())
        .collect();
    
    // Check each BUILD-ARG against Dockerfile
    for build_arg_name in build_args.keys() {
        if !available_args.contains(build_arg_name) {
            // Find similar ARG names for suggestions
            let suggestions = find_similar_arg_names(build_arg_name, &available_args);
            
            let mut warning = format!(
                "BUILD-ARG '{}' not found in Dockerfile '{}'",
                build_arg_name, dockerfile_analysis.dockerfile_path
            );
            
            if !available_args.is_empty() {
                warning.push_str(&format!(
                    "\nAvailable ARGs in Dockerfile: {}",
                    available_args.iter().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
            
            if !suggestions.is_empty() {
                warning.push_str(&format!(
                    "\nDid you mean: {}?",
                    suggestions.join(", ")
                ));
            }
            
            warnings.push(warning);
        }
    }
    
    Ok(warnings)
}

/// Find ARG names similar to the given name (simple Levenshtein-like approach)
fn find_similar_arg_names(target: &str, available: &HashSet<String>) -> Vec<String> {
    let mut similar = Vec::new();
    let target_lower = target.to_lowercase();
    
    for arg_name in available {
        let arg_lower = arg_name.to_lowercase();
        
        // Exact match (case-insensitive)
        if arg_lower == target_lower {
            similar.push(arg_name.clone());
            continue;
        }
        
        // Contains or is contained
        if arg_lower.contains(&target_lower) || target_lower.contains(&arg_lower) {
            similar.push(arg_name.clone());
            continue;
        }
        
        // Simple similarity check (common prefix/suffix)
        if let Some(similarity) = calculate_similarity(&target_lower, &arg_lower) {
            if similarity > 0.6 {
                similar.push(arg_name.clone());
            }
        }
    }
    
    // Sort by similarity (exact matches first, then by name)
    similar.sort();
    similar.truncate(3); // Limit suggestions to top 3
    
    similar
}

/// Calculate simple similarity score between two strings
fn calculate_similarity(a: &str, b: &str) -> Option<f32> {
    if a.is_empty() || b.is_empty() {
        return Some(0.0);
    }
    
    let len_a = a.len();
    let len_b = b.len();
    let max_len = len_a.max(len_b);
    
    // Simple approach: count common characters at same positions
    let common_chars = a.chars()
        .zip(b.chars())
        .filter(|(a, b)| a == b)
        .count();
    
    Some(common_chars as f32 / max_len as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_parse_arg_line_with_default() {
        let result = parse_arg_line("NODE_VERSION=18", 1).unwrap();
        assert!(result.is_some());
        
        let arg = result.unwrap();
        assert_eq!(arg.name, "NODE_VERSION");
        assert_eq!(arg.default_value, Some("18".to_string()));
        assert_eq!(arg.line_number, 1);
    }
    
    #[test]
    fn test_parse_arg_line_without_default() {
        let result = parse_arg_line("BUILD_ENV", 2).unwrap();
        assert!(result.is_some());
        
        let arg = result.unwrap();
        assert_eq!(arg.name, "BUILD_ENV");
        assert_eq!(arg.default_value, None);
        assert_eq!(arg.line_number, 2);
    }
    
    #[test]
    fn test_parse_arg_line_with_quoted_default() {
        let result = parse_arg_line(r#"APP_NAME="my-app""#, 3).unwrap();
        assert!(result.is_some());
        
        let arg = result.unwrap();
        assert_eq!(arg.name, "APP_NAME");
        assert_eq!(arg.default_value, Some("my-app".to_string()));
    }
    
    #[test]
    fn test_extract_arg_declarations() {
        let dockerfile_content = r#"
# This is a comment
FROM node:18

ARG NODE_ENV=production
ARG PORT=3000
ARG API_URL

# Another comment
ARG DEBUG="false"
RUN echo "Building app"
        "#;
        
        let args = extract_arg_declarations(dockerfile_content).unwrap();
        assert_eq!(args.len(), 4);
        
        assert_eq!(args[0].name, "NODE_ENV");
        assert_eq!(args[0].default_value, Some("production".to_string()));
        
        assert_eq!(args[1].name, "PORT");
        assert_eq!(args[1].default_value, Some("3000".to_string()));
        
        assert_eq!(args[2].name, "API_URL");
        assert_eq!(args[2].default_value, None);
        
        assert_eq!(args[3].name, "DEBUG");
        assert_eq!(args[3].default_value, Some("false".to_string()));
    }
    
    #[test]
    fn test_validate_build_args_success() {
        let mut dockerfile_args = Vec::new();
        dockerfile_args.push(DockerfileArg {
            name: "NODE_VERSION".to_string(),
            default_value: Some("18".to_string()),
            line_number: 1,
        });
        dockerfile_args.push(DockerfileArg {
            name: "BUILD_ENV".to_string(),
            default_value: None,
            line_number: 2,
        });
        
        let analysis = DockerfileAnalysis {
            args: dockerfile_args,
            dockerfile_path: "Dockerfile".to_string(),
            content: "".to_string(),
        };
        
        let mut build_args = HashMap::new();
        build_args.insert("NODE_VERSION".to_string(), "20".to_string());
        build_args.insert("BUILD_ENV".to_string(), "production".to_string());
        
        let warnings = validate_build_args_against_dockerfile(&build_args, &analysis).unwrap();
        assert!(warnings.is_empty());
    }
    
    #[test]
    fn test_validate_build_args_with_warnings() {
        let dockerfile_args = vec![
            DockerfileArg {
                name: "NODE_VERSION".to_string(),
                default_value: Some("18".to_string()),
                line_number: 1,
            },
        ];
        
        let analysis = DockerfileAnalysis {
            args: dockerfile_args,
            dockerfile_path: "Dockerfile".to_string(),
            content: "".to_string(),
        };
        
        let mut build_args = HashMap::new();
        build_args.insert("NODEJS_VERSION".to_string(), "20".to_string()); // Typo
        build_args.insert("UNKNOWN_ARG".to_string(), "value".to_string());
        
        let warnings = validate_build_args_against_dockerfile(&build_args, &analysis).unwrap();
        assert_eq!(warnings.len(), 2);
        
        // Check that both invalid args are mentioned in warnings
        let warning_text = warnings.join(" ");
        assert!(warning_text.contains("NODEJS_VERSION"));
        assert!(warning_text.contains("UNKNOWN_ARG"));
    }
    
    #[test]
    fn test_is_valid_arg_name() {
        assert!(is_valid_arg_name("NODE_VERSION"));
        assert!(is_valid_arg_name("API_URL"));
        assert!(is_valid_arg_name("_PRIVATE"));
        assert!(is_valid_arg_name("VERSION123"));
        
        assert!(!is_valid_arg_name(""));
        assert!(!is_valid_arg_name("123VERSION")); // Starts with number
        assert!(!is_valid_arg_name("API-URL"));    // Contains hyphen
        assert!(!is_valid_arg_name("API.URL"));    // Contains dot
    }
    
    #[test]
    fn test_clean_dockerfile_value() {
        assert_eq!(clean_dockerfile_value("\"production\""), "production");
        assert_eq!(clean_dockerfile_value("'development'"), "development");
        assert_eq!(clean_dockerfile_value("plain"), "plain");
        assert_eq!(clean_dockerfile_value("  spaced  "), "spaced");
    }
}