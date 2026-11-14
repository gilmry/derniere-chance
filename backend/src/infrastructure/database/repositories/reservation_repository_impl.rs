use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::application::ports::{
    ConsumerStats, MerchantDailyStats, NewReservationGroup, RepoError, ReservationRepository,
    ReservationSummary,
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
    async fn create_group(&self, new: NewReservationGroup) -> Result<Vec<Reservation>, RepoError> {
        let mut tx = self.pool.begin().await?;

        // Le code doit être libre pour toute la réservation. Le verrou,
        // relâché à la fin de la transaction, empêche deux réservations
        // simultanées de prendre le même code entre le contrôle et l'insertion.
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
            .bind(&new.code)
            .execute(&mut *tx)
            .await?;
        let (pris,): (bool,) =
            sqlx::query_as("SELECT EXISTS (SELECT 1 FROM reservations WHERE code = $1)")
                .bind(&new.code)
                .fetch_one(&mut *tx)
                .await?;
        if pris {
            return Err(RepoError::Conflict("reservation code already used".into()));
        }

        let mut lignes = Vec::with_capacity(new.lignes.len());
        for (produit_id, quantite) in new.lignes {
            // Même décrément atomique qu'à l'unité : qui prend les dernières
            // unités gagne, les autres échouent sans rien réserver.
            let retire = sqlx::query(
                "UPDATE produits SET
                     quantite = quantite - $2,
                     statut = CASE WHEN quantite - $2 <= 0 THEN 'ecoule'::product_status ELSE statut END
                 WHERE id = $1 AND statut = 'publie' AND quantite >= $2",
            )
            .bind(produit_id)
            .bind(quantite)
            .execute(&mut *tx)
            .await?
            .rows_affected();
            if retire == 0 {
                // La transaction abandonnée annule les décréments déjà faits.
                return Err(RepoError::NotFound);
            }
            let ligne = sqlx::query_as::<_, Reservation>(
                "INSERT INTO reservations (produit_id, consommateur_id, code, quantite)
                 VALUES ($1, $2, $3, $4)
                 RETURNING id, produit_id, consommateur_id, code, quantite, statut, created_at",
            )
            .bind(produit_id)
            .bind(new.consommateur_id)
            .bind(&new.code)
            .bind(quantite)
            .fetch_one(&mut *tx)
            .await?;
            lignes.push(ligne);
        }

        tx.commit().await?;
        Ok(lignes)
    }

    async fn paniers_en_attente(
        &self,
        consommateur_id: Uuid,
        marchand_id: Uuid,
    ) -> Result<i64, RepoError> {
        let (total,): (i64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(r.quantite), 0)::bigint
             FROM reservations r JOIN produits p ON p.id = r.produit_id
             WHERE r.consommateur_id = $1 AND p.marchand_id = $2
               AND r.statut = 'reservee' AND p.retrait_fin > now()",
        )
        .bind(consommateur_id)
        .bind(marchand_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(total)
    }

    async fn find_by_code(&self, code: &str) -> Result<Vec<Reservation>, RepoError> {
        sqlx::query_as::<_, Reservation>(
            "SELECT id, produit_id, consommateur_id, code, quantite, statut, created_at
             FROM reservations WHERE code = $1 ORDER BY created_at, id",
        )
        .bind(code)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn mark_recuperee_by_code(&self, code: &str) -> Result<u64, RepoError> {
        Ok(sqlx::query(
            "UPDATE reservations SET statut = 'recuperee' WHERE code = $1 AND statut = 'reservee'",
        )
        .bind(code)
        .execute(&self.pool)
        .await?
        .rows_affected())
    }

    async fn merchant_daily_stats(
        &self,
        marchand_id: Uuid,
    ) -> Result<MerchantDailyStats, RepoError> {
        let (paniers_sauves, chiffre_recupere): (i64, Decimal) = sqlx::query_as(
            "SELECT COALESCE(SUM(r.quantite), 0)::bigint, COALESCE(SUM(p.prix_demarque * r.quantite), 0)
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
            "SELECT COALESCE(SUM(r.quantite), 0)::bigint, \
             COALESCE(SUM((p.prix_initial - p.prix_demarque) * r.quantite), 0)
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

    async fn count(&self) -> Result<i64, RepoError> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM reservations")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    async fn list_by_consumer(
        &self,
        consommateur_id: Uuid,
    ) -> Result<Vec<ReservationSummary>, RepoError> {
        sqlx::query_as::<_, ReservationSummary>(
            "SELECT r.id, r.code, r.quantite, r.statut, m.nom AS marchand_nom, p.nom AS produit_nom, \
             p.prix_demarque, p.retrait_debut, p.retrait_fin, r.created_at \
             FROM reservations r \
             JOIN produits p ON p.id = r.produit_id \
             JOIN marchands m ON m.id = p.marchand_id \
             WHERE r.consommateur_id = $1 \
             ORDER BY r.created_at DESC",
        )
        .bind(consommateur_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }
}
