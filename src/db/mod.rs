use crate::errors::{RpmError, RpmResult};
use crate::models::{PasswordEntry, PasswordEntryDto};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new<P: AsRef<Path>>(path: P) -> RpmResult<Self> {
        let options = SqliteConnectOptions::from_str(path.as_ref().to_str().unwrap())?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    pub async fn init(&self) -> RpmResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS password_entries (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                username TEXT,
                password_ciphertext BLOB NOT NULL,
                password_nonce BLOB NOT NULL,
                url TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                tags TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_entry(&self, entry: PasswordEntryDto) -> RpmResult<()> {
        let tags_json = serde_json::to_string(&entry.tags)?;
        let password_bytes = BASE64_STANDARD.decode(&entry.password)
            .map_err(|e| RpmError::Crypto(format!("Invalid password encoding: {}", e)))?;
        let nonce_bytes = BASE64_STANDARD.decode(&entry.nonce)
            .map_err(|e| RpmError::Crypto(format!("Invalid nonce encoding: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO password_entries 
            (id, title, username, password_ciphertext, password_nonce, url, notes, created_at, updated_at, tags)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(entry.id.to_string())
        .bind(entry.title)
        .bind(entry.username)
        .bind(password_bytes)
        .bind(nonce_bytes)
        .bind(entry.url)
        .bind(entry.notes)
        .bind(entry.created_at.to_rfc3339())
        .bind(entry.updated_at.to_rfc3339())
        .bind(tags_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_entry(&self, id: Uuid) -> RpmResult<Option<PasswordEntryDto>> {
        let row = sqlx::query_as!(
            PasswordEntryDto,
            r#"
            SELECT 
                id,
                title,
                username,
                password as "password: String",
                nonce as "nonce: String",
                url,
                notes,
                created_at,
                updated_at,
                tags
            FROM password_entries
            WHERE id = ?
            "#,
            id.to_string()
        )
        .fetch_optional(&self.pool)
        .await?;

        // Note: This is a simplified version. In production, you'd need proper mapping
        Ok(None) // Placeholder
    }

    pub async fn list_entries(&self) -> RpmResult<Vec<PasswordEntryDto>> {
        // Placeholder implementation
        Ok(vec![])
    }

    pub async fn update_entry(&self, id: Uuid, entry: PasswordEntryDto) -> RpmResult<()> {
        let tags_json = serde_json::to_string(&entry.tags)?;
        let password_bytes = BASE64_STANDARD.decode(&entry.password)
            .map_err(|e| RpmError::Crypto(format!("Invalid password encoding: {}", e)))?;
        let nonce_bytes = BASE64_STANDARD.decode(&entry.nonce)
            .map_err(|e| RpmError::Crypto(format!("Invalid nonce encoding: {}", e)))?;

        sqlx::query(
            r#"
            UPDATE password_entries
            SET title = ?, username = ?, password_ciphertext = ?, password_nonce = ?,
                url = ?, notes = ?, updated_at = ?, tags = ?
            WHERE id = ?
            "#,
        )
        .bind(entry.title)
        .bind(entry.username)
        .bind(password_bytes)
        .bind(nonce_bytes)
        .bind(entry.url)
        .bind(entry.notes)
        .bind(Utc::now().to_rfc3339())
        .bind(tags_json)
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_entry(&self, id: Uuid) -> RpmResult<()> {
        sqlx::query("DELETE FROM password_entries WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

