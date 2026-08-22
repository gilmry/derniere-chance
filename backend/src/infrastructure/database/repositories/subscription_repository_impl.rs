use async_trait::async_trait;
use uuid::Uuid;

use crate::application::ports::{RepoError, SubscriberContact, SubscriptionRepository};
use crate::domain::entities::Merchant;
use crate::infrastructure::database::DbPool;

pub struct PostgresSubscriptionRepository {
    pool: DbPool,
}

impl PostgresSubscriptionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubscriptionRepository for PostgresSubscriptionRepository {
    async fn follow(&self, consommateur_id: Uuid, marchand_id: Uuid) -> Result<(), RepoError> {
        sqlx::query(
            "INSERT INTO abonnements (consommateur_id, marchand_id) VALUES ($1, $2)
             ON CONFLICT (consommateur_id, marchand_id) DO NOTHING",
        )
        .bind(consommateur_id)
        .bind(marchand_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }

    async fn unfollow(&self, consommateur_id: Uuid, marchand_id: Uuid) -> Result<(), RepoError> {
        sqlx::query("DELETE FROM abonnements WHERE consommateur_id = $1 AND marchand_id = $2")
            .bind(consommateur_id)
            .bind(marchand_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    async fn is_following(
        &self,
        consommateur_id: Uuid,
        marchand_id: Uuid,
    ) -> Result<bool, RepoError> {
        let (exists,): (bool,) = sqlx::query_as(
            "SELECT EXISTS (SELECT 1 FROM abonnements WHERE consommateur_id = $1 AND marchand_id = $2)",
        )
        .bind(consommateur_id)
        .bind(marchand_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    async fn list_followed_merchants(
        &self,
        consommateur_id: Uuid,
    ) -> Result<Vec<Merchant>, RepoError> {
        sqlx::query_as::<_, Merchant>(
            "SELECT m.id, m.nom, m.adresse, m.categorie, m.note, m.email, m.password_hash, m.created_at
             FROM marchands m
             JOIN abonnements a ON a.marchand_id = m.id
             WHERE a.consommateur_id = $1
             ORDER BY a.created_at DESC",
        )
        .bind(consommateur_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn list_subscribers(
        &self,
        marchand_id: Uuid,
    ) -> Result<Vec<SubscriberContact>, RepoError> {
        sqlx::query_as::<_, (Uuid, String)>(
            "SELECT c.id, c.email
             FROM consommateurs c
             JOIN abonnements a ON a.consommateur_id = c.id
             WHERE a.marchand_id = $1",
        )
        .bind(marchand_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|(consommateur_id, email)| SubscriberContact {
                    consommateur_id,
                    email,
                })
                .collect()
        })
        .map_err(Into::into)
    }
}
