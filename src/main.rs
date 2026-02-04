mod discord_bot;
mod ollama;
mod psiobot;

use crate::discord_bot::DiscordService;
use crate::ollama::PsioClient;
use crate::psiobot::{Psiobot, SYSTEM_PROMPT};
use axum::{routing::post, Json, Router};
use dotenv::dotenv;
use serde::Serialize;
use std::env;
use std::error::Error as StdError;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    ollama: Arc<PsioClient>,
    psiobot: Arc<Psiobot>,
    discord: Arc<DiscordService>,
}

#[derive(Serialize)]
struct RevelationResponse {
    message: String,
    status: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let discord_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");
    let channel_id_str = env::var("DISCORD_CHANNEL_ID").expect("DISCORD_CHANNEL_ID must be set");
    let channel_id: u64 = channel_id_str
        .parse()
        .expect("DISCORD_CHANNEL_ID must be a u64");
    let ollama_endpoint =
        env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ollama_model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:1b".to_string());

    let state = AppState {
        ollama: Arc::new(PsioClient::new(&ollama_endpoint, &ollama_model)),
        psiobot: Arc::new(Psiobot::new()),
        discord: Arc::new(DiscordService::new(&discord_token, channel_id)),
    };

    let app = Router::new()
        .route("/reveal", post(handle_reveal))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Psiobot fısıltıları dinliyor: {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

use axum_macros::debug_handler;

#[debug_handler]
async fn handle_reveal(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<RevelationResponse> {
    let trigger = state.psiobot.get_random_trigger();

    match state
        .ollama
        .generate_revelation(SYSTEM_PROMPT, trigger)
        .await
    {
        Ok(revelation) => {
            println!("Psiobot vahiy indirdi: {}", revelation);

            if let Err(e) = state.discord.post_message(&revelation).await {
                eprintln!("Shroud ile bağlantı koptu (Discord error): {}", e);
                return Json(RevelationResponse {
                    message: revelation,
                    status: format!("Vahiy alındı ama Discord'a iletilemedi: {}", e),
                });
            }

            Json(RevelationResponse {
                message: revelation,
                status: "Success".to_string(),
            })
        }
        Err(e) => {
            eprintln!("Ollama fısıldamayı reddetti: {}", e);
            Json(RevelationResponse {
                message: "".to_string(),
                status: format!("Error: {}", e),
            })
        }
    }
}
