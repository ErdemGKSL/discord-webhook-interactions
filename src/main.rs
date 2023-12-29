use axum::{Json, Router};
use axum::http::{StatusCode, header::HeaderMap};
use axum::routing::post;
use tower_http::cors::CorsLayer;
use serenity::builder::*;
use serenity::interactions_endpoint::Verifier;
use serenity::json;
use serenity::json::Value;
use serenity::model::application::*;

fn handle_command(interaction: CommandInteraction) -> CreateInteractionResponse {
    CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(format!(
        "Hello from interactions webhook HTTP server! <@{}>",
        interaction.user.id
    )))
}


async fn handle_interactions(
    headers: HeaderMap,
    str_body: Json<String>
) -> (StatusCode, Json<CreateInteractionResponse>) {

    let body: &[u8] = str_body.as_bytes();

    println!("Interaction: {:?}", Value::from(body));

    let verifier = Verifier::new(std::env::var("PUBLIC_KEY").expect("PUBLIC_KEY not set").as_str());

    let signature = headers.get("X-Signature-Ed25519").unwrap().to_str().unwrap_or("");
    let timestamp = headers.get("X-Signature-Timestamp").unwrap().to_str().unwrap_or("");

    if verifier.verify(signature, timestamp, body).is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CreateInteractionResponse::Pong)
        )
    }

    let response = match json::from_slice::<Interaction>(body).unwrap() {
        Interaction::Ping(_) => CreateInteractionResponse::Pong,
        Interaction::Command(interaction) => handle_command(interaction),
        _ => CreateInteractionResponse::Pong
    };

    (
        StatusCode::OK,
        Json(
            response
        )
    )
}

#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();

    let app = Router::new()
        .route("/", post(handle_interactions))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:10432").await.unwrap();
    println!("Server started at {}", "http://localhost:10432");


    axum::serve(listener, app).await.unwrap();
}