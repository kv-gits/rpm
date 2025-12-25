use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber;
use tokio::sync::watch;

mod config;
mod crypto;
mod errors;
mod i18n;
mod models;
mod server;
mod storage;
mod tui;
mod tray;

use config::Config;

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

    // Initialize cryptography module
    let crypto = crypto::CryptoManager::new()?;
    info!("Cryptography module initialized");

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = watch::channel(());

    // Start system tray
    let tray_manager = tray::TrayManager::new()?;
    let tray_handle = tray_manager.handle.clone();
    info!("System tray initialized");

    // Start HTTP server for browser extensions
    let server_handle = {
        let crypto_clone = crypto.clone();
        let shutdown_rx = shutdown_rx.clone();
        tokio::spawn(async move {
            if let Err(e) = server::start_server(config.server_port, crypto_clone, shutdown_rx).await {
                error!("Server error: {}", e);
            }
        })
    };
    info!("HTTP server started on port {}", config.server_port);

    // Start TUI with shutdown sender
    info!("Starting TUI...");
    let shutdown_tx_for_tui = shutdown_tx.clone();
    let tui_handle = tokio::spawn(async move {
        if let Err(e) = tui::run_tui(crypto, tray_handle, config, shutdown_tx_for_tui).await {
            error!("TUI error: {}", e);
        }
    });

    // Wait for TUI to finish
    let _ = tui_handle.await;

    // Send shutdown signal to all components
    info!("Shutting down...");
    let _ = shutdown_tx.send(());

    // Wait for server to finish gracefully
    let _ = server_handle.await;

    info!("RPM shutdown complete");
    Ok(())
}

