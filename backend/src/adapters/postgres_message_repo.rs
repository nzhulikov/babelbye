use crate::domain::message::MessageReceipt;
use crate::ports::MessageRepo;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresMessageRepo {
    pool: PgPool,
}

impl PostgresMessageRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessageRepo for PostgresMessageRepo {
    async fn record_receipt(&self, receipt: MessageReceipt) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO message_receipts
                (id, sender_id, recipient_id, has_translation, created_at)
            VALUES
                ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(receipt.id)
        .bind(receipt.sender_id)
        .bind(receipt.recipient_id)
        .bind(receipt.has_translation)
        .bind(receipt.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_history(&self, user_id: Uuid, peer_id: Option<Uuid>) -> anyhow::Result<u64> {
        let result = if let Some(peer) = peer_id {
            sqlx::query(
                r#"
                DELETE FROM message_receipts
                WHERE (sender_id = $1 AND recipient_id = $2)
                   OR (sender_id = $2 AND recipient_id = $1)
                "#,
            )
            .bind(user_id)
            .bind(peer)
            .execute(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                DELETE FROM message_receipts
                WHERE sender_id = $1 OR recipient_id = $1
                "#,
            )
            .bind(user_id)
            .execute(&self.pool)
            .await?
        };

        Ok(result.rows_affected())
    }
}
