use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use thiserror::Error;

use crate::application::dto::{AuthResponse, Claims, LoginRequest, RegisterConsumerRequest};
use crate::application::ports::{
    ConsentRepository, ConsumerRepository, EventNotifier, NewConsumer, RepoError,
};
use crate::application::use_cases::BETA_CONSENT_VERSION;
use crate::domain::entities::Consumer;

#[derive(Debug, Error)]
pub enum ConsumerAuthError {
    #[error("an account already exists for this email")]
    EmailTaken,
    #[error("invalid email or password")]
    InvalidCredentials,
    #[error("consent version is no longer current")]
    StaleConsentVersion,
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("password hashing failed")]
    HashingFailed,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

const TOKEN_LIFETIME_HOURS: i64 = 24 * 30;
const ROLE: &str = "consommateur";

pub struct ConsumerAuthUseCases {
    consumer_repo: Arc<dyn ConsumerRepository>,
    consent_repo: Arc<dyn ConsentRepository>,
    jwt_secret: String,
    event_notifier: Arc<dyn EventNotifier>,
}

impl ConsumerAuthUseCases {
    pub fn new(
        consumer_repo: Arc<dyn ConsumerRepository>,
        consent_repo: Arc<dyn ConsentRepository>,
        jwt_secret: String,
        event_notifier: Arc<dyn EventNotifier>,
    ) -> Self {
        Self {
            consumer_repo,
            consent_repo,
            jwt_secret,
            event_notifier,
        }
    }

    pub async fn register(
        &self,
        dto: RegisterConsumerRequest,
    ) -> Result<AuthResponse, ConsumerAuthError> {
        // Vérifié avant toute écriture : inutile de créer un compte si la
        // case cochée portait sur un texte qui n'est plus en vigueur (page
        // restée ouverte pendant un déploiement).
        if dto.consent_version != BETA_CONSENT_VERSION {
            return Err(ConsumerAuthError::StaleConsentVersion);
        }

        if self
            .consumer_repo
            .find_by_email(&dto.email)
            .await?
            .is_some()
        {
            return Err(ConsumerAuthError::EmailTaken);
        }

        let password_hash = bcrypt::hash(&dto.password, bcrypt::DEFAULT_COST)
            .map_err(|_| ConsumerAuthError::HashingFailed)?;

        let consumer = self
            .consumer_repo
            .create(NewConsumer {
                email: dto.email,
                password_hash,
            })
            .await?;

        // Deux écritures successives, faute de transaction traversant les
        // ports. Si celle-ci échoue, le compte existe sans consentement et
        // le portier `ConsentedConsumer` le bloque jusqu'à ce que la
        // personne consente depuis /consentement : l'échec est fermé.
        //
        // Enregistré avant la notification : annoncer une inscription qui
        // serait ensuite bloquée faute de consentement induirait en erreur.
        self.consent_repo
            .grant(consumer.id, BETA_CONSENT_VERSION)
            .await?;

        self.event_notifier
            .notify(
                "nouveau_consommateur",
                format!("Nouveau client inscrit : {}", consumer.email),
            )
            .await;

        let token = self.mint_token(&consumer)?;
        Ok(AuthResponse { token })
    }

    pub async fn login(&self, dto: LoginRequest) -> Result<AuthResponse, ConsumerAuthError> {
        let consumer = self
            .consumer_repo
            .find_by_email(&dto.email)
            .await?
            .ok_or(ConsumerAuthError::InvalidCredentials)?;

        let valid = bcrypt::verify(&dto.password, &consumer.password_hash)
            .map_err(|_| ConsumerAuthError::InvalidCredentials)?;
        if !valid {
            return Err(ConsumerAuthError::InvalidCredentials);
        }

        let token = self.mint_token(&consumer)?;
        Ok(AuthResponse { token })
    }

    fn mint_token(&self, consumer: &Consumer) -> Result<String, ConsumerAuthError> {
        let exp = (Utc::now() + Duration::hours(TOKEN_LIFETIME_HOURS)).timestamp() as usize;
        let claims = Claims {
            sub: consumer.id,
            email: consumer.email.clone(),
            role: ROLE.to_string(),
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| ConsumerAuthError::InvalidToken)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, ConsumerAuthError> {
        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| ConsumerAuthError::InvalidToken)?;

        if claims.role != ROLE {
            return Err(ConsumerAuthError::InvalidToken);
        }
        Ok(claims)
    }
}
