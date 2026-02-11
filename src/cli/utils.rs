use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

use crate::athena::{AthenaError, AthenaResult};

/// Auto-detect a .ath file in the current directory.
///
/// If `input` is `Some`, returns it directly. Otherwise, scans the current
/// directory for `.ath` files and returns the single match, or an error if
/// zero or multiple files are found.
pub fn auto_detect_ath_file(input: Option<PathBuf>) -> AthenaResult<PathBuf> {
    if let Some(path) = input {
        return Ok(path);
    }

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
            "No .ath file found in current directory. Please specify a file or create one."
                .to_string(),
        )),
        1 => {
            let file = &ath_files[0];
            println!("Auto-detected: {}", file.display());
            Ok(file.clone())
        }
        _ => {
            let mut error_msg =
                "Multiple .ath files found. Please specify which one to use:\n".to_string();
            for file in &ath_files {
                let _ = writeln!(error_msg, "  - {}", file.display());
            }
            error_msg.push_str("Usage: athena build <FILE>");
            Err(AthenaError::ConfigError(error_msg))
        }
    }
}

/// Determine if we should be in verbose mode (default yes, unless --quiet).
#[must_use]
pub fn should_be_verbose(quiet: bool) -> bool {
    !quiet
}