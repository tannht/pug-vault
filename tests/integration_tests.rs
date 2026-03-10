use std::env::{remove_var, set_var};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

// Integration tests for PugVault CLI
#[test]
fn test_cli_without_password_env() {
    let output = Command::new("cargo")
        .args(["run", "--", "list"])
        .env_remove("PUG_MASTER_PASSWORD")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("PUG_MASTER_PASSWORD environment variable not set"));
}

#[test]
fn test_cli_set_and_get_secret() {
    let temp_home = TempDir::new().expect("Failed to create temp dir");
    let test_password = "test_master_password_123";

    // Set environment variables
    set_var("PUG_MASTER_PASSWORD", test_password);
    set_var("HOME", temp_home.path());

    // Test setting a secret
    let output = Command::new("cargo")
        .args(["run", "--", "set", "test_key", "test_value_123"])
        .output()
        .expect("Failed to execute set command");

    if !output.status.success() {
        let stdout_str = String::from_utf8(output.stdout.clone()).unwrap();
        let stderr_str = String::from_utf8(output.stderr.clone()).unwrap();
        eprintln!("STDOUT: {}", stdout_str);
        eprintln!("STDERR: {}", stderr_str);
    }
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Secret 'test_key' stored successfully"));

    // Test getting the secret
    let output = Command::new("cargo")
        .args(["run", "--", "get", "test_key"])
        .output()
        .expect("Failed to execute get command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "test_value_123");

    // Cleanup
    remove_var("PUG_MASTER_PASSWORD");
}

#[test]
fn test_cli_list_secrets() {
    let temp_home = TempDir::new().expect("Failed to create temp dir");
    let test_password = "test_list_password";

    set_var("PUG_MASTER_PASSWORD", test_password);
    set_var("HOME", temp_home.path());

    // Initially should be empty
    let output = Command::new("cargo")
        .args(["run", "--", "list"])
        .output()
        .expect("Failed to execute list command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Vault is empty! 🦴"));

    // Add some secrets
    Command::new("cargo")
        .args(["run", "--", "set", "api_key", "secret123"])
        .output()
        .expect("Failed to set secret");

    Command::new("cargo")
        .args(["run", "--", "set", "db_password", "dbpass456"])
        .output()
        .expect("Failed to set secret");

    // List should show keys
    let output = Command::new("cargo")
        .args(["run", "--", "list"])
        .output()
        .expect("Failed to execute list command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("- api_key"));
    assert!(stdout.contains("- db_password"));

    // Cleanup
    remove_var("PUG_MASTER_PASSWORD");
}

#[test]
fn test_cli_delete_secret() {
    let temp_home = TempDir::new().expect("Failed to create temp dir");
    let test_password = "test_delete_password";

    set_var("PUG_MASTER_PASSWORD", test_password);
    set_var("HOME", temp_home.path());

    // Set a secret first
    Command::new("cargo")
        .args(["run", "--", "set", "delete_me", "value_to_delete"])
        .output()
        .expect("Failed to set secret");

    // Verify it exists
    let output = Command::new("cargo")
        .args(["run", "--", "get", "delete_me"])
        .output()
        .expect("Failed to get secret");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "value_to_delete"
    );

    // Delete it
    let output = Command::new("cargo")
        .args(["run", "--", "delete", "delete_me"])
        .output()
        .expect("Failed to delete secret");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Secret 'delete_me' deleted"));

    // Verify it's gone
    let output = Command::new("cargo")
        .args(["run", "--", "get", "delete_me"])
        .output()
        .expect("Failed to get secret");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Secret 'delete_me' not found"));

    // Cleanup
    remove_var("PUG_MASTER_PASSWORD");
}

#[test]
fn test_cli_get_nonexistent_secret() {
    let temp_home = TempDir::new().expect("Failed to create temp dir");
    let test_password = "test_nonexistent_password";

    set_var("PUG_MASTER_PASSWORD", test_password);
    set_var("HOME", temp_home.path());

    // Try to get non-existent secret
    let output = Command::new("cargo")
        .args(["run", "--", "get", "nonexistent_key"])
        .output()
        .expect("Failed to execute get command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Secret 'nonexistent_key' not found"));

    // Cleanup
    remove_var("PUG_MASTER_PASSWORD");
}

#[test]
fn test_cli_delete_nonexistent_secret() {
    let temp_home = TempDir::new().expect("Failed to create temp dir");
    let test_password = "test_delete_nonexistent_password";

    set_var("PUG_MASTER_PASSWORD", test_password);
    set_var("HOME", temp_home.path());

    // Try to delete non-existent secret
    let output = Command::new("cargo")
        .args(["run", "--", "delete", "nonexistent_key"])
        .output()
        .expect("Failed to execute delete command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Secret 'nonexistent_key' not found"));

    // Cleanup
    remove_var("PUG_MASTER_PASSWORD");
}

#[test]
fn test_wrong_password_cannot_read() {
    let temp_home = TempDir::new().expect("Failed to create temp dir");

    // Set a secret with one password
    set_var("PUG_MASTER_PASSWORD", "password_1");
    set_var("HOME", temp_home.path());

    Command::new("cargo")
        .args(["run", "--", "set", "test", "value"])
        .output()
        .expect("Failed to set secret");

    // Try to read with different password
    set_var("PUG_MASTER_PASSWORD", "password_2");

    let output = Command::new("cargo")
        .args(["run", "--", "get", "test"])
        .output()
        .expect("Failed to execute get command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Invalid Master Password"));

    // Cleanup
    remove_var("PUG_MASTER_PASSWORD");
}

#[test]
fn test_unicode_secrets() {
    let temp_home = TempDir::new().expect("Failed to create temp dir");
    let test_password = "test_unicode_password";

    set_var("PUG_MASTER_PASSWORD", test_password);
    set_var("HOME", temp_home.path());

    // Test with unicode key and value
    let unicode_key = "测试_key_🐶";
    let unicode_value = "🔐 Gâu gâu! 安全密码 🐕";

    let output = Command::new("cargo")
        .args(["run", "--", "set", unicode_key, unicode_value])
        .output()
        .expect("Failed to set unicode secret");

    assert!(output.status.success());

    let output = Command::new("cargo")
        .args(["run", "--", "get", unicode_key])
        .output()
        .expect("Failed to get unicode secret");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), unicode_value);

    // Cleanup
    remove_var("PUG_MASTER_PASSWORD");
}
