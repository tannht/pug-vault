use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use pug_vault::Vault;
use std::io::{self, Write};
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_OWNER: &str = "tannht";
const REPO_NAME: &str = "pug-vault";

#[derive(Parser)]
#[command(name = "pug-vault")]
#[command(version = VERSION)]
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
    /// Update pug-vault to the latest version
    Update {
        /// Force reinstall even if already on the latest version
        #[arg(long, short)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Commands that do NOT require the master password
    if let Commands::Update { force } = &cli.command {
        return handle_update(*force);
    }

    // All other commands require the master password
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
        Commands::Update { .. } => unreachable!(),
    }

    Ok(())
}

/// Fetch the latest release tag from GitHub API using a simple HTTPS request.
fn fetch_latest_version() -> Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        REPO_OWNER, REPO_NAME
    );

    // Try using curl (available on all platforms including Windows 10+)
    let output = Command::new("curl")
        .args([
            "-s",
            "-L",
            "-H",
            "Accept: application/vnd.github.v3+json",
            &url,
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let body = String::from_utf8(out.stdout)
                .map_err(|_| anyhow!("Failed to parse GitHub API response"))?;

            // Parse the "tag_name" field from JSON
            let json: serde_json::Value = serde_json::from_str(&body)
                .map_err(|_| anyhow!("Failed to parse GitHub API JSON response"))?;

            let tag = json["tag_name"].as_str().ok_or_else(|| {
                anyhow!("No releases found on GitHub. You may already be on the latest version.")
            })?;

            // Strip leading 'v' if present (e.g., "v1.2.3" -> "1.2.3")
            let version = tag.strip_prefix('v').unwrap_or(tag);
            Ok(version.to_string())
        }
        _ => Err(anyhow!(
            "Could not check for updates. Please check your internet connection."
        )),
    }
}

/// Handle the `update` subcommand.
fn handle_update(force: bool) -> Result<()> {
    println!("🐶 PugVault Updater");
    println!("   Current version: v{}", VERSION);
    println!();

    // Step 1: Check latest version from GitHub
    print!("🔍 Checking for updates...");
    io::stdout().flush()?;

    let latest_version = match fetch_latest_version() {
        Ok(v) => {
            println!(" found v{}", v);
            v
        }
        Err(e) => {
            println!(" skipped");
            eprintln!("   ⚠️  {}", e);
            if !force {
                eprintln!("   Use --force to install from source anyway.");
                return Ok(());
            }
            String::new()
        }
    };

    // Step 2: Compare versions
    if !force && !latest_version.is_empty() && latest_version == VERSION {
        println!();
        println!(
            "✅ Already on the latest version (v{}). Gâu gâu! 🐶",
            VERSION
        );
        return Ok(());
    }

    if !latest_version.is_empty() && latest_version != VERSION {
        println!(
            "   📦 New version available: v{} → v{}",
            VERSION, latest_version
        );
    }

    println!();

    // Step 3: Check if cargo is available
    let cargo_available = Command::new("cargo")
        .args(["--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !cargo_available {
        return Err(anyhow!(
            "❌ 'cargo' not found. Please install Rust toolchain first:\n   https://rustup.rs/"
        ));
    }

    // Step 4: Install latest version from GitHub
    println!("📥 Installing latest version from GitHub...");
    println!(
        "   cargo install --git https://github.com/{}/{}.git --force",
        REPO_OWNER, REPO_NAME
    );
    println!();

    let status = Command::new("cargo")
        .args([
            "install",
            "--git",
            &format!("https://github.com/{}/{}.git", REPO_OWNER, REPO_NAME),
            "--force",
        ])
        .status()
        .map_err(|e| anyhow!("Failed to run cargo install: {}", e))?;

    if status.success() {
        println!();
        println!("🎉 PugVault has been updated successfully! Gâu gâu! 🐶");
        println!("   Run 'pug-vault --version' to verify.");
    } else {
        return Err(anyhow!(
            "❌ Update failed. Please try manually:\n   cargo install --git https://github.com/{}/{}.git --force",
            REPO_OWNER,
            REPO_NAME
        ));
    }

    Ok(())
}

// Built with love by PubPug 🐶
