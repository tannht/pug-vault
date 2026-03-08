# 🐶 PugVault - Secure Local Secrets for AI Agents 🦀

**PugVault** is a high-performance CLI tool designed for AI agents to securely store and retrieve sensitive credentials (API keys, tokens, passwords) using industrial-grade local encryption. No cloud, no fees, no leaks.

## 🌟 Key Features
- **🦀 Built with Rust:** Blazing fast performance and memory safety.
- **🛡️ Industrial Encryption:** Uses `AES-256-GCM` for data encryption and `Argon2` for key derivation.
- **🤫 Zero-Cloud:** Your secrets never leave your machine.
- **🤖 Agent-Optimized:** Clean CLI output for easy parsing by AI agents.

## 🚀 Installation
Currently in development. For manual installation:
```bash
git clone https://github.com/tannht/pug-vault.git
cd pug-vault
cargo build --release
sudo cp target/release/pug-vault /usr/local/bin/
```

## 🛠 Usage
PugVault requires the `PUG_MASTER_PASSWORD` environment variable to be set for all operations.

### Store a secret
```bash
PUG_MASTER_PASSWORD=your-secure-pass pug-vault set github_token gho_your_token
```

### Retrieve a secret
```bash
PUG_MASTER_PASSWORD=your-secure-pass pug-vault get github_token
```

### List all keys
```bash
PUG_MASTER_PASSWORD=your-secure-pass pug-vault list
```

## 🔒 Security
Data is stored at `~/.pug_vault_rust_data` with strict `600` permissions.

## 🐾 About
Created by [Hoàng Tân](https://github.com/tannht) and **PubPug** 🐶.
