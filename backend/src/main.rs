mod adapters;
mod auth;
mod config;
mod delivery;
mod domain;
mod ports;
mod use_cases;

use crate::adapters::{
    GithubFeedbackAdapter, MockFeedbackAdapter, MockTranslationAdapter,
    OpenAiTranslationAdapter, PostgresConnectionRepo, PostgresMessageRepo, PostgresUserRepo,
};
use crate::auth::AuthState;
use crate::config::Config;
use crate::delivery::{http_routes, ws_routes, AppState, WsState};
use crate::ports::{ConnectionRepo, FeedbackPort, MessageRepo, TranslationPort, UserRepo};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let db = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!().run(&db).await?;

    let user_repo: Arc<dyn UserRepo> = Arc::new(PostgresUserRepo::new(db.clone()));
    let connection_repo: Arc<dyn ConnectionRepo> = Arc::new(PostgresConnectionRepo::new(db.clone()));
    let message_repo: Arc<dyn MessageRepo> = Arc::new(PostgresMessageRepo::new(db.clone()));
    let translation: Arc<dyn TranslationPort> = if let Some(api_key) = config.openai_api_key.clone() {
        Arc::new(OpenAiTranslationAdapter::new(
            config.openai_api_url.clone(),
            api_key,
            config.openai_model.clone(),
        ))
    } else {
        Arc::new(MockTranslationAdapter::new())
    };
    let feedback: Arc<dyn FeedbackPort> = match (config.github_token.clone(), config.feedback_repo.clone()) {
        (Some(token), Some(repo)) => Arc::new(GithubFeedbackAdapter::new(repo, token)),
        _ => Arc::new(MockFeedbackAdapter::new()),
    };

    let ws_state = WsState::new();
    let auth_state = AuthState::new(config.clone());

    let app_state = AppState {
        config,
        user_repo,
        connection_repo,
        message_repo,
        translation,
        feedback,
        ws_state,
        auth_state,
    };

    let app = http_routes(app_state.clone()).merge(ws_routes(app_state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("listening on 0.0.0.0:8080");
    axum::serve(listener, app).await?;
    Ok(())
}
