use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber;

mod config;
mod crypto;
mod db;
mod errors;
mod models;
mod server;
mod tui;
mod tray;

use config::Config;
use errors::RpmError;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting RPM - Rust Password Manager");

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded");

    // Initialize database
    let db = db::Database::new(&config.database_path).await?;
    db.init().await?;
    info!("Database initialized");

    // Initialize cryptography module
    let crypto = crypto::CryptoManager::new()?;
    info!("Cryptography module initialized");

    // Start system tray
    let tray_manager = tray::TrayManager::new()?;
    let tray_handle = tray_manager.handle.clone();
    info!("System tray initialized");

    // Start HTTP server for browser extensions
    let server_handle = {
        let db_clone = db.clone();
        let crypto_clone = crypto.clone();
        tokio::spawn(async move {
            if let Err(e) = server::start_server(config.server_port, db_clone, crypto_clone).await {
                error!("Server error: {}", e);
            }
        })
    };
    info!("HTTP server started on port {}", config.server_port);

    // Start TUI
    info!("Starting TUI...");
    if let Err(e) = tui::run_tui(db, crypto, tray_handle).await {
        error!("TUI error: {}", e);
        return Err(e.into());
    }

    // Wait for server to finish (shouldn't happen unless error)
    let _ = server_handle.await;

    info!("RPM shutdown complete");
    Ok(())
}

