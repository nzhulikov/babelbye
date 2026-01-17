use crate::domain::connection::{Connection, ConnectionStatus};
use crate::domain::message::MessageReceipt;
use crate::domain::user::{ProfileUpdate, UserProfile, UserSummary};
use crate::ports::{ConnectionRepo, FeedbackPort, MessageRepo, TranslationPort, UserRepo};
use chrono::Utc;
use uuid::Uuid;

pub async fn upsert_profile(
    user_repo: &dyn UserRepo,
    user_id: Uuid,
    update: ProfileUpdate,
) -> anyhow::Result<UserProfile> {
    user_repo.upsert_profile(user_id, update).await
}

pub async fn get_profile(
    user_repo: &dyn UserRepo,
    user_id: Uuid,
) -> anyhow::Result<Option<UserProfile>> {
    user_repo.get_profile(user_id).await
}

pub async fn search_users(
    user_repo: &dyn UserRepo,
    query: &str,
) -> anyhow::Result<Vec<UserSummary>> {
    user_repo.search_users(query).await
}

pub async fn request_connection(
    connection_repo: &dyn ConnectionRepo,
    requester_id: Uuid,
    addressee_id: Uuid,
) -> anyhow::Result<Connection> {
    connection_repo
        .request_connection(requester_id, addressee_id)
        .await
}

pub async fn respond_connection(
    connection_repo: &dyn ConnectionRepo,
    requester_id: Uuid,
    addressee_id: Uuid,
    accept: bool,
) -> anyhow::Result<Connection> {
    let status = if accept {
        ConnectionStatus::Accepted
    } else {
        ConnectionStatus::Declined
    };
    connection_repo
        .respond_connection(requester_id, addressee_id, status)
        .await
}

pub async fn list_connections(
    connection_repo: &dyn ConnectionRepo,
    user_id: Uuid,
) -> anyhow::Result<Vec<Connection>> {
    connection_repo.list_connections(user_id).await
}

pub async fn list_pending_connections(
    connection_repo: &dyn ConnectionRepo,
    user_id: Uuid,
) -> anyhow::Result<Vec<Connection>> {
    connection_repo.list_pending(user_id).await
}

pub async fn translate_or_fallback(
    translation: &dyn TranslationPort,
    user_repo: &dyn UserRepo,
    recipient_id: Uuid,
    text: &str,
) -> anyhow::Result<(String, bool)> {
    let profile = user_repo
        .get_profile(recipient_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("recipient missing"))?;
    if profile.translation_quota_remaining <= 0 {
        return Ok((text.to_string(), false));
    }
    let _ = user_repo.update_quota(recipient_id, -1).await?;
    let translated = translation
        .translate(text, &profile.native_language)
        .await?;
    Ok((translated, true))
}

pub async fn record_receipt(
    message_repo: &dyn MessageRepo,
    sender_id: Uuid,
    recipient_id: Uuid,
    has_translation: bool,
) -> anyhow::Result<()> {
    let receipt = MessageReceipt {
        id: Uuid::new_v4(),
        sender_id,
        recipient_id,
        has_translation,
        created_at: Utc::now(),
    };
    message_repo.record_receipt(receipt).await
}

pub async fn delete_history(
    message_repo: &dyn MessageRepo,
    user_id: Uuid,
    peer_id: Option<Uuid>,
) -> anyhow::Result<u64> {
    message_repo.delete_history(user_id, peer_id).await
}

pub async fn submit_feedback(
    feedback: &dyn FeedbackPort,
    user_id: Uuid,
    message: &str,
) -> anyhow::Result<Option<String>> {
    let title = format!("Feedback from {}", user_id);
    feedback
        .create_issue(&title, message)
        .await
}
