# RPM - Rust Password Manager

A secure password manager built in Rust with modern TUI, system tray, and browser extension support (later).

This project is inspired by [pass](https://www.passwordstore.org/) and rewritten in Rust to be platform-independent, with a security-focused approach to building and extending functionality. Password entries are stored as a collection of files in a specified directory with encrypted filenames.

## ⚠️ Warning

**This project is in early development and has not been tested. As a local password manager, it is generally safe, but caution is still advised when using it for storing passwords or sensitive data.**

## Features

- 🔐 **Strong Encryption**: AES-256-GCM, Argon2id, ChaCha20Poly1305
- 🖥️ **TUI Interface**: User-friendly terminal interface built on ratatui
- 🌍 **Internationalization**: Support for multiple languages
- 🎨 **Themes**: Multiple UI themes
- 📋 **Clipboard Management**: Automatic clipboard clearing after timeout

## Installation

### Requirements

- **Rust**: version 1.70 or higher

### Building from Source

```bash
git clone <repository-url>
cd rpm

# If cargo is not available, initialize bashrc first:
source ~/.bashrc && cargo build

# Release build (recommended)
cargo build --release

# Run the application
cargo run --release
```

## License

AGPL-3.0

## TODO

- [x] Basic project structure
- [x] Cryptography module
- [x] TUI interface (basic)
- [x] HTTP server (basic)
- [x] Settings menu in TUI
- [x] Password storage
- [x] Password generator
- [x] Internationalization
- [x] Multiple themes
- [ ] Full TUI implementation
- [ ] System tray
- [ ] Browser extensions
- [ ] Import/export passwords
- [ ] Synchronization
- [ ] Security audit
- [ ] Testing
