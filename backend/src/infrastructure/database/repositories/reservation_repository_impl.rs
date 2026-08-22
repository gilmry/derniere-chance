use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::application::ports::{
    ConsumerStats, MerchantDailyStats, NewReservation, RepoError, ReservationRepository,
};
use crate::domain::entities::Reservation;
use crate::infrastructure::database::DbPool;

pub struct PostgresReservationRepository {
    pool: DbPool,
}

impl PostgresReservationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReservationRepository for PostgresReservationRepository {
    async fn create(&self, new: NewReservation) -> Result<Reservation, RepoError> {
        sqlx::query_as::<_, Reservation>(
            "INSERT INTO reservations (produit_id, consommateur_id, code) VALUES ($1, $2, $3)
             RETURNING id, produit_id, consommateur_id, code, statut, created_at",
        )
        .bind(new.produit_id)
        .bind(new.consommateur_id)
        .bind(new.code)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| match &err {
            // 23505 = unique_violation - the `code` column collided, the use
            // case regenerates and retries rather than surfacing this.
            sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23505") => {
                RepoError::Conflict("reservation code already used".into())
            }
            _ => err.into(),
        })
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<Reservation>, RepoError> {
        sqlx::query_as::<_, Reservation>(
            "SELECT id, produit_id, consommateur_id, code, statut, created_at
             FROM reservations WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn mark_recuperee(&self, id: Uuid) -> Result<Reservation, RepoError> {
        sqlx::query_as::<_, Reservation>(
            "UPDATE reservations SET statut = 'recuperee' WHERE id = $1
             RETURNING id, produit_id, consommateur_id, code, statut, created_at",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepoError::NotFound)
    }

    async fn merchant_daily_stats(&self, marchand_id: Uuid) -> Result<MerchantDailyStats, RepoError> {
        let (paniers_sauves, chiffre_recupere): (i64, Decimal) = sqlx::query_as(
            "SELECT COUNT(*)::bigint, COALESCE(SUM(p.prix_demarque), 0)
             FROM reservations r JOIN produits p ON p.id = r.produit_id
             WHERE p.marchand_id = $1 AND r.created_at::date = CURRENT_DATE",
        )
        .bind(marchand_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(MerchantDailyStats {
            paniers_sauves,
            chiffre_recupere,
        })
    }

    async fn consumer_stats(&self, consommateur_id: Uuid) -> Result<ConsumerStats, RepoError> {
        let (paniers_sauves, montant_economise): (i64, Decimal) = sqlx::query_as(
            "SELECT COUNT(*)::bigint, COALESCE(SUM(p.prix_initial - p.prix_demarque), 0)
             FROM reservations r JOIN produits p ON p.id = r.produit_id
             WHERE r.consommateur_id = $1",
        )
        .bind(consommateur_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ConsumerStats {
            paniers_sauves,
            montant_economise,
        })
    }
}
