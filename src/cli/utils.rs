use std::fs;
use std::path::PathBuf;

use crate::athena::{AthenaResult, AthenaError};

/// Auto-detect a .ath file in the current directory
pub fn auto_detect_ath_file(input: Option<PathBuf>) -> AthenaResult<PathBuf> {
    match input {
        Some(path) => Ok(path),
        None => {
            // Search for .ath files in the current directory
            let ath_files: Vec<_> = fs::read_dir(".")
                .map_err(AthenaError::IoError)?
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    if path.extension()? == "ath" && path.is_file() {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect();

            match ath_files.len() {
                0 => Err(AthenaError::ConfigError(
                    "No .ath file found in current directory. Please specify a file or create one.".to_string()
                )),
                1 => {
                    let file = &ath_files[0];
                    println!("Auto-detected: {}", file.display());
                    Ok(file.clone())
                },
                _ => {
                    let mut error_msg = "Multiple .ath files found. Please specify which one to use:\n".to_string();
                    for file in &ath_files {
                        error_msg.push_str(&format!("  - {}\n", file.display()));
                    }
                    error_msg.push_str("\nUsage: athena build <FILE>");
                    Err(AthenaError::ConfigError(error_msg))
                }
            }
        }
    }
}

/// Determine if we should be in verbose mode (default yes, unless --quiet)
pub fn should_be_verbose(_global_verbose: bool, quiet: bool) -> bool {
    if quiet {
        false
    } else {
        true // Verbose by default
    }
}