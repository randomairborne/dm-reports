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
use twilight_http::Client;
use twilight_model::id::{
    marker::{ApplicationMarker, WebhookMarker},
    Id,
};

use crate::{interact::register_commands, validate_signature::Key};

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

    let token = valk_utils::get_var("DISCORD_TOKEN");
    let webhook_url = valk_utils::get_var("WEBHOOK_URL");

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

    let client = Client::new(token.clone());

    let current_user_application = client
        .current_user_application()
        .await
        .unwrap()
        .model()
        .await
        .unwrap();

    let verify_key = Key::from_hex(current_user_application.verify_key.as_bytes()).unwrap();
    let application_id = current_user_application.id;

    let state = InnerAppState {
        client,
        verify_key,
        config,
        application_id,
    };
    let state = AppState(Arc::new(state));

    register_commands(&state).await.unwrap();

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
    application_id: Id<ApplicationMarker>,
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
        (self.status(), "").into_response()
    }
}
