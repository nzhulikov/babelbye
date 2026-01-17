use crate::domain::connection::{Connection, ConnectionStatus};
use crate::ports::ConnectionRepo;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub struct PostgresConnectionRepo {
    pool: PgPool,
}

impl PostgresConnectionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct ConnectionRow {
    id: Uuid,
    requester_id: Uuid,
    addressee_id: Uuid,
    status: String,
    created_at: DateTime<Utc>,
}

impl ConnectionRow {
    fn to_domain(self) -> Connection {
        Connection {
            id: self.id,
            requester_id: self.requester_id,
            addressee_id: self.addressee_id,
            status: match self.status.as_str() {
                "accepted" => ConnectionStatus::Accepted,
                "declined" => ConnectionStatus::Declined,
                _ => ConnectionStatus::Pending,
            },
            created_at: self.created_at,
        }
    }
}

#[async_trait]
impl ConnectionRepo for PostgresConnectionRepo {
    async fn request_connection(
        &self,
        requester_id: Uuid,
        addressee_id: Uuid,
    ) -> anyhow::Result<Connection> {
        let row = sqlx::query_as::<_, ConnectionRow>(
            r#"
            INSERT INTO connections
                (id, requester_id, addressee_id, status)
            VALUES
                ($1, $2, $3, 'pending')
            ON CONFLICT (requester_id, addressee_id)
            DO UPDATE SET status = 'pending'
            RETURNING id, requester_id, addressee_id, status, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(requester_id)
        .bind(addressee_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.to_domain())
    }

    async fn respond_connection(
        &self,
        requester_id: Uuid,
        addressee_id: Uuid,
        status: ConnectionStatus,
    ) -> anyhow::Result<Connection> {
        let status_value = match status {
            ConnectionStatus::Accepted => "accepted",
            ConnectionStatus::Declined => "declined",
            ConnectionStatus::Pending => "pending",
        };

        let row = sqlx::query_as::<_, ConnectionRow>(
            r#"
            UPDATE connections
            SET status = $3
            WHERE requester_id = $1 AND addressee_id = $2
            RETURNING id, requester_id, addressee_id, status, created_at
            "#,
        )
        .bind(requester_id)
        .bind(addressee_id)
        .bind(status_value)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.to_domain())
    }

    async fn list_connections(&self, user_id: Uuid) -> anyhow::Result<Vec<Connection>> {
        let rows = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, requester_id, addressee_id, status, created_at
            FROM connections
            WHERE requester_id = $1 OR addressee_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.to_domain()).collect())
    }

    async fn list_pending(&self, user_id: Uuid) -> anyhow::Result<Vec<Connection>> {
        let rows = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, requester_id, addressee_id, status, created_at
            FROM connections
            WHERE addressee_id = $1 AND status = 'pending'
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.to_domain()).collect())
    }

    async fn is_connected(&self, a: Uuid, b: Uuid) -> anyhow::Result<bool> {
        let connected = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(1)
            FROM connections
            WHERE ((requester_id = $1 AND addressee_id = $2)
               OR (requester_id = $2 AND addressee_id = $1))
              AND status = 'accepted'
            "#,
        )
        .bind(a)
        .bind(b)
        .fetch_one(&self.pool)
        .await?;

        Ok(connected > 0)
    }
}
