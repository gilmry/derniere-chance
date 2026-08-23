use async_trait::async_trait;
use uuid::Uuid;

use crate::application::ports::{NewProduct, ProductRepository, ProductWithMerchant, RepoError};
use crate::domain::entities::{Product, ProductStatus};
use crate::infrastructure::database::DbPool;

const PRODUCT_COLUMNS: &str =
    "id, marchand_id, nom, description, prix_initial, prix_demarque, quantite, \
     retrait_debut, retrait_fin, statut, photo_url, created_at";

const OFFER_SELECT: &str = "SELECT p.id, p.marchand_id, m.nom AS marchand_nom, \
     m.categorie AS marchand_categorie, m.note AS marchand_note, \
     m.latitude AS marchand_latitude, m.longitude AS marchand_longitude, \
     p.nom, p.description, \
     p.prix_initial, p.prix_demarque, p.quantite, p.retrait_debut, p.retrait_fin, p.statut, \
     p.photo_url \
     FROM produits p JOIN marchands m ON m.id = p.marchand_id";

pub struct PostgresProductRepository {
    pool: DbPool,
}

impl PostgresProductRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for PostgresProductRepository {
    async fn create(&self, new: NewProduct) -> Result<Product, RepoError> {
        let query = format!(
            "INSERT INTO produits (marchand_id, nom, description, prix_initial, prix_demarque, \
             quantite, retrait_debut, retrait_fin, photo_url)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING {PRODUCT_COLUMNS}"
        );
        sqlx::query_as::<_, Product>(&query)
            .bind(new.marchand_id)
            .bind(new.nom)
            .bind(new.description)
            .bind(new.prix_initial)
            .bind(new.prix_demarque)
            .bind(new.quantite)
            .bind(new.retrait_debut)
            .bind(new.retrait_fin)
            .bind(new.photo_url)
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>, RepoError> {
        let query = format!("SELECT {PRODUCT_COLUMNS} FROM produits WHERE id = $1");
        sqlx::query_as::<_, Product>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn find_with_merchant(&self, id: Uuid) -> Result<Option<ProductWithMerchant>, RepoError> {
        let query = format!("{OFFER_SELECT} WHERE p.id = $1");
        sqlx::query_as::<_, ProductWithMerchant>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn list_active(
        &self,
        categorie: Option<&str>,
    ) -> Result<Vec<ProductWithMerchant>, RepoError> {
        let query = format!(
            "{OFFER_SELECT} WHERE p.statut = 'publie' \
             AND ($1::text IS NULL OR m.categorie = $1) \
             ORDER BY p.retrait_fin ASC"
        );
        sqlx::query_as::<_, ProductWithMerchant>(&query)
            .bind(categorie)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn list_active_by_merchant(&self, marchand_id: Uuid) -> Result<Vec<Product>, RepoError> {
        let query = format!(
            "SELECT {PRODUCT_COLUMNS} FROM produits \
             WHERE marchand_id = $1 AND statut = 'publie' ORDER BY retrait_fin ASC"
        );
        sqlx::query_as::<_, Product>(&query)
            .bind(marchand_id)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn list_by_merchant(&self, marchand_id: Uuid) -> Result<Vec<Product>, RepoError> {
        let query = format!(
            "SELECT {PRODUCT_COLUMNS} FROM produits WHERE marchand_id = $1 ORDER BY created_at DESC"
        );
        sqlx::query_as::<_, Product>(&query)
            .bind(marchand_id)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn list_all(&self) -> Result<Vec<ProductWithMerchant>, RepoError> {
        let query = format!("{OFFER_SELECT} ORDER BY p.created_at DESC");
        sqlx::query_as::<_, ProductWithMerchant>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepoError> {
        sqlx::query("DELETE FROM produits WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    async fn count_active(&self) -> Result<i64, RepoError> {
        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM produits WHERE statut = 'publie'")
                .fetch_one(&self.pool)
                .await?;
        Ok(count)
    }

    async fn update_status(&self, id: Uuid, statut: ProductStatus) -> Result<Product, RepoError> {
        let query = format!(
            "UPDATE produits SET statut = $2 WHERE id = $1 RETURNING {PRODUCT_COLUMNS}"
        );
        sqlx::query_as::<_, Product>(&query)
            .bind(id)
            .bind(statut)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(RepoError::NotFound)
    }

    async fn reserve_unit(&self, id: Uuid) -> Result<Product, RepoError> {
        let query = format!(
            "UPDATE produits SET
                 quantite = quantite - 1,
                 statut = CASE WHEN quantite - 1 <= 0 THEN 'ecoule'::product_status ELSE statut END
             WHERE id = $1 AND statut = 'publie' AND quantite > 0
             RETURNING {PRODUCT_COLUMNS}"
        );
        sqlx::query_as::<_, Product>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(RepoError::NotFound)
    }
}
