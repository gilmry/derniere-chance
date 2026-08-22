use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use thiserror::Error;

use crate::application::dto::{AuthResponse, Claims, LoginRequest, RegisterMerchantRequest};
use crate::application::ports::{MerchantRepository, NewMerchant, RepoError};
use crate::domain::entities::Merchant;

#[derive(Debug, Error)]
pub enum MerchantAuthError {
    #[error("an account already exists for this email")]
    EmailTaken,
    #[error("invalid email or password")]
    InvalidCredentials,
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("password hashing failed")]
    HashingFailed,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

const TOKEN_LIFETIME_HOURS: i64 = 24;
const ROLE: &str = "marchand";

pub struct MerchantAuthUseCases {
    merchant_repo: Arc<dyn MerchantRepository>,
    jwt_secret: String,
}

impl MerchantAuthUseCases {
    pub fn new(merchant_repo: Arc<dyn MerchantRepository>, jwt_secret: String) -> Self {
        Self {
            merchant_repo,
            jwt_secret,
        }
    }

    pub async fn register(
        &self,
        dto: RegisterMerchantRequest,
    ) -> Result<AuthResponse, MerchantAuthError> {
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

        let token = self.mint_token(&merchant)?;
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

        let token = self.mint_token(&merchant)?;
        Ok(AuthResponse { token })
    }

    fn mint_token(&self, merchant: &Merchant) -> Result<String, MerchantAuthError> {
        let exp = (Utc::now() + Duration::hours(TOKEN_LIFETIME_HOURS)).timestamp() as usize;
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
