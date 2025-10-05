mod admin;
mod config;

use std::net::IpAddr;

use crate::{
    admin::admin_subrouter,
    config::{Action, DynamicSettings, Persisted},
};
use axum::{
    Router,
    extract::{FromRef, State},
    response::Redirect,
    routing::get,
};
use clap::Parser as _;
use sec::Secret;

#[derive(clap::Parser)]
pub struct Args {
    /// IP address to listen on
    #[arg(short, long, default_value = "::")]
    pub address: IpAddr,

    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    pub port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let state = ArmQRState {
        settings: Persisted::open("redirect_config.toml".into()).await,
        password: std::env::var("ADMIN_PASSWORD")
            .expect("ADMIN_PASSWORD not provided")
            .into(),
    };
    let app = Router::new()
        .route("/", get(index))
        .nest("/admin", admin_subrouter())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("::", args.port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone)]
pub struct ArmQRState {
    settings: Persisted<DynamicSettings>,
    password: Secret<String>,
}

pub trait AdminPasswordValidator {
    fn validate_admin_password(&self, input: Secret<String>) -> bool;
}

impl AdminPasswordValidator for ArmQRState {
    fn validate_admin_password(&self, input: Secret<String>) -> bool {
        self.password == input
    }
}

impl FromRef<ArmQRState> for Persisted<DynamicSettings> {
    fn from_ref(input: &ArmQRState) -> Self {
        input.settings.clone()
    }
}

#[axum::debug_handler]
async fn index(State(state): State<Persisted<DynamicSettings>>) -> Redirect {
    let profile = state.snapshot().await;

    match &profile.current_profile().action {
        Action::Redirect(uri) => Redirect::to(&uri),
    }
}
