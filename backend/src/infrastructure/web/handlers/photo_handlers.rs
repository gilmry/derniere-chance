use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{web, HttpResponse};

use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{bad_request, internal_error};
use crate::infrastructure::web::middleware::AuthenticatedMerchant;

const MAX_PHOTO_BYTES: usize = 5 * 1024 * 1024; // 5 Mo

#[derive(Debug, MultipartForm)]
pub struct PhotoUploadForm {
    #[multipart(limit = "5MB")]
    photo: TempFile,
}

fn extension_for(content_type: &str) -> Option<&'static str> {
    match content_type {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        _ => None,
    }
}

/// Upload une photo de panier vers le stockage (MinIO/S3) et renvoie son URL
/// publique, à réutiliser comme `photo_url` dans `POST /marchands/moi/produits`.
pub async fn upload_photo(
    state: web::Data<AppState>,
    _merchant: AuthenticatedMerchant,
    MultipartForm(form): MultipartForm<PhotoUploadForm>,
) -> HttpResponse {
    let content_type = form
        .photo
        .content_type
        .as_ref()
        .map(|m| m.essence_str().to_string())
        .unwrap_or_default();

    let Some(extension) = extension_for(&content_type) else {
        return bad_request("unsupported image type (jpeg, png, webp only)");
    };

    if form.photo.size > MAX_PHOTO_BYTES {
        return bad_request("photo too large (5MB max)");
    }

    let bytes = match tokio::fs::read(form.photo.file.path()).await {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!(?err, "failed to read uploaded photo tempfile");
            return internal_error();
        }
    };

    match state
        .photo_storage
        .upload(bytes, &content_type, extension)
        .await
    {
        Ok(photo_url) => HttpResponse::Ok().json(serde_json::json!({ "photo_url": photo_url })),
        Err(err) => {
            tracing::error!(%err, "photo upload failed");
            internal_error()
        }
    }
}
