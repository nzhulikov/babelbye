use crate::domain::user::{ProfileUpdate, UserProfile, UserSummary};
use crate::ports::UserRepo;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub struct PostgresUserRepo {
    pool: PgPool,
}

impl PostgresUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct UserRow {
    id: Uuid,
    email: Option<String>,
    phone: Option<String>,
    nickname: String,
    tagline: Option<String>,
    native_language: String,
    is_searchable: bool,
    translation_quota_remaining: i32,
    created_at: DateTime<Utc>,
}

impl From<UserRow> for UserProfile {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            phone: row.phone,
            nickname: row.nickname,
            tagline: row.tagline,
            native_language: row.native_language,
            is_searchable: row.is_searchable,
            translation_quota_remaining: row.translation_quota_remaining,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl UserRepo for PostgresUserRepo {
    async fn upsert_profile(
        &self,
        user_id: Uuid,
        update: ProfileUpdate,
    ) -> anyhow::Result<UserProfile> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users
                (id, email, phone, nickname, tagline, native_language, is_searchable)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id)
            DO UPDATE SET
                email = EXCLUDED.email,
                phone = EXCLUDED.phone,
                nickname = EXCLUDED.nickname,
                tagline = EXCLUDED.tagline,
                native_language = EXCLUDED.native_language,
                is_searchable = EXCLUDED.is_searchable
            RETURNING id, email, phone, nickname, tagline, native_language,
                      is_searchable, translation_quota_remaining, created_at
            "#,
        )
        .bind(user_id)
        .bind(update.email)
        .bind(update.phone)
        .bind(update.nickname)
        .bind(update.tagline)
        .bind(update.native_language)
        .bind(update.is_searchable)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    async fn get_profile(&self, user_id: Uuid) -> anyhow::Result<Option<UserProfile>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, email, phone, nickname, tagline, native_language,
                   is_searchable, translation_quota_remaining, created_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    async fn search_users(&self, query: &str) -> anyhow::Result<Vec<UserSummary>> {
        let like_query = format!("%{}%", query);
        let rows = sqlx::query_as::<_, (Uuid, String, Option<String>, String)>(
            r#"
            SELECT id, nickname, tagline, native_language
            FROM users
            WHERE email = $1
               OR phone = $1
               OR (
                   is_searchable = true
                   AND (nickname ILIKE $2 OR tagline ILIKE $2)
               )
            "#,
        )
        .bind(query)
        .bind(like_query)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| UserSummary {
                id: row.0,
                nickname: row.1,
                tagline: row.2,
                native_language: row.3,
            })
            .collect())
    }

    async fn update_quota(&self, user_id: Uuid, delta: i32) -> anyhow::Result<i32> {
        let row = sqlx::query_scalar::<_, i32>(
            r#"
            UPDATE users
            SET translation_quota_remaining = GREATEST(translation_quota_remaining + $2, 0)
            WHERE id = $1
            RETURNING translation_quota_remaining
            "#,
        )
        .bind(user_id)
        .bind(delta)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }
}
