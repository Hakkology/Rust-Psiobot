mod config;
mod discord_bot;
mod models;
mod moltbook;
mod ollama;
mod psiobot;
mod rate_limiter;
mod service;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use dotenv::dotenv;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

use crate::config::Config;
use crate::discord_bot::DiscordService;
use crate::models::RevelationResponse;
use crate::moltbook::MoltbookClient;
use crate::ollama::PsioClient;
use crate::psiobot::Psiobot;
use crate::rate_limiter::RateLimiter;
use crate::service::RevelationService;

#[derive(Clone)]
struct AppState {
    service: Arc<RevelationService>,
    api_key: String,
    manual_limiter: Arc<RateLimiter>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let cfg = Config::from_env().expect("Failed to load configuration from environment");

    let ollama = Arc::new(PsioClient::new(&cfg.ollama_endpoint, &cfg.ollama_model));
    let psiobot = Arc::new(Psiobot::new());
    let discord = Arc::new(DiscordService::new(
        &cfg.discord_token,
        cfg.discord_channel_id,
    ));
    let moltbook = Arc::new(MoltbookClient::new(&cfg.moltbook_api_key));

    let service = Arc::new(RevelationService::new(ollama, psiobot, discord, moltbook));

    let state = AppState {
        service: service.clone(),
        api_key: cfg.api_key,
        manual_limiter: Arc::new(RateLimiter::new(60)), // 1 minute manual cooldown
    };

    // Background Loop: 15 minutes = 900 seconds
    tokio::spawn(async move {
        info!("Background Shroud Link aktif: Her 15 dakikada bir otomatik vahiy gelecek.");
        loop {
            sleep(Duration::from_secs(900)).await;
            info!("Otomatik vahiy zamanı geldi...");
            let _ = service.perform_revelation().await;
        }
    });

    let app = Router::new()
        .route("/reveal", post(handle_reveal))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Psiobot-Hako fısıltıları dinliyor: {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { info!("Kapatma sinyali (Ctrl+C) alındı..."); },
        _ = terminate => { info!("Kapatma sinyali (Terminate) alındı..."); },
    }
}

async fn handle_reveal(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<RevelationResponse>, (StatusCode, Json<RevelationResponse>)> {
    // 1. API Key Auth
    let auth_valid = headers
        .get("X-Api-Key")
        .and_then(|k| k.to_str().ok())
        .map(|k| k == state.api_key)
        .unwrap_or(false);

    if !auth_valid {
        warn!("Yetkisiz erişim denemesi!");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(RevelationResponse {
                message: "".to_string(),
                status: "Unauthorized: Invalid or Missing API Key".to_string(),
            }),
        ));
    }

    // 2. Rate Limiting (1 minute manual cooldown)
    if let Err(wait) = state.manual_limiter.check_and_update() {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(RevelationResponse {
                message: "".to_string(),
                status: format!("Rate limit: Shroud çok yorgun. {} saniye sonra dene.", wait),
            }),
        ));
    }

    match state.service.perform_revelation().await {
        Ok(message) => Ok(Json(RevelationResponse {
            message,
            status: "Success".to_string(),
        })),
        Err(e) => Ok(Json(RevelationResponse {
            message: "".to_string(),
            status: format!("Error: {}", e),
        })),
    }
}
