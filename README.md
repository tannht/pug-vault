# 🐶 PugVault 🦀

Ultra-fast, secure, locally encrypted secret store for AI agents. Built with Rust.

## 🚀 Quick Start
```bash
# Install (requires Rust)
curl -sSf https://raw.githubusercontent.com/tannht/pug-vault/main/install.sh | sh
```

## 🛠 Usage
```bash
export PUG_MASTER_PASSWORD=your-secure-key

pug-vault set <key> <value>   # Store
pug-vault get <key>           # Retrieve
pug-vault list                # List all
pug-vault delete <key>        # Remove
```

## 🔒 Security
- **Engine:** Argon2 + AES-256-GCM.
- **Local-only:** Data stored at `~/.pug_vault_rust_data` (chmod 600).

---
Created by [Hoàng Tân](https://github.com/tannht) & **PubPug** 🐶.
