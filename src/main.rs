use anyhow::{anyhow, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
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
    io::Write,
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
}

#[derive(Serialize, Deserialize)]
struct VaultData {
    secrets: HashMap<String, String>,
}

struct Vault {
    key: [u8; 32],
    data_file: PathBuf,
}

impl Vault {
    fn new(password: &str) -> Result<Self> {
        let salt = SaltString::from_b64("cHVnLXNhbHQtdjE")
            .map_err(|_| anyhow!("Salt error"))?;
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("KDF error: {}", e))?;
        
        let mut key = [0u8; 32];
        let hash_bytes = hash.hash.ok_or_else(|| anyhow!("Hash derivation failed"))?;
        key.copy_from_slice(&hash_bytes.as_bytes()[..32]);

        let mut data_file = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
        data_file.push(".pug_vault_rust_data");

        Ok(Self { key, data_file })
    }

    fn read_data(&self) -> Result<VaultData> {
        if !self.data_file.exists() {
            return Ok(VaultData { secrets: HashMap::new() });
        }

        let encrypted_hex = fs::read_to_string(&self.data_file)?;
        let parts: Vec<&str> = encrypted_hex.split(':').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Corrupted vault file."));
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

    fn write_data(&self, data: &VaultData) -> Result<()> {
        let mut iv = [0u8; 12];
        OsRng.fill_bytes(&mut iv);

        let cipher = Aes256Gcm::new(&self.key.into());
        let nonce = Nonce::from_slice(&iv);
        
        let plaintext = serde_json::to_vec(data)?;
        let ciphertext_with_tag = cipher
            .encrypt(nonce, plaintext.as_slice())
            .map_err(|e| anyhow!("Encryption error: {}", e))?;

        let tag_pos = ciphertext_with_tag.len() - 16;
        let ciphertext = &ciphertext_with_tag[..tag_pos];
        let tag = &ciphertext_with_tag[tag_pos..];

        let output = format!("{}:{}:{}", hex::encode(iv), hex::encode(tag), hex::encode(ciphertext));
        
        let mut file = File::create(&self.data_file)?;
        fs::set_permissions(&self.data_file, fs::Permissions::from_mode(0o600))?;
        file.write_all(output.as_bytes())?;
        
        Ok(())
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
                println!("Hầm trống rỗng! 🦴");
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
    }

    Ok(())
}
