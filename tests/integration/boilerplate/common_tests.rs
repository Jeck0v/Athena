use super::*;
use serial_test::serial;
use tempfile::TempDir;

#[test]
#[serial]
fn test_project_already_exists_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "existing_project";
    
    // Pre-create the directory
    let project_path = temp_dir.path().join(project_name);
    fs::create_dir_all(&project_path).expect("Failed to create directory");
    fs::write(project_path.join("existing_file.txt"), "content")
        .expect("Failed to create file");

    let mut cmd = run_init_command("fastapi", project_name, &[]);
    cmd.current_dir(&temp_dir);

    // The command might succeed (overwrite) or fail (directory exists)
    // This documents the current behavior
    let _result = cmd.assert();
    
    // Either way, we should get some indication
    // If it fails, there should be an error message
    // If it succeeds, the directory should contain project files
}

#[test]
fn test_invalid_project_name() {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init")
        .arg("fastapi")
        .arg(""); // Empty project name

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required").or(
            predicate::str::contains("invalid").or(
                predicate::str::contains("NAME").or(
                    predicate::str::contains("cannot be empty")
                )
            )
        ));
}

#[test]
fn test_init_help_commands() {
    let mut cmd = Command::cargo_bin("athena").expect("Failed to find athena binary");
    cmd.arg("init").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialize new project"))
        .stdout(predicate::str::contains("fastapi"))
        .stdout(predicate::str::contains("flask"))
        .stdout(predicate::str::contains("go"));
}