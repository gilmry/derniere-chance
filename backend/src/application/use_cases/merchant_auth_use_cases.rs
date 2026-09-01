use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use thiserror::Error;

use uuid::Uuid;

use crate::application::dto::{
    AuthResponse, Claims, LoginRequest, MerchantResponseDto, RegisterMerchantRequest,
    UpdateMerchantDto,
};
use crate::application::ports::{
    ConsentRepository, EventNotifier, MerchantRepository, MerchantUpdate, NewMerchant, RepoError,
};
use crate::application::use_cases::BETA_CONSENT_VERSION;
use crate::domain::entities::{ConsentSubject, Merchant};
use crate::domain::services::password;

#[derive(Debug, Error)]
pub enum MerchantAuthError {
    #[error("an account already exists for this email")]
    EmailTaken,
    #[error("invalid email or password")]
    InvalidCredentials,
    #[error("consent version is no longer current")]
    StaleConsentVersion,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("password hashing failed")]
    HashingFailed,
    #[error("merchant not found")]
    NotFound,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

const TOKEN_LIFETIME_HOURS: i64 = 24;
const ROLE: &str = "marchand";

pub struct MerchantAuthUseCases {
    merchant_repo: Arc<dyn MerchantRepository>,
    consent_repo: Arc<dyn ConsentRepository>,
    jwt_secret: String,
    event_notifier: Arc<dyn EventNotifier>,
}

impl MerchantAuthUseCases {
    pub fn new(
        merchant_repo: Arc<dyn MerchantRepository>,
        consent_repo: Arc<dyn ConsentRepository>,
        jwt_secret: String,
        event_notifier: Arc<dyn EventNotifier>,
    ) -> Self {
        Self {
            merchant_repo,
            consent_repo,
            jwt_secret,
            event_notifier,
        }
    }

    pub async fn register(
        &self,
        dto: RegisterMerchantRequest,
    ) -> Result<AuthResponse, MerchantAuthError> {
        // Vérifié avant toute écriture : inutile de créer un compte si la
        // case cochée portait sur un texte qui n'est plus en vigueur.
        if dto.consent_version != BETA_CONSENT_VERSION {
            return Err(MerchantAuthError::StaleConsentVersion);
        }

        // Même règle qu'à la réinitialisation : le compte ne doit pas pouvoir
        // naître avec un mot de passe que le parcours « mot de passe oublié »
        // refuserait ensuite.
        password::validate(&dto.password).map_err(MerchantAuthError::InvalidInput)?;

        if self
            .merchant_repo
            .find_by_email(&dto.email)
            .await?
            .is_some()
        {
            return Err(MerchantAuthError::EmailTaken);
        }

        let password_hash = bcrypt::hash(&dto.password, bcrypt::DEFAULT_COST)
            .map_err(|_| MerchantAuthError::HashingFailed)?;

        let merchant = self
            .merchant_repo
            .create(NewMerchant {
                nom: dto.nom,
                adresse: dto.adresse,
                categorie: dto.categorie,
                email: dto.email,
                password_hash,
                latitude: dto.latitude,
                longitude: dto.longitude,
            })
            .await?;

        // Enregistré avant la notification : annoncer une inscription qui
        // serait ensuite bloquée faute de consentement induirait en erreur.
        // Si cette écriture échoue, le compte existe sans consentement et le
        // portier `ConsentedMerchant` le bloque jusqu'à ce que le commerçant
        // consente depuis /pro/consentement : l'échec est fermé.
        self.consent_repo
            .grant(ConsentSubject::Merchant(merchant.id), BETA_CONSENT_VERSION)
            .await?;

        self.event_notifier
            .notify(
                "nouveau_marchand",
                format!(
                    "Nouveau marchand inscrit : {} ({}, {})",
                    merchant.nom, merchant.categorie, merchant.email
                ),
            )
            .await;

        let token = self.mint_token(&merchant, Duration::hours(TOKEN_LIFETIME_HOURS))?;
        Ok(AuthResponse { token })
    }

    pub async fn login(&self, dto: LoginRequest) -> Result<AuthResponse, MerchantAuthError> {
        let merchant = self
            .merchant_repo
            .find_by_email(&dto.email)
            .await?
            .ok_or(MerchantAuthError::InvalidCredentials)?;

        let valid = bcrypt::verify(&dto.password, &merchant.password_hash)
            .map_err(|_| MerchantAuthError::InvalidCredentials)?;
        if !valid {
            return Err(MerchantAuthError::InvalidCredentials);
        }

        let token = self.mint_token(&merchant, Duration::hours(TOKEN_LIFETIME_HOURS))?;
        Ok(AuthResponse { token })
    }

    /// Exposed (not just used internally) so `OAuthUseCases` can mint the
    /// exact same JWT shape for the MCP/OAuth flow, just with a shorter,
    /// refreshable lifetime instead of the 24h `/marchands/connexion` one -
    /// no REST handler needs to special-case a token that came from OAuth.
    pub fn mint_token(
        &self,
        merchant: &Merchant,
        lifetime: Duration,
    ) -> Result<String, MerchantAuthError> {
        let exp = (Utc::now() + lifetime).timestamp() as usize;
        let claims = Claims {
            sub: merchant.id,
            email: merchant.email.clone(),
            role: ROLE.to_string(),
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| MerchantAuthError::InvalidToken)
    }

    pub async fn update_logo(
        &self,
        marchand_id: Uuid,
        logo_url: &str,
    ) -> Result<Merchant, MerchantAuthError> {
        Ok(self.merchant_repo.update_logo(marchand_id, logo_url).await?)
    }

    pub async fn get_own_profile(
        &self,
        marchand_id: Uuid,
    ) -> Result<MerchantResponseDto, MerchantAuthError> {
        let merchant = self
            .merchant_repo
            .find_by_id(marchand_id)
            .await?
            .ok_or(MerchantAuthError::NotFound)?;
        Ok(merchant.into())
    }

    pub async fn update_profile(
        &self,
        marchand_id: Uuid,
        dto: UpdateMerchantDto,
    ) -> Result<MerchantResponseDto, MerchantAuthError> {
        let merchant = self
            .merchant_repo
            .update(
                marchand_id,
                MerchantUpdate {
                    nom: dto.nom,
                    adresse: dto.adresse,
                    categorie: dto.categorie,
                },
            )
            .await?;
        Ok(merchant.into())
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, MerchantAuthError> {
        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| MerchantAuthError::InvalidToken)?;

        if claims.role != ROLE {
            return Err(MerchantAuthError::InvalidToken);
        }
        Ok(claims)
    }
}
