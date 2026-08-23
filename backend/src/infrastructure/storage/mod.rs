//! Stockage des photos de panier (MinIO/S3, cf. docker-compose.yml). Le
//! bucket est créé et rendu public en lecture par le sidecar
//! `minio-bootstrap` au démarrage - ce module ne fait qu'écrire dedans.
//! Les lectures ne passent jamais par le backend : Traefik route
//! `PathPrefix(/photos)` directement vers MinIO (voir infra/shared-traefik
//! et le docker-compose de DernièreChance), pour ne pas charger le seul
//! vCPU partagé avec Elevia sur chaque affichage d'image du feed.

use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct PhotoStorageConfig {
    pub bucket: String,
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    /// Base URL depuis laquelle les objets du bucket sont servis publiquement
    /// (Traefik -> MinIO), ex. `https://api.derniere-chance.ecosolva.org/photos`.
    pub public_base_url: String,
}

impl PhotoStorageConfig {
    pub fn from_env() -> Self {
        Self {
            bucket: std::env::var("S3_BUCKET").unwrap_or_else(|_| "photos".to_string()),
            endpoint: std::env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "http://minio:9000".to_string()),
            access_key: std::env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
            secret_key: std::env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
            public_base_url: std::env::var("S3_PUBLIC_URL")
                .unwrap_or_else(|_| "http://localhost:8080/photos".to_string()),
        }
    }
}

pub struct PhotoStorage {
    client: Client,
    bucket: String,
    public_base_url: String,
}

impl PhotoStorage {
    pub async fn from_config(config: PhotoStorageConfig) -> Self {
        let credentials = SharedCredentialsProvider::new(Credentials::new(
            config.access_key,
            config.secret_key,
            None,
            None,
            "derniere-chance-storage",
        ));
        // MinIO ignore la région mais le SDK en exige une.
        let region_provider = RegionProviderChain::first_try(Region::new("us-east-1"));
        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .credentials_provider(credentials)
            .load()
            .await;
        let s3_config = S3ConfigBuilder::from(&shared_config)
            .endpoint_url(config.endpoint)
            .force_path_style(true)
            .build();

        Self {
            client: Client::from_conf(s3_config),
            bucket: config.bucket,
            public_base_url: config.public_base_url,
        }
    }

    /// Upload une photo et renvoie son URL publique. `extension` sans le
    /// point (ex. `"jpg"`), `prefix` sans slash de fin (ex. `"produits"`,
    /// `"marchands"`) pour séparer les objets par usage dans le bucket.
    pub async fn upload(
        &self,
        prefix: &str,
        bytes: Vec<u8>,
        content_type: &str,
        extension: &str,
    ) -> Result<String, String> {
        let key = format!("{prefix}/{}.{extension}", Uuid::new_v4());

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(bytes))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| format!("échec de l'upload vers le stockage: {e}"))?;

        Ok(format!(
            "{}/{key}",
            self.public_base_url.trim_end_matches('/')
        ))
    }
}
