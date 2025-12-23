use crate::crypto::CryptoManager;
use crate::db::Database;
use crate::errors::RpmResult;
use crate::models::{AuthRequest, AuthResponse, CreatePasswordRequest};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{Duration, Utc};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub struct AppState {
    pub db: Database,
    pub crypto: CryptoManager,
}

pub async fn start_server(port: u16, db: Database, crypto: CryptoManager) -> RpmResult<()> {
    let state = Arc::new(AppState { db, crypto });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth", post(authenticate))
        .route("/api/passwords", post(create_password))
        .route("/api/passwords", get(list_passwords))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "rpm-api"
    }))
}

async fn authenticate(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // TODO: Verify master password
    // TODO: Generate JWT token
    let token = state
        .crypto
        .generate_token()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        token,
        expires_at: Utc::now() + Duration::hours(24),
    }))
}

async fn create_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePasswordRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement password creation
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn list_passwords(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement password listing
    Err(StatusCode::NOT_IMPLEMENTED)
}

