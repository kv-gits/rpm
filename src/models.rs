use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct PasswordEntry {
    pub id: Uuid,
    pub title: String,
    pub username: Option<String>,
    #[serde(skip_serializing)]
    pub password: EncryptedPassword,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EncryptedPassword {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntryDto {
    pub id: Uuid,
    pub title: String,
    pub username: Option<String>,
    pub password: String, // Base64 encoded encrypted password
    pub nonce: String,    // Base64 encoded nonce
    pub url: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePasswordRequest {
    pub title: String,
    pub username: Option<String>,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePasswordRequest {
    pub title: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub master_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefFileEntry {
    pub encrypted_filename: String, // UUID filename
    pub encrypted_name: String,      // Base64 encoded encrypted name
    pub nonce: String,               // Base64 encoded nonce
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefFile {
    pub entries: Vec<DefFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordFile {
    pub encrypted_password: String, // Base64 encoded encrypted password
    pub nonce: String,              // Base64 encoded nonce
}

