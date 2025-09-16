//! Utilities for boilerplate generation

use crate::athena::AthenaError;
use std::fs;
use std::path::Path;

pub type UtilResult<T> = Result<T, AthenaError>;

/// Create directory structure recursively
pub fn create_directory_structure(base_path: &Path, dirs: &[&str]) -> UtilResult<()> {
    for dir in dirs {
        let full_path = base_path.join(dir);
        fs::create_dir_all(&full_path)
            .map_err(AthenaError::IoError)?;
    }
    Ok(())
}

/// Write file with content, creating parent directories if needed
pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> UtilResult<()> {
    let path = path.as_ref();
    
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(AthenaError::IoError)?;
    }
    
    fs::write(path, content)
        .map_err(AthenaError::IoError)?;
    
    Ok(())
}

/// Replace template variables in content
#[allow(dead_code)]
pub fn replace_template_vars(content: &str, vars: &[(&str, &str)]) -> String {
    let mut result = content.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

/// Replace template variables in content with String values
pub fn replace_template_vars_string(content: &str, vars: &[(&str, String)]) -> String {
    let mut result = content.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

/// Generate secure random string for secrets
pub fn generate_secret_key() -> String {
    use uuid::Uuid;
    format!("{}{}", Uuid::new_v4().to_string().replace("-", ""), Uuid::new_v4().to_string().replace("-", ""))
}

/// Convert project name to different cases
pub struct ProjectNames {
    #[allow(dead_code)]
    pub original: String,
    pub snake_case: String,
    pub kebab_case: String,
    pub pascal_case: String,
    pub upper_case: String,
}

impl ProjectNames {
    pub fn new(name: &str) -> Self {
        let snake_case = name.replace("-", "_").to_lowercase();
        let kebab_case = name.replace("_", "-").to_lowercase();
        let pascal_case = name
            .split(['_', '-'])
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<String>();
        let upper_case = snake_case.to_uppercase();
        
        Self {
            original: name.to_string(),
            snake_case,
            kebab_case,
            pascal_case,
            upper_case,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_create_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        let dirs = vec!["src/models", "src/routes", "tests"];
        create_directory_structure(base_path, &dirs).unwrap();
        
        assert!(base_path.join("src/models").exists());
        assert!(base_path.join("src/routes").exists());
        assert!(base_path.join("tests").exists());
    }
    
    #[test]
    fn test_replace_template_vars() {
        let template = "Hello {{name}}, your project is {{project_name}}!";
        let vars = vec![("name", "Alice"), ("project_name", "MyApp")];
        let result = replace_template_vars(template, &vars);
        assert_eq!(result, "Hello Alice, your project is MyApp!");
    }
    
    #[test]
    fn test_project_names() {
        let names = ProjectNames::new("my-awesome_project");
        assert_eq!(names.snake_case, "my_awesome_project");
        assert_eq!(names.kebab_case, "my-awesome-project");
        assert_eq!(names.pascal_case, "MyAwesomeProject");
        assert_eq!(names.upper_case, "MY_AWESOME_PROJECT");
    }
}