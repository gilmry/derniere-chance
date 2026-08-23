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

/// Valide le fichier reçu et renvoie ses octets + le content-type détecté,
/// ou une réponse d'erreur prête à renvoyer telle quelle.
async fn read_validated_photo(form: &PhotoUploadForm) -> Result<(Vec<u8>, String, &'static str), HttpResponse> {
    let content_type = form
        .photo
        .content_type
        .as_ref()
        .map(|m| m.essence_str().to_string())
        .unwrap_or_default();

    let Some(extension) = extension_for(&content_type) else {
        return Err(bad_request("unsupported image type (jpeg, png, webp only)"));
    };

    if form.photo.size > MAX_PHOTO_BYTES {
        return Err(bad_request("photo too large (5MB max)"));
    }

    let bytes = match tokio::fs::read(form.photo.file.path()).await {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!(?err, "failed to read uploaded photo tempfile");
            return Err(internal_error());
        }
    };

    Ok((bytes, content_type, extension))
}

/// Upload une photo de panier vers le stockage (MinIO/S3) et renvoie son URL
/// publique, à réutiliser comme `photo_url` dans `POST /marchands/moi/produits`.
pub async fn upload_photo(
    state: web::Data<AppState>,
    _merchant: AuthenticatedMerchant,
    MultipartForm(form): MultipartForm<PhotoUploadForm>,
) -> HttpResponse {
    let (bytes, content_type, extension) = match read_validated_photo(&form).await {
        Ok(validated) => validated,
        Err(response) => return response,
    };

    match state
        .photo_storage
        .upload("produits", bytes, &content_type, extension)
        .await
    {
        Ok(photo_url) => HttpResponse::Ok().json(serde_json::json!({ "photo_url": photo_url })),
        Err(err) => {
            tracing::error!(%err, "photo upload failed");
            internal_error()
        }
    }
}

/// Upload le logo/photo du commerce, l'enregistre directement sur le compte
/// marchand et renvoie l'URL publique.
pub async fn upload_logo(
    state: web::Data<AppState>,
    merchant: AuthenticatedMerchant,
    MultipartForm(form): MultipartForm<PhotoUploadForm>,
) -> HttpResponse {
    let (bytes, content_type, extension) = match read_validated_photo(&form).await {
        Ok(validated) => validated,
        Err(response) => return response,
    };

    let logo_url = match state
        .photo_storage
        .upload("marchands", bytes, &content_type, extension)
        .await
    {
        Ok(url) => url,
        Err(err) => {
            tracing::error!(%err, "logo upload failed");
            return internal_error();
        }
    };

    match state
        .merchant_auth_use_cases
        .update_logo(merchant.marchand_id, &logo_url)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "logo_url": logo_url })),
        Err(err) => {
            tracing::error!(?err, "failed to save logo_url on merchant");
            internal_error()
        }
    }
}
