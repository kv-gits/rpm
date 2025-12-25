use crate::config::Config;
use crate::crypto::CryptoManager;
use crate::errors::{RpmError, RpmResult};
use crate::models::{DefFile, DefFileEntry, PasswordFile};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use std::path::PathBuf;
use uuid::Uuid;

pub struct PasswordStorage {
    passwords_dir: PathBuf,
    crypto: CryptoManager,
}

impl PasswordStorage {
    pub fn new(config: &Config, crypto: CryptoManager) -> Self {
        Self {
            passwords_dir: config.passwords_directory_path(),
            crypto,
        }
    }

    /// Get the path to the def file
    fn def_file_path(&self) -> PathBuf {
        self.passwords_dir.join("def")
    }

    /// Get the path to a password file by UUID
    pub fn password_file_path(&self, filename: &str) -> PathBuf {
        self.passwords_dir.join(filename)
    }

    /// Ensure passwords directory exists
    fn ensure_passwords_dir(&self) -> RpmResult<()> {
        std::fs::create_dir_all(&self.passwords_dir)
            .map_err(|e| RpmError::Io(e))?;
        Ok(())
    }

    /// Load and decrypt the def file
    pub fn load_def_file(&self, key: &[u8]) -> RpmResult<DefFile> {
        let def_path = self.def_file_path();
        
        if !def_path.exists() {
            // Return empty def file if it doesn't exist
            return Ok(DefFile { entries: Vec::new() });
        }

        let encrypted_content = std::fs::read(&def_path)
            .map_err(|e| RpmError::Io(e))?;

        // Decrypt the def file
        // The def file itself is encrypted, so we need to handle it
        // For now, we'll store it as JSON encrypted with the key
        // Format: first 12 bytes are nonce, rest is ciphertext
        if encrypted_content.len() < 12 {
            return Err(RpmError::Crypto("Invalid def file format".to_string()));
        }

        let nonce = &encrypted_content[0..12];
        let ciphertext = &encrypted_content[12..];

        let plaintext = self.crypto.decrypt_data(ciphertext, nonce, key)?;
        let json_str = String::from_utf8(plaintext)
            .map_err(|e| RpmError::Crypto(format!("Invalid UTF-8 in def file: {}", e)))?;

        let def_file: DefFile = serde_json::from_str(&json_str)
            .map_err(|e| RpmError::Serialization(e.into()))?;

        Ok(def_file)
    }

    /// Save the def file encrypted
    pub fn save_def_file(&self, def_file: &DefFile, key: &[u8]) -> RpmResult<()> {
        self.ensure_passwords_dir()?;

        let json_str = serde_json::to_string(def_file)
            .map_err(|e| RpmError::Serialization(e.into()))?;

        let (ciphertext, nonce) = self.crypto.encrypt_data(json_str.as_bytes(), key)?;

        // Write nonce (12 bytes) + ciphertext
        let mut encrypted_content = nonce;
        encrypted_content.extend_from_slice(&ciphertext);

        std::fs::write(self.def_file_path(), encrypted_content)
            .map_err(|e| RpmError::Io(e))?;

        Ok(())
    }

    /// Encrypt a filename (name) and return encrypted data with nonce
    pub fn encrypt_filename(&self, name: &str, key: &[u8]) -> RpmResult<(String, String)> {
        let (ciphertext, nonce) = self.crypto.encrypt_data(name.as_bytes(), key)?;
        Ok((
            BASE64_STANDARD.encode(&ciphertext),
            BASE64_STANDARD.encode(&nonce),
        ))
    }

    /// Decrypt a filename
    pub fn decrypt_filename(&self, encrypted_name: &str, nonce: &str, key: &[u8]) -> RpmResult<String> {
        let ciphertext = BASE64_STANDARD.decode(encrypted_name)
            .map_err(|e| RpmError::Crypto(format!("Invalid base64 in encrypted name: {}", e)))?;
        let nonce_bytes = BASE64_STANDARD.decode(nonce)
            .map_err(|e| RpmError::Crypto(format!("Invalid base64 in nonce: {}", e)))?;

        let plaintext = self.crypto.decrypt_data(&ciphertext, &nonce_bytes, key)?;
        String::from_utf8(plaintext)
            .map_err(|e| RpmError::Crypto(format!("Invalid UTF-8 in decrypted name: {}", e)))
    }

    /// Save a password to a file
    pub fn save_password_file(&self, password: &str, key: &[u8]) -> RpmResult<String> {
        self.ensure_passwords_dir()?;

        let (ciphertext, nonce) = self.crypto.encrypt_password(password, key)?;

        let password_file = PasswordFile {
            encrypted_password: BASE64_STANDARD.encode(&ciphertext),
            nonce: BASE64_STANDARD.encode(&nonce),
        };

        // Generate UUID for filename
        let filename = format!("{}.pwd", Uuid::new_v4());
        let file_path = self.password_file_path(&filename);

        let json_str = serde_json::to_string(&password_file)
            .map_err(|e| RpmError::Serialization(e.into()))?;

        std::fs::write(file_path, json_str)
            .map_err(|e| RpmError::Io(e))?;

        Ok(filename)
    }

    /// Load and decrypt a password from a file
    pub fn load_password_file(&self, filename: &str, key: &[u8]) -> RpmResult<String> {
        let file_path = self.password_file_path(filename);

        let json_str = std::fs::read_to_string(&file_path)
            .map_err(|e| RpmError::Io(e))?;

        let password_file: PasswordFile = serde_json::from_str(&json_str)
            .map_err(|e| RpmError::Serialization(e.into()))?;

        let ciphertext = BASE64_STANDARD.decode(&password_file.encrypted_password)
            .map_err(|e| RpmError::Crypto(format!("Invalid base64 in encrypted password: {}", e)))?;
        let nonce = BASE64_STANDARD.decode(&password_file.nonce)
            .map_err(|e| RpmError::Crypto(format!("Invalid base64 in nonce: {}", e)))?;

        self.crypto.decrypt_password(&ciphertext, &nonce, key)
    }

    /// Update password in an existing file
    pub fn update_password_file(&self, filename: &str, password: &str, key: &[u8]) -> RpmResult<()> {
        self.ensure_passwords_dir()?;

        let (ciphertext, nonce) = self.crypto.encrypt_password(password, key)?;

        let password_file = PasswordFile {
            encrypted_password: BASE64_STANDARD.encode(&ciphertext),
            nonce: BASE64_STANDARD.encode(&nonce),
        };

        let file_path = self.password_file_path(filename);

        let json_str = serde_json::to_string(&password_file)
            .map_err(|e| RpmError::Serialization(e.into()))?;

        std::fs::write(file_path, json_str)
            .map_err(|e| RpmError::Io(e))?;

        Ok(())
    }

    /// Get list of decrypted names from def file
    pub fn list_decrypted_names(&self, key: &[u8]) -> RpmResult<Vec<(String, String)>> {
        let def_file = self.load_def_file(key)?;
        let mut names = Vec::new();

        for entry in def_file.entries {
            let decrypted_name = self.decrypt_filename(&entry.encrypted_name, &entry.nonce, key)?;
            names.push((entry.encrypted_filename, decrypted_name));
        }

        Ok(names)
    }

    /// Add a new entry to def file
    pub fn add_entry(&self, name: &str, key: &[u8]) -> RpmResult<String> {
        let mut def_file = self.load_def_file(key)?;

        // Encrypt the name
        let (encrypted_name, nonce) = self.encrypt_filename(name, key)?;

        // Generate UUID for filename
        let filename = format!("{}.pwd", Uuid::new_v4());

        let entry = DefFileEntry {
            encrypted_filename: filename.clone(),
            encrypted_name,
            nonce,
        };

        def_file.entries.push(entry);
        self.save_def_file(&def_file, key)?;

        Ok(filename)
    }

    /// Update an entry in def file (by filename)
    pub fn update_entry(&self, filename: &str, new_name: &str, key: &[u8]) -> RpmResult<()> {
        let mut def_file = self.load_def_file(key)?;

        // Find and update the entry
        for entry in &mut def_file.entries {
            if entry.encrypted_filename == filename {
                let (encrypted_name, nonce) = self.encrypt_filename(new_name, key)?;
                entry.encrypted_name = encrypted_name;
                entry.nonce = nonce;
                break;
            }
        }

        self.save_def_file(&def_file, key)?;
        Ok(())
    }

    /// Delete an entry from def file
    pub fn delete_entry(&self, filename: &str, key: &[u8]) -> RpmResult<()> {
        let mut def_file = self.load_def_file(key)?;
        def_file.entries.retain(|e| e.encrypted_filename != filename);
        self.save_def_file(&def_file, key)?;

        // Also delete the password file
        let file_path = self.password_file_path(filename);
        if file_path.exists() {
            std::fs::remove_file(file_path)
                .map_err(|e| RpmError::Io(e))?;
        }

        Ok(())
    }

    /// Find filename by decrypted name
    pub fn find_filename_by_name(&self, name: &str, key: &[u8]) -> RpmResult<Option<String>> {
        let def_file = self.load_def_file(key)?;

        for entry in def_file.entries {
            let decrypted_name = self.decrypt_filename(&entry.encrypted_name, &entry.nonce, key)?;
            if decrypted_name == name {
                return Ok(Some(entry.encrypted_filename));
            }
        }

        Ok(None)
    }
}

