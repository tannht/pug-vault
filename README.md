# 🐶 PugVault 🦀

Ultra-fast, secure, locally encrypted secret store for AI agents. Built with Rust.

## 🚀 Quick Start
```bash
# Install (requires Rust/Cargo)
curl -sSf https://raw.githubusercontent.com/tannht/pug-vault/main/install.sh | sh
```

## 🛠 Usage
PugVault uses the `PUG_MASTER_PASSWORD` environment variable to encrypt/decrypt your data.

```bash
export PUG_MASTER_PASSWORD=your-secure-key

pug-vault set <key> <value>   # Store a secret
pug-vault get <key>           # Retrieve a secret
pug-vault list                # List all secret keys
pug-vault delete <key>        # Remove a secret
pug-vault change-password     # Change Master Password (requires old one)
```

## 🔒 Security & FAQ

### 1. Where is the Master Password stored?
**Nowhere.** It lives only in your head (or your password manager). PugVault never saves your password to disk. It is used on-the-fly to derive the encryption key.

### 2. What if I forget my Master Password?
Since PugVault uses industry-standard encryption (**AES-256-GCM**), there is **no recovery**. If you lose your password, your data is permanently inaccessible. To start fresh, delete the data file: `rm ~/.pug_vault_rust_data`.

### 3. Can I have multiple Master Passwords?
**No.** To ensure data integrity, PugVault prevents adding or modifying secrets if the provided password doesn't match the one used to create the vault.

### 4. Inline vs Export
For maximum security, use **inline variables** to avoid leaving the password in your shell history:
`PUG_MASTER_PASSWORD=your_pass pug-vault get my_key`
*(Pro tip: Start your command with a leading space to prevent it from being saved to `~/.bash_history` on most systems).*

## 🐾 About
Created by [Hoàng Tân](https://github.com/tannht) & **PubPug** 🐶.
