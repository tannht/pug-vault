// Re-export main components for testing  
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct VaultData {
    pub secrets: HashMap<String, String>,
}

pub struct Vault {
    pub key: [u8; 32],
    pub data_file: PathBuf,
}

impl Vault {
    pub fn derive_key(password: &str) -> Result<[u8; 32]> {
        let salt = SaltString::from_b64("cHVnLXNhbHQtdjE")
            .map_err(|_| anyhow!("Salt derivation error"))?;
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("KDF error: {}", e))?;

        let mut key = [0u8; 32];
        let hash_bytes = hash.hash.ok_or_else(|| anyhow!("Hash derivation failed"))?;
        key.copy_from_slice(&hash_bytes.as_bytes()[..32]);
        Ok(key)
    }

    pub fn new(password: &str) -> Result<Self> {
        let key = Self::derive_key(password)?;
        let mut data_file =
            dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
        data_file.push(".pug_vault_rust_data");

        Ok(Self { key, data_file })
    }

    pub fn read_data(&self) -> Result<VaultData> {
        if !self.data_file.exists() {
            return Ok(VaultData {
                secrets: HashMap::new(),
            });
        }

        let encrypted_hex = fs::read_to_string(&self.data_file)?;
        let parts: Vec<&str> = encrypted_hex.split(':').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Corrupted vault file structure."));
        }

        let iv = hex::decode(parts[0])?;
        let tag = hex::decode(parts[1])?;
        let ciphertext = hex::decode(parts[2])?;

        let cipher = Aes256Gcm::new(&self.key.into());
        let nonce = Nonce::from_slice(&iv);

        let mut combined = ciphertext;
        combined.extend_from_slice(&tag);

        let decrypted = cipher
            .decrypt(nonce, combined.as_slice())
            .map_err(|_| anyhow!("Invalid Master Password or corrupted data."))?;

        Ok(serde_json::from_slice(&decrypted)?)
    }

    pub fn write_data_with_key(data_file: &PathBuf, key: &[u8; 32], data: &VaultData) -> Result<()> {
        let mut iv = [0u8; 12];
        OsRng.fill_bytes(&mut iv);

        let cipher = Aes256Gcm::new(key.into());
        let nonce = Nonce::from_slice(&iv);

        let plaintext = serde_json::to_vec(data)?;
        let ciphertext_with_tag = cipher
            .encrypt(nonce, plaintext.as_slice())
            .map_err(|e| anyhow!("Encryption error: {}", e))?;

        let tag_pos = ciphertext_with_tag.len() - 16;
        let ciphertext = &ciphertext_with_tag[..tag_pos];
        let tag = &ciphertext_with_tag[tag_pos..];

        let output = format!(
            "{}:{}:{}",
            hex::encode(iv),
            hex::encode(tag),
            hex::encode(ciphertext)
        );

        let mut file = File::create(data_file)?;
        fs::set_permissions(data_file, fs::Permissions::from_mode(0o600))?;
        file.write_all(output.as_bytes())?;

        Ok(())
    }

    pub fn write_data(&self, data: &VaultData) -> Result<()> {
        Self::write_data_with_key(&self.data_file, &self.key, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs;

    // Helper function to create a temporary vault for testing
    fn create_test_vault(password: &str) -> Vault {
        let key = Vault::derive_key(password).unwrap();
        let mut data_file = temp_dir();
        data_file.push(format!("test_vault_{}.data", rand::random::<u32>()));
        
        Vault { key, data_file }
    }

    #[test]
    fn test_key_derivation() {
        let password = "test_password_123";
        let key1 = Vault::derive_key(password).unwrap();
        let key2 = Vault::derive_key(password).unwrap();
        
        // Same password should always generate same key
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32);
        
        // Different password should generate different key
        let key3 = Vault::derive_key("different_password").unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_vault_creation() {
        let password = "test_password_123";
        let vault = create_test_vault(password);
        
        assert_eq!(vault.key.len(), 32);
        assert!(vault.data_file.to_str().unwrap().contains("test_vault_"));
    }

    #[test]
    fn test_read_empty_vault() {
        let vault = create_test_vault("test_password");
        let data = vault.read_data().unwrap();
        
        assert!(data.secrets.is_empty());
    }

    #[test] 
    fn test_write_and_read_data() {
        let vault = create_test_vault("test_password");
        
        // Create test data
        let mut secrets = HashMap::new();
        secrets.insert("api_key".to_string(), "secret_value_123".to_string());
        secrets.insert("db_password".to_string(), "super_secure_pass".to_string());
        
        let test_data = VaultData { secrets: secrets.clone() };
        
        // Write data
        vault.write_data(&test_data).unwrap();
        assert!(vault.data_file.exists());
        
        // Read data back
        let read_data = vault.read_data().unwrap();
        assert_eq!(read_data.secrets, secrets);
        
        // Cleanup
        fs::remove_file(&vault.data_file).ok();
    }

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        let vault = create_test_vault("encryption_test_password");
        
        let original_data = VaultData {
            secrets: {
                let mut map = HashMap::new();
                map.insert("test_key".to_string(), "test_value".to_string());
                map.insert("unicode_test".to_string(), "🐶 gâu gâu! 测试".to_string());
                map
            }
        };
        
        // Encrypt and write
        vault.write_data(&original_data).unwrap();
        
        // Read and decrypt  
        let decrypted_data = vault.read_data().unwrap();
        
        assert_eq!(original_data.secrets, decrypted_data.secrets);
        
        // Cleanup
        fs::remove_file(&vault.data_file).ok();
    }

    #[test]
    fn test_wrong_password_fails() {
        let vault1 = create_test_vault("correct_password");
        let vault2 = create_test_vault("wrong_password");
        vault2.data_file = vault1.data_file.clone();
        
        // Write with first vault (correct password)
        let test_data = VaultData {
            secrets: {
                let mut map = HashMap::new();
                map.insert("secret".to_string(), "value".to_string());
                map
            }
        };
        vault1.write_data(&test_data).unwrap();
        
        // Try to read with second vault (wrong password)
        let result = vault2.read_data();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid Master Password"));
        
        // Cleanup
        fs::remove_file(&vault1.data_file).ok();
    }

    #[test]
    fn test_write_data_with_key() {
        let test_key = [42u8; 32]; // Test key
        
        let mut data_file = temp_dir();
        data_file.push(format!("test_write_key_{}.data", rand::random::<u32>()));
        
        let test_data = VaultData {
            secrets: {
                let mut map = HashMap::new();
                map.insert("test".to_string(), "value".to_string());
                map
            }
        };
        
        // Test writing with specific key
        Vault::write_data_with_key(&data_file, &test_key, &test_data).unwrap();
        assert!(data_file.exists());
        
        // Verify file permissions are secure (600)
        let metadata = fs::metadata(&data_file).unwrap();
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode() & 0o777, 0o600);
        
        // Cleanup
        fs::remove_file(&data_file).ok();
    }

    #[test]
    fn test_corrupted_vault_file() {
        let vault = create_test_vault("test_password");
        
        // Write invalid data to file  
        fs::write(&vault.data_file, "invalid:data").unwrap();
        
        let result = vault.read_data();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Corrupted vault file"));
        
        // Cleanup
        fs::remove_file(&vault.data_file).ok();
    }

    #[test]
    fn test_invalid_hex_in_vault_file() {
        let vault = create_test_vault("test_password");
        
        // Write invalid hex data
        fs::write(&vault.data_file, "invalid_hex:more_invalid:even_more").unwrap();
        
        let result = vault.read_data();
        assert!(result.is_err());
        
        // Cleanup
        fs::remove_file(&vault.data_file).ok();
    }

    #[test]
    fn test_vault_data_serialization() {
        let mut secrets = HashMap::new();
        secrets.insert("key1".to_string(), "value1".to_string()); 
        secrets.insert("key2".to_string(), "value2".to_string());
        
        let vault_data = VaultData { secrets };
        
        // Test serialization
        let json = serde_json::to_string(&vault_data).unwrap();
        assert!(json.contains("key1"));
        assert!(json.contains("value1"));
        
        // Test deserialization 
        let deserialized: VaultData = serde_json::from_str(&json).unwrap();
        assert_eq!(vault_data.secrets, deserialized.secrets);
    }

    #[test]
    fn test_empty_password_key_derivation() {
        let empty_key = Vault::derive_key("").unwrap();
        let non_empty_key = Vault::derive_key("password").unwrap();
        
        assert_ne!(empty_key, non_empty_key);
        assert_eq!(empty_key.len(), 32);
    }

    #[test]
    fn test_large_secret_storage() {
        let vault = create_test_vault("large_data_test");
        
        let large_value = "x".repeat(10000); // 10KB value
        let mut secrets = HashMap::new();
        secrets.insert("large_secret".to_string(), large_value.clone());
        
        let test_data = VaultData { secrets };
        
        vault.write_data(&test_data).unwrap();
        let read_data = vault.read_data().unwrap();
        
        assert_eq!(read_data.secrets.get("large_secret").unwrap(), &large_value);
        
        // Cleanup
        fs::remove_file(&vault.data_file).ok();
    }

    #[test]
    fn test_special_characters_in_secrets() {
        let vault = create_test_vault("special_chars_test");
        
        let mut secrets = HashMap::new();
        secrets.insert("emoji_key".to_string(), "🐶🔐💎".to_string());
        secrets.insert("unicode_key".to_string(), "测试中文العربية🌍".to_string());
        secrets.insert("json_like".to_string(), r#"{"nested": "value", "array": [1,2,3]}"#.to_string());
        
        let test_data = VaultData { secrets: secrets.clone() };
        
        vault.write_data(&test_data).unwrap();
        let read_data = vault.read_data().unwrap();
        
        assert_eq!(read_data.secrets, secrets);
        
        // Cleanup  
        fs::remove_file(&vault.data_file).ok();
    }
}