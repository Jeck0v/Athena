use assert_cmd::Command;
use serde_yaml::Value;
use std::fs;
use tempfile::TempDir;

// Common test modules
pub mod basic_structure;
pub mod service_configuration;
pub mod networking;
pub mod policies;
pub mod formatting;
pub mod complex_scenarios;
pub mod comments;

/// Create a test .ath file with given content
pub fn create_test_ath_file(temp_dir: &TempDir, filename: &str, content: &str) -> String {
    let file_path = temp_dir.path().join(filename);
    fs::write(&file_path, content).expect("Failed to create test file");
    file_path.to_string_lossy().to_string()
}

/// Run athena build command and parse the resulting YAML
pub fn run_athena_build_and_parse(ath_file: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_file = temp_dir.path().join("docker-compose.yml").to_string_lossy().to_string();
    
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    let result = cmd.arg("build")
        .arg(ath_file)
        .arg("-o")
        .arg(&output_file)
        .output()
        .expect("Failed to execute command");
    
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!("Command failed: {}", stderr).into());
    }
    
    let yaml_content = fs::read_to_string(&output_file)?;
    let parsed: Value = serde_yaml::from_str(&yaml_content)?;
    Ok(parsed)
}