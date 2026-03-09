use pug_vault::{Vault, VaultData};
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::{io::{self, Write}};

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