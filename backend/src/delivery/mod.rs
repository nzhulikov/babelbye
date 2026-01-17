use crate::auth::AuthState;
use crate::config::Config;
use crate::domain::user::{ProfileUpdate, UserProfile, UserSummary};
use crate::ports::{ConnectionRepo, FeedbackPort, MessageRepo, TranslationPort, UserRepo};
use crate::use_cases;
use axum::extract::{FromRef, Query, State, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub user_repo: Arc<dyn UserRepo>,
    pub connection_repo: Arc<dyn ConnectionRepo>,
    pub message_repo: Arc<dyn MessageRepo>,
    pub translation: Arc<dyn TranslationPort>,
    pub feedback: Arc<dyn FeedbackPort>,
    pub ws_state: WsState,
    pub auth_state: AuthState,
}

#[derive(Clone, Default)]
pub struct WsState {
    clients: Arc<RwLock<HashMap<Uuid, WsClient>>>,
}

impl WsState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone)]
struct WsClient {
    sender: mpsc::UnboundedSender<axum::extract::ws::Message>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
}

#[derive(Clone)]
struct AuthUser {
    user_id: Uuid,
}

#[derive(Debug)]
struct AuthError;

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                message: "unauthorized".to_string(),
            }),
        )
            .into_response()
    }
}

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
    let app_state = AppState::from_ref(state);
        let headers = &parts.headers;
        if app_state.config.auth_bypass {
            if let Some(value) = headers
                .get("x-user-id")
                .and_then(|value| value.to_str().ok())
            {
                if let Ok(user_id) = Uuid::parse_str(value) {
                    return Ok(Self { user_id });
                }
            }
        }

        let token = bearer_token(headers).ok_or(AuthError)?;
        let claims = app_state
            .auth_state
            .verify(token)
            .await
            .map_err(|_| AuthError)?;
        let user_id = parse_user_id(&claims.sub).ok_or(AuthError)?;
        Ok(Self { user_id })
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
}

fn parse_user_id(sub: &str) -> Option<Uuid> {
    Uuid::parse_str(sub)
        .ok()
        .or_else(|| sub.split('|').last().and_then(|part| Uuid::parse_str(part).ok()))
}

pub fn http_routes(state: AppState) -> Router {
    let cors = cors_layer(&state.config.allowed_origins);
    Router::new()
        .route("/healthz", get(healthz))
        .route("/api/profile", get(get_profile).put(update_profile))
        .route("/api/search", get(search_users))
        .route("/api/connections", get(list_connections))
        .route("/api/connections/requests", get(list_pending_requests))
        .route("/api/connections/request", post(request_connection))
        .route("/api/connections/respond", post(respond_connection))
        .route("/api/history", delete(delete_all_history))
        .route("/api/history/:peer_id", delete(delete_history_with_peer))
        .route("/api/feedback", post(submit_feedback))
        .with_state(state)
        .layer(cors)
}

pub fn ws_routes(state: AppState) -> Router {
    Router::new().route("/ws", get(ws_handler)).with_state(state)
}

fn cors_layer(allowed: &str) -> CorsLayer {
    if allowed == "*" {
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_headers(Any)
            .allow_methods(Any);
    }
    let origins: Vec<_> = allowed
        .split(',')
        .filter_map(|value| value.trim().parse().ok())
        .collect();
    CorsLayer::new()
        .allow_origin(origins)
        .allow_headers(Any)
        .allow_methods(Any)
}

async fn healthz() -> impl IntoResponse {
    StatusCode::OK
}

async fn get_profile(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<UserProfile>, AuthError> {
    let profile = use_cases::get_profile(state.user_repo.as_ref(), user_id)
        .await
        .map_err(|_| AuthError)?
        .ok_or(AuthError)?;
    Ok(Json(profile))
}

async fn update_profile(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Json(payload): Json<ProfileUpdate>,
) -> Result<Json<UserProfile>, AuthError> {
    let profile = use_cases::upsert_profile(state.user_repo.as_ref(), user_id, payload)
        .await
        .map_err(|_| AuthError)?;
    Ok(Json(profile))
}

#[derive(Deserialize)]
struct SearchQuery {
    query: String,
}

async fn search_users(
    State(state): State<AppState>,
    AuthUser { .. }: AuthUser,
    Query(search): Query<SearchQuery>,
) -> Result<Json<Vec<UserSummary>>, AuthError> {
    let results = use_cases::search_users(state.user_repo.as_ref(), &search.query)
        .await
        .map_err(|_| AuthError)?;
    Ok(Json(results))
}

#[derive(Deserialize)]
struct ConnectionRequestPayload {
    target_user_id: Uuid,
}

async fn request_connection(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Json(payload): Json<ConnectionRequestPayload>,
) -> Result<Json<crate::domain::connection::Connection>, AuthError> {
    let connection = use_cases::request_connection(
        state.connection_repo.as_ref(),
        user_id,
        payload.target_user_id,
    )
    .await
    .map_err(|_| AuthError)?;
    Ok(Json(connection))
}

#[derive(Deserialize)]
struct ConnectionRespondPayload {
    requester_id: Uuid,
    accept: bool,
}

async fn respond_connection(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Json(payload): Json<ConnectionRespondPayload>,
) -> Result<Json<crate::domain::connection::Connection>, AuthError> {
    let connection = use_cases::respond_connection(
        state.connection_repo.as_ref(),
        payload.requester_id,
        user_id,
        payload.accept,
    )
    .await
    .map_err(|_| AuthError)?;
    Ok(Json(connection))
}

async fn list_connections(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Vec<crate::domain::connection::Connection>>, AuthError> {
    let connections = use_cases::list_connections(state.connection_repo.as_ref(), user_id)
        .await
        .map_err(|_| AuthError)?;
    Ok(Json(connections))
}

async fn list_pending_requests(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Vec<crate::domain::connection::Connection>>, AuthError> {
    let connections = use_cases::list_pending_connections(state.connection_repo.as_ref(), user_id)
        .await
        .map_err(|_| AuthError)?;
    Ok(Json(connections))
}

async fn delete_all_history(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<u64>, AuthError> {
    let deleted = use_cases::delete_history(state.message_repo.as_ref(), user_id, None)
        .await
        .map_err(|_| AuthError)?;
    Ok(Json(deleted))
}

async fn delete_history_with_peer(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    axum::extract::Path(peer_id): axum::extract::Path<Uuid>,
) -> Result<Json<u64>, AuthError> {
    let deleted =
        use_cases::delete_history(state.message_repo.as_ref(), user_id, Some(peer_id))
            .await
            .map_err(|_| AuthError)?;
    Ok(Json(deleted))
}

#[derive(Deserialize)]
struct FeedbackPayload {
    message: String,
}

async fn submit_feedback(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Json(payload): Json<FeedbackPayload>,
) -> Result<Json<HashMap<&'static str, Option<String>>>, AuthError> {
    let issue_url = use_cases::submit_feedback(state.feedback.as_ref(), user_id, &payload.message)
        .await
        .map_err(|_| AuthError)?;
    let mut response = HashMap::new();
    response.insert("issue_url", issue_url);
    Ok(Json(response))
}

#[derive(Deserialize)]
struct WsQuery {
    token: Option<String>,
    user_id: Option<Uuid>,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<WsQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, headers, query))
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    state: AppState,
    headers: HeaderMap,
    query: WsQuery,
) {
    let user_id = match extract_ws_user(&state, &headers, &query).await {
        Ok(user_id) => user_id,
        Err(_) => return,
    };

    let (mut sender_ws, mut receiver_ws) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let ws_state = state.ws_state.clone();

    ws_state
        .clients
        .write()
        .await
        .insert(user_id, WsClient { sender: tx });

    let send_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if sender_ws.send(message).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver_ws.next().await {
            if let axum::extract::ws::Message::Text(text) = message {
                if handle_ws_message(user_id, &state, &text).await.is_err() {
                    break;
                }
            }
        }
    });

    let _ = tokio::join!(send_task, recv_task);
    ws_state.clients.write().await.remove(&user_id);
}

async fn extract_ws_user(
    state: &AppState,
    headers: &HeaderMap,
    query: &WsQuery,
) -> Result<Uuid, AuthError> {
    if state.config.auth_bypass {
        if let Some(user_id) = query.user_id {
            return Ok(user_id);
        }
        if let Some(value) = headers
            .get("x-user-id")
            .and_then(|value| value.to_str().ok())
        {
            if let Ok(user_id) = Uuid::parse_str(value) {
                return Ok(user_id);
            }
        }
    }
    let token = query.token.as_deref().or_else(|| bearer_token(headers)).ok_or(AuthError)?;
    let claims = state.auth_state.verify(token).await.map_err(|_| AuthError)?;
    parse_user_id(&claims.sub).ok_or(AuthError)
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ClientEvent {
    Message {
        to: Uuid,
        text: String,
        client_id: Option<String>,
    },
    Typing { to: Uuid },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ServerEvent {
    Message {
        from: Uuid,
        text: String,
        original: String,
        translated: bool,
        client_id: Option<String>,
    },
    Delivery {
        to: Uuid,
        status: String,
        client_id: Option<String>,
    },
    Error {
        message: String,
    },
}

async fn handle_ws_message(user_id: Uuid, state: &AppState, text: &str) -> anyhow::Result<()> {
    let event: ClientEvent = serde_json::from_str(text)?;
    match event {
        ClientEvent::Message {
            to,
            text,
            client_id,
        } => {
            if !state
                .connection_repo
                .is_connected(user_id, to)
                .await
                .unwrap_or(false)
            {
                send_to(
                    &state.ws_state,
                    user_id,
                    ServerEvent::Error {
                        message: "connection_required".to_string(),
                    },
                )
                .await;
                return Ok(());
            }

            let (translated_text, did_translate) = use_cases::translate_or_fallback(
                state.translation.as_ref(),
                state.user_repo.as_ref(),
                to,
                &text,
            )
            .await?;
            use_cases::record_receipt(state.message_repo.as_ref(), user_id, to, did_translate)
                .await?;

            send_to(
                &state.ws_state,
                to,
                ServerEvent::Message {
                    from: user_id,
                    text: translated_text.clone(),
                    original: text.clone(),
                    translated: did_translate,
                    client_id: client_id.clone(),
                },
            )
            .await;

            send_to(
                &state.ws_state,
                user_id,
                ServerEvent::Delivery {
                    to,
                    status: "sent".to_string(),
                    client_id,
                },
            )
            .await;
        }
        ClientEvent::Typing { to } => {
            send_to(
                &state.ws_state,
                to,
                ServerEvent::Delivery {
                    to: user_id,
                    status: "typing".to_string(),
                    client_id: None,
                },
            )
            .await;
        }
    }
    Ok(())
}

async fn send_to(ws_state: &WsState, user_id: Uuid, event: ServerEvent) {
    if let Some(client) = ws_state.clients.read().await.get(&user_id) {
        let _ = client
            .sender
            .send(axum::extract::ws::Message::Text(
                serde_json::to_string(&event).unwrap_or_else(|_| "".to_string()),
            ));
    }
}

