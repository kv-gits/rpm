use crate::errors::{RpmError, RpmResult};
use argon2::Argon2;
use argon2::password_hash::{rand_core::OsRng, SaltString};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;

/// Derive a 32-byte encryption key from a password using Argon2id
pub fn derive_key(password: &str, salt: Option<&[u8]>) -> RpmResult<Vec<u8>> {
    // Use Argon2id for key derivation
    let salt_string = if let Some(salt) = salt {
        // Convert bytes to base64 string for SaltString
        let salt_b64 = BASE64_STANDARD.encode(salt);
        SaltString::from_b64(&salt_b64)
            .map_err(|e| RpmError::Crypto(format!("Invalid salt: {}", e)))?
    } else {
        SaltString::generate(&mut OsRng)
    };

    let argon2 = Argon2::default();
    let mut output_key_material = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt_string.as_salt(), &mut output_key_material)
        .map_err(|e| RpmError::Crypto(format!("Key derivation failed: {}", e)))?;

    Ok(output_key_material.to_vec())
}

