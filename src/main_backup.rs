mod lib;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use clap::{Parser, Subcommand};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

#[derive(Parser)]
#[command(name = "pug-vault")]
#[command(about = "🐶 PugVault - Secure, simple, and locally encrypted secret store for AI agents.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Store a secret
    Set { key: String, value: String },
    /// Retrieve a secret
    Get { key: String },
    /// List all secret keys
    List,
    /// Remove a secret
    Delete { key: String },
    /// Change the Master Password
    ChangePassword,
}

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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let password = std::env::var("PUG_MASTER_PASSWORD")
        .map_err(|_| anyhow!("PUG_MASTER_PASSWORD environment variable not set."))?;

    let vault = Vault::new(&password)?;

    match cli.command {
        Commands::Set { key, value } => {
            let mut data = vault.read_data()?;
            data.secrets.insert(key.clone(), value);
            vault.write_data(&data)?;
            println!("✅ Secret '{}' stored successfully.", key);
        }
        Commands::Get { key } => {
            let data = vault.read_data()?;
            match data.secrets.get(&key) {
                Some(val) => println!("{}", val),
                None => return Err(anyhow!("Secret '{}' not found.", key)),
            }
        }
        Commands::List => {
            let data = vault.read_data()?;
            if data.secrets.is_empty() {
                println!("Vault is empty! 🦴");
            } else {
                for k in data.secrets.keys() {
                    println!("- {}", k);
                }
            }
        }
        Commands::Delete { key } => {
            let mut data = vault.read_data()?;
            if data.secrets.remove(&key).is_some() {
                vault.write_data(&data)?;
                println!("🗑️ Secret '{}' deleted.", key);
            } else {
                return Err(anyhow!("Secret '{}' not found.", key));
            }
        }
        Commands::ChangePassword => {
            let data = vault.read_data()?; // Verify old password first

            println!("🐶 Changing Master Password...");
            print!("🔑 Enter NEW Master Password: ");
            io::stdout().flush()?;
            let mut new_pass = String::new();
            io::stdin().read_line(&mut new_pass)?;
            let new_pass = new_pass.trim();

            print!("🔄 Re-enter NEW Master Password to confirm: ");
            io::stdout().flush()?;
            let mut confirm_pass = String::new();
            io::stdin().read_line(&mut confirm_pass)?;
            let confirm_pass = confirm_pass.trim();

            if new_pass != confirm_pass {
                return Err(anyhow!("❌ Error: Confirmation password does not match!"));
            }

            if new_pass.is_empty() {
                return Err(anyhow!("❌ Error: New password cannot be empty!"));
            }

            let new_key = Vault::derive_key(new_pass)?;
            Vault::write_data_with_key(&vault.data_file, &new_key, &data)?;

            println!("🎉 Success! Master Password has been changed. Gâu gâu! 🐶");
        }
    }

    Ok(())
}

// Built with love by PubPug 🐶
