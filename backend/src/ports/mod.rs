use crate::domain::connection::{Connection, ConnectionStatus};
use crate::domain::message::MessageReceipt;
use crate::domain::user::{ProfileUpdate, UserProfile, UserSummary};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn upsert_profile(&self, user_id: Uuid, update: ProfileUpdate)
        -> anyhow::Result<UserProfile>;
    async fn get_profile(&self, user_id: Uuid) -> anyhow::Result<Option<UserProfile>>;
    async fn search_users(&self, query: &str) -> anyhow::Result<Vec<UserSummary>>;
    async fn update_quota(&self, user_id: Uuid, delta: i32) -> anyhow::Result<i32>;
}

#[async_trait]
pub trait ConnectionRepo: Send + Sync {
    async fn request_connection(
        &self,
        requester_id: Uuid,
        addressee_id: Uuid,
    ) -> anyhow::Result<Connection>;
    async fn respond_connection(
        &self,
        requester_id: Uuid,
        addressee_id: Uuid,
        status: ConnectionStatus,
    ) -> anyhow::Result<Connection>;
    async fn list_pending(&self, user_id: Uuid) -> anyhow::Result<Vec<Connection>>;
    async fn list_connections(&self, user_id: Uuid) -> anyhow::Result<Vec<Connection>>;
    async fn is_connected(&self, a: Uuid, b: Uuid) -> anyhow::Result<bool>;
}

#[async_trait]
pub trait MessageRepo: Send + Sync {
    async fn record_receipt(&self, receipt: MessageReceipt) -> anyhow::Result<()>;
    async fn delete_history(&self, user_id: Uuid, peer_id: Option<Uuid>) -> anyhow::Result<u64>;
}

#[async_trait]
pub trait TranslationPort: Send + Sync {
    async fn translate(&self, text: &str, target_locale: &str) -> anyhow::Result<String>;
}

#[async_trait]
pub trait FeedbackPort: Send + Sync {
    async fn create_issue(
        &self,
        title: &str,
        body: &str,
    ) -> anyhow::Result<Option<String>>;
}
