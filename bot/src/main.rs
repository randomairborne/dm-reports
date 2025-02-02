use std::{
    net::{Ipv6Addr, SocketAddr},
    ops::Deref,
    sync::Arc,
};

use axum::{
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use http::StatusCode;
use tokio::net::TcpListener;
use tracing::Level;
use twilight_http::{client::ClientBuilder, Client};
use twilight_model::id::{marker::WebhookMarker, Id};

use crate::validate_signature::Key;

mod interact;
mod validate_signature;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .json()
        .init();

    let webhook_url = valk_utils::get_var("WEBHOOK_URL");
    let verify_key = Key::from_hex(valk_utils::get_var("VERIFY_KEY").as_bytes())
        .expect("Failed to convert verify_key to a verifying key");

    let (webhook_id, Some(webhook_token)) =
        twilight_util::link::webhook::parse(&webhook_url).expect("Got invalid webhook URL")
    else {
        panic!("Webhook URL did not contain token!");
    };

    let hook = SendableWebhook {
        id: webhook_id,
        token: webhook_token.to_string(),
    };

    let config = Config { hook };
    let client = ClientBuilder::new().build();

    let state = InnerAppState {
        client,
        verify_key,
        config,
    };
    let state = AppState(Arc::new(state));

    let app = Router::new()
        .route("/api/interactions", post(interact::interact))
        .with_state(state);

    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, 8080));
    let tcp = TcpListener::bind(addr).await.unwrap();

    axum::serve(tcp, app)
        .with_graceful_shutdown(vss::shutdown_signal())
        .await
        .unwrap();
}

#[derive(Clone)]
pub struct AppState(pub Arc<InnerAppState>);

impl Deref for AppState {
    type Target = InnerAppState;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub struct InnerAppState {
    client: Client,
    verify_key: Key,
    config: Config,
}

pub struct Config {
    pub hook: SendableWebhook,
}

pub struct SendableWebhook {
    pub id: Id<WebhookMarker>,
    pub token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to validate signature")]
    InvalidSignature(#[from] validate_signature::SignatureValidationFailure),
    #[error("Failed to deserialize body")]
    Json(#[from] serde_json::Error),
    #[error("Failed to read body")]
    BodyRead(#[from] axum::extract::rejection::BytesRejection),
    #[error("Missing header {0}")]
    MissingHeader(&'static str),
    #[error("{0}")]
    Interact(interact::Error),
}

impl Error {
    pub fn status(&self) -> StatusCode {
        match self {
            Self::InvalidSignature(_) => StatusCode::UNAUTHORIZED,
            Self::MissingHeader(_) | Self::Json(_) | Self::Interact(_) => StatusCode::BAD_REQUEST,
            Self::BodyRead(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!(error = ?self, "Error processing request");
        (self.status(), "").into_response()
    }
}
