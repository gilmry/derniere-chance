use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use thiserror::Error;

use crate::application::dto::{AuthResponse, Claims, LoginRequest};
use crate::application::ports::{AdminRepository, RepoError};
use crate::domain::entities::Admin;

#[derive(Debug, Error)]
pub enum AdminAuthError {
    #[error("invalid email or password")]
    InvalidCredentials,
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

// Session courte : backoffice sensible (accès à toutes les données), pas
// besoin de rester connecté des jours comme un compte marchand/consommateur.
const TOKEN_LIFETIME_HOURS: i64 = 8;
const ROLE: &str = "admin";

pub struct AdminAuthUseCases {
    admin_repo: Arc<dyn AdminRepository>,
    jwt_secret: String,
}

impl AdminAuthUseCases {
    pub fn new(admin_repo: Arc<dyn AdminRepository>, jwt_secret: String) -> Self {
        Self {
            admin_repo,
            jwt_secret,
        }
    }

    pub async fn login(&self, dto: LoginRequest) -> Result<AuthResponse, AdminAuthError> {
        let admin = self
            .admin_repo
            .find_by_email(&dto.email)
            .await?
            .ok_or(AdminAuthError::InvalidCredentials)?;

        let valid = bcrypt::verify(&dto.password, &admin.password_hash)
            .map_err(|_| AdminAuthError::InvalidCredentials)?;
        if !valid {
            return Err(AdminAuthError::InvalidCredentials);
        }

        let token = self.mint_token(&admin)?;
        Ok(AuthResponse { token })
    }

    fn mint_token(&self, admin: &Admin) -> Result<String, AdminAuthError> {
        let exp = (Utc::now() + Duration::hours(TOKEN_LIFETIME_HOURS)).timestamp() as usize;
        let claims = Claims {
            sub: admin.id,
            email: admin.email.clone(),
            role: ROLE.to_string(),
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AdminAuthError::InvalidToken)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AdminAuthError> {
        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AdminAuthError::InvalidToken)?;

        if claims.role != ROLE {
            return Err(AdminAuthError::InvalidToken);
        }
        Ok(claims)
    }
}
