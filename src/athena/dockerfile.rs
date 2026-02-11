use std::collections::HashSet;
use std::fmt::Write;
use std::fs;
use std::path::Path;

use crate::athena::error::{AthenaError, AthenaResult};

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
}

/// Parse Dockerfile and extract ARG declarations.
pub fn analyze_dockerfile<P: AsRef<Path>>(dockerfile_path: P) -> AthenaResult<DockerfileAnalysis> {
    let path_str = dockerfile_path.as_ref().to_string_lossy().to_string();

    if !dockerfile_path.as_ref().exists() {
        return Err(AthenaError::config_error(format!(
            "Dockerfile not found: {path_str}. CREATE-ARGS validation requires a Dockerfile to be present."
        )));
    }

    let content = fs::read_to_string(&dockerfile_path).map_err(|e| {
        AthenaError::config_error(format!("Failed to read Dockerfile '{path_str}': {e}"))
    })?;

    let args = extract_arg_declarations(&content)?;

    Ok(DockerfileAnalysis {
        args,
        dockerfile_path: path_str,
    })
}

/// Extract ARG declarations from Dockerfile content.
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
        // Handle bare "ARG" without a name (Dockerfile syntax error, skip gracefully)
    }

    Ok(args)
}

/// Parse a single ARG line from Dockerfile.
fn parse_arg_line(arg_line: &str, line_number: usize) -> AthenaResult<Option<DockerfileArg>> {
    let arg_line = arg_line.trim();

    if let Some(eq_pos) = arg_line.find('=') {
        let name = arg_line[..eq_pos].trim().to_string();
        let default_value = arg_line[eq_pos + 1..].trim();

        if !is_valid_arg_name(&name) {
            return Err(AthenaError::config_error(format!(
                "Invalid ARG name '{name}' at line {line_number} in Dockerfile"
            )));
        }

        let cleaned_default = strip_quotes(default_value);

        Ok(Some(DockerfileArg {
            name,
            default_value: Some(cleaned_default),
            line_number,
        }))
    } else {
        let name = arg_line.to_string();

        if !is_valid_arg_name(&name) {
            return Err(AthenaError::config_error(format!(
                "Invalid ARG name '{name}' at line {line_number} in Dockerfile"
            )));
        }

        Ok(Some(DockerfileArg {
            name,
            default_value: None,
            line_number,
        }))
    }
}

/// Validate if an ARG name follows Docker conventions.
///
/// Docker ARG names must start with a letter or underscore and contain
/// only alphanumeric characters and underscores.
fn is_valid_arg_name(name: &str) -> bool {
    let mut chars = name.chars();

    match chars.next() {
        None => return false,
        Some(c) if !c.is_alphabetic() && c != '_' => return false,
        _ => {}
    }

    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Strip surrounding quotes (single or double) from a value.
fn strip_quotes(value: &str) -> String {
    let trimmed = value.trim();

    if let Some(inner) = trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return inner.to_string();
    }

    if let Some(inner) = trimmed.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
        return inner.to_string();
    }

    trimmed.to_string()
}

/// Validate BUILD-ARGS against Dockerfile ARGs.
pub fn validate_build_args_against_dockerfile(
    build_args: &std::collections::HashMap<String, String>,
    dockerfile_analysis: &DockerfileAnalysis,
) -> Vec<String> {
    let mut warnings = Vec::new();

    let available_args: HashSet<String> = dockerfile_analysis
        .args
        .iter()
        .map(|arg| arg.name.clone())
        .collect();

    for build_arg_name in build_args.keys() {
        if !available_args.contains(build_arg_name) {
            let suggestions = find_similar_arg_names(build_arg_name, &available_args);

            let mut warning = format!(
                "BUILD-ARG '{build_arg_name}' not found in Dockerfile '{}'",
                dockerfile_analysis.dockerfile_path
            );

            if !available_args.is_empty() {
                let _ = write!(
                    warning,
                    "\nAvailable ARGs in Dockerfile: {}",
                    available_args.iter().cloned().collect::<Vec<_>>().join(", ")
                );
            }

            if !suggestions.is_empty() {
                let _ = write!(warning, "\nDid you mean: {}?", suggestions.join(", "));
            }

            warnings.push(warning);
        }
    }

    warnings
}

/// Find ARG names similar to the given name for suggestion purposes.
fn find_similar_arg_names(target: &str, available: &HashSet<String>) -> Vec<String> {
    let mut similar = Vec::new();
    let target_lower = target.to_lowercase();

    for arg_name in available {
        let arg_lower = arg_name.to_lowercase();

        if arg_lower == target_lower {
            similar.push(arg_name.clone());
            continue;
        }

        if arg_lower.contains(&target_lower) || target_lower.contains(&arg_lower) {
            similar.push(arg_name.clone());
            continue;
        }

        if calculate_similarity(&target_lower, &arg_lower) > 0.6 {
            similar.push(arg_name.clone());
        }
    }

    similar.sort();
    similar.truncate(3);

    similar
}

/// Calculate a simple positional similarity score between two strings.
///
/// Returns a value between 0.0 (no similarity) and 1.0 (identical).
fn calculate_similarity(a: &str, b: &str) -> f32 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let max_len = a.len().max(b.len());

    let common_chars = a.chars().zip(b.chars()).filter(|(ca, cb)| ca == cb).count();

    common_chars as f32 / max_len as f32
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
        };

        let mut build_args = HashMap::new();
        build_args.insert("NODE_VERSION".to_string(), "20".to_string());
        build_args.insert("BUILD_ENV".to_string(), "production".to_string());

        let warnings = validate_build_args_against_dockerfile(&build_args, &analysis);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_validate_build_args_with_warnings() {
        let dockerfile_args = vec![DockerfileArg {
            name: "NODE_VERSION".to_string(),
            default_value: Some("18".to_string()),
            line_number: 1,
        }];

        let analysis = DockerfileAnalysis {
            args: dockerfile_args,
            dockerfile_path: "Dockerfile".to_string(),
        };

        let mut build_args = HashMap::new();
        build_args.insert("NODEJS_VERSION".to_string(), "20".to_string()); // Typo
        build_args.insert("UNKNOWN_ARG".to_string(), "value".to_string());

        let warnings = validate_build_args_against_dockerfile(&build_args, &analysis);
        assert_eq!(warnings.len(), 2);

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
        assert!(!is_valid_arg_name("API-URL")); // Contains hyphen
        assert!(!is_valid_arg_name("API.URL")); // Contains dot
    }

    #[test]
    fn test_strip_quotes() {
        assert_eq!(strip_quotes("\"production\""), "production");
        assert_eq!(strip_quotes("'development'"), "development");
        assert_eq!(strip_quotes("plain"), "plain");
        assert_eq!(strip_quotes("  spaced  "), "spaced");
    }

    #[test]
    fn test_calculate_similarity() {
        assert!(calculate_similarity("", "abc") == 0.0);
        assert!(calculate_similarity("abc", "") == 0.0);
        assert!(calculate_similarity("abc", "abc") == 1.0);
        assert!(calculate_similarity("abc", "axc") > 0.5);
        assert!(calculate_similarity("abc", "xyz") == 0.0);
    }
}