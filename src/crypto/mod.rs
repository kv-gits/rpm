use crate::errors::{RpmError, RpmResult};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng as ArgonOsRng, SaltString};
use std::sync::Arc;
use zeroize::{Zeroize, ZeroizeOnDrop};

pub mod key_derivation;

pub use key_derivation::derive_key;

#[derive(Clone)]
pub struct CryptoManager {
    // Using Arc for shared ownership across async tasks
    // Note: In production, consider using secure memory for key storage
}

impl CryptoManager {
    pub fn new() -> RpmResult<Self> {
        Ok(Self {})
    }

    /// Hash a master password using Argon2id
    pub fn hash_password(&self, password: &str) -> RpmResult<String> {
        let salt = SaltString::generate(&mut ArgonOsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| RpmError::Crypto(format!("Password hashing failed: {}", e)))?;
        Ok(password_hash.to_string())
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> RpmResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| RpmError::Crypto(format!("Invalid hash format: {}", e)))?;
        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Encrypt a password using AES-256-GCM
    pub fn encrypt_password(&self, password: &str, key: &[u8]) -> RpmResult<(Vec<u8>, Vec<u8>)> {
        if key.len() != 32 {
            return Err(RpmError::Crypto("Key must be 32 bytes for AES-256".to_string()));
        }

        let cipher_key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(cipher_key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, password.as_bytes())
            .map_err(|e| RpmError::Crypto(format!("Encryption failed: {}", e)))?;

        Ok((ciphertext, nonce.to_vec()))
    }

    /// Decrypt a password using AES-256-GCM
    pub fn decrypt_password(&self, ciphertext: &[u8], nonce: &[u8], key: &[u8]) -> RpmResult<String> {
        if key.len() != 32 {
            return Err(RpmError::Crypto("Key must be 32 bytes for AES-256".to_string()));
        }

        let cipher_key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(cipher_key);
        let nonce = Nonce::from_slice(nonce);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| RpmError::Crypto(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| RpmError::Crypto(format!("Invalid UTF-8 in decrypted data: {}", e)))
    }

    /// Generate a cryptographically secure random token
    pub fn generate_token(&self) -> RpmResult<String> {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        Ok(hex::encode(bytes))
    }
}

#[derive(ZeroizeOnDrop)]
pub struct SecureKey {
    key: Vec<u8>,
}

impl SecureKey {
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.key
    }
}

impl Drop for SecureKey {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

