use std::collections::HashMap;
use std::fs;
use pug_vault::{Vault, VaultData};
use tempfile::TempDir;

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_file_permissions_are_secure() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_vault_in_dir(&temp_dir, "secure_permissions_test");
        
        let test_data = create_test_data();
        vault.write_data(&test_data).unwrap();
        
        // Check file permissions (should be 600 - owner read/write only)
        let metadata = fs::metadata(&vault.data_file).unwrap();
        let permissions = metadata.permissions();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = permissions.mode();
            assert_eq!(mode & 0o777, 0o600, "File should have 600 permissions");
        }
    }

    #[test]
    fn test_encrypted_file_content_not_readable() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_vault_in_dir(&temp_dir, "content_encryption_test");
        
        let secret_value = "super_secret_password_123";
        let mut secrets = HashMap::new();
        secrets.insert("test_key".to_string(), secret_value.to_string());
        let test_data = VaultData { secrets };
        
        vault.write_data(&test_data).unwrap();
        
        // Read file content as raw bytes
        let file_content = fs::read_to_string(&vault.data_file).unwrap();
        
        // The secret value should not appear in plain text
        assert!(!file_content.contains(secret_value));
        assert!(!file_content.contains("test_key"));
        
        // File should contain hex-encoded data (colon-separated)
        assert!(file_content.matches(':').count() == 2);
        
        // All parts should be valid hex
        let parts: Vec<&str> = file_content.split(':').collect();
        assert_eq!(parts.len(), 3);
        
        for part in parts {
            assert!(hex::decode(part).is_ok(), "Part should be valid hex: {}", part);
        }
    }

    #[test]
    fn test_key_derivation_is_deterministic() {
        let password = "deterministic_test_password";
        
        let key1 = Vault::derive_key(password).unwrap();
        let key2 = Vault::derive_key(password).unwrap();
        let key3 = Vault::derive_key(password).unwrap();
        
        assert_eq!(key1, key2);
        assert_eq!(key2, key3);
        assert_eq!(key1, key3);
    }

    #[test]
    fn test_different_passwords_generate_different_keys() {
        let passwords = vec![
            "password1",
            "password2", 
            "completely_different_password",
            "p@ssw0rd!",
            "",  // empty password
            "🐶 unicode password 测试",
        ];
        
        let mut keys = Vec::new();
        
        for password in &passwords {
            let key = Vault::derive_key(password).unwrap();
            keys.push(key);
        }
        
        // All keys should be different
        for i in 0..keys.len() {
            for j in i+1..keys.len() {
                assert_ne!(keys[i], keys[j], 
                    "Keys for '{}' and '{}' should be different", 
                    passwords[i], passwords[j]);
            }
        }
    }

    #[test]
    fn test_encryption_uses_different_nonces() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_vault_in_dir(&temp_dir, "nonce_test");
        
        let test_data = create_test_data();
        
        // Encrypt same data multiple times
        vault.write_data(&test_data).unwrap();
        let content1 = fs::read_to_string(&vault.data_file).unwrap();
        
        vault.write_data(&test_data).unwrap();
        let content2 = fs::read_to_string(&vault.data_file).unwrap();
        
        vault.write_data(&test_data).unwrap();
        let content3 = fs::read_to_string(&vault.data_file).unwrap();
        
        // Even with same data, encrypted output should be different (different nonces)
        assert_ne!(content1, content2);
        assert_ne!(content2, content3);
        assert_ne!(content1, content3);
        
        // But all should decrypt to same data
        let decrypted1 = vault.read_data().unwrap();
        assert_eq!(decrypted1.secrets, test_data.secrets);
    }

    #[test]
    fn test_tampering_detection() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_vault_in_dir(&temp_dir, "tampering_test");
        
        let test_data = create_test_data();
        vault.write_data(&test_data).unwrap();
        
        // Read original content
        let original_content = fs::read_to_string(&vault.data_file).unwrap();
        let mut parts: Vec<&str> = original_content.split(':').collect();
        
        // Tamper with the ciphertext (change one character)
        let mut tampered_ciphertext = parts[2].to_string();
        tampered_ciphertext.replace_range(0..2, "FF");
        parts[2] = &tampered_ciphertext;
        
        let tampered_content = parts.join(":");
        fs::write(&vault.data_file, tampered_content).unwrap();
        
        // Reading should fail due to authentication tag mismatch
        let result = vault.read_data();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid Master Password"));
    }

    #[test]
    fn test_partial_file_corruption() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_vault_in_dir(&temp_dir, "corruption_test");
        
        let test_data = create_test_data();
        vault.write_data(&test_data).unwrap();
        
        // Corrupt file by truncating it
        let original_content = fs::read_to_string(&vault.data_file).unwrap();
        let truncated = &original_content[..original_content.len()/2];
        fs::write(&vault.data_file, truncated).unwrap();
        
        // Reading should fail gracefully
        let result = vault.read_data();
        assert!(result.is_err());
        
        // Error should be about corrupted file or invalid hex
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Corrupted vault file") || 
                error_msg.contains("Odd number of digits") ||
                error_msg.contains("Invalid character"));
    }

    #[test]
    fn test_empty_file_handling() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_vault_in_dir(&temp_dir, "empty_file_test");
        
        // Create empty file
        fs::write(&vault.data_file, "").unwrap();
        
        let result = vault.read_data();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Corrupted vault file"));
    }

    #[test]
    fn test_invalid_format_handling() {
        let temp_dir = TempDir::new().unwrap();
        let vault = create_vault_in_dir(&temp_dir, "invalid_format_test");
        
        // Write file with wrong format (not hex:hex:hex)
        let invalid_formats = vec![
            "not_hex_at_all",
            "onlyonepart",
            "two:parts", 
            "four:parts:are:too:many",
            "valid_hex:but_only:two_parts",
            ":::", // empty parts
        ];
        
        for invalid_format in invalid_formats {
            fs::write(&vault.data_file, invalid_format).unwrap();
            
            let result = vault.read_data();
            assert!(result.is_err(), "Should fail for format: {}", invalid_format);
        }
    }

    #[test]
    fn test_salt_is_consistent() {
        // The salt should be hardcoded and consistent
        let password = "test_salt_consistency";
        
        let key1 = Vault::derive_key(password).unwrap();
        
        // Even across different instances, same password should generate same key
        let key2 = Vault::derive_key(password).unwrap();
        
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_argon2_parameters() {
        // Test that Argon2 is using reasonable default parameters
        let start = std::time::Instant::now();
        let _key = Vault::derive_key("test_password").unwrap();
        let duration = start.elapsed();
        
        // Key derivation should take some time (but not too much)
        assert!(duration.as_millis() > 10);  // At least 10ms (not trivial)
        assert!(duration.as_millis() < 5000); // But less than 5 seconds (practical)
        
        println!("Key derivation time: {:?}", duration);
    }

    // Helper functions
    fn create_vault_in_dir(temp_dir: &TempDir, password: &str) -> Vault {
        let key = Vault::derive_key(password).unwrap();
        let data_file = temp_dir.path().join(format!("test_vault_{}.data", rand::random::<u32>()));
        
        Vault { key, data_file }
    }

    fn create_test_data() -> VaultData {
        let mut secrets = HashMap::new();
        secrets.insert("key1".to_string(), "value1".to_string());
        secrets.insert("key2".to_string(), "value2".to_string());
        VaultData { secrets }
    }
}