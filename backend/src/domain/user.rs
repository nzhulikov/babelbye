use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub nickname: String,
    pub tagline: Option<String>,
    pub native_language: String,
    pub is_searchable: bool,
    pub translation_quota_remaining: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub nickname: String,
    pub tagline: Option<String>,
    pub native_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileUpdate {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub nickname: String,
    pub tagline: Option<String>,
    pub native_language: String,
    pub is_searchable: bool,
}
