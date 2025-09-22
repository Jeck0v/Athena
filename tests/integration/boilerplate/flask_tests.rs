use super::*;
use serial_test::serial;
use tempfile::TempDir;

#[test]
#[serial]
fn test_flask_init_basic() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_flask_basic";
    
    let mut cmd = run_init_command("flask", project_name, &[]);
    cmd.current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Flask project"));

    let project_dir = temp_dir.path().join(project_name);
    assert!(project_dir.exists(), "Project directory should be created");

    // Check for Flask-specific files
    let has_flask_app = project_dir.join("app.py").exists() || 
                       project_dir.join("run.py").exists() ||
                       project_dir.join("wsgi.py").exists() ||
                       project_dir.join("app").join("__init__.py").exists();
    assert!(has_flask_app, "Flask application file should exist");
            
    let has_deps = project_dir.join("requirements.txt").exists() ||
                   project_dir.join("Pipfile").exists();
    assert!(has_deps, "Dependencies file should exist");
}

#[test]
#[serial]
fn test_flask_init_with_mysql() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_name = "test_flask_mysql";
    
    let mut cmd = run_init_command("flask", project_name, &["--with-mysql"]);
    cmd.current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Flask project"));

    let project_dir = temp_dir.path().join(project_name);
    assert!(project_dir.exists(), "Project directory should be created");

    // Check for MySQL-specific configuration
    let has_mysql_config = check_for_mysql_configuration(&project_dir);
    assert!(has_mysql_config, "MySQL configuration should be present");
}