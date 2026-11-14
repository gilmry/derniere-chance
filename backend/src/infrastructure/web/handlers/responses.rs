use actix_web::HttpResponse;
use serde_json::json;

pub fn bad_request(message: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(json!({ "error": message }))
}

pub fn forbidden(message: &str) -> HttpResponse {
    HttpResponse::Forbidden().json(json!({ "error": message }))
}

pub fn unauthorized(message: &str) -> HttpResponse {
    HttpResponse::Unauthorized().json(json!({ "error": message }))
}

pub fn conflict(message: &str) -> HttpResponse {
    HttpResponse::Conflict().json(json!({ "error": message }))
}

/// Erreur que le client sait traiter d'après `code`, pas seulement afficher.
pub fn unprocessable(code: &str, message: &str) -> HttpResponse {
    HttpResponse::UnprocessableEntity().json(json!({ "error": message, "code": code }))
}

pub fn too_many_requests(message: &str) -> HttpResponse {
    HttpResponse::TooManyRequests().json(json!({ "error": message }))
}

pub fn service_unavailable(message: &str) -> HttpResponse {
    HttpResponse::ServiceUnavailable().json(json!({ "error": message }))
}

pub fn internal_error() -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({ "error": "internal error" }))
}

pub fn not_found(message: &str) -> HttpResponse {
    HttpResponse::NotFound().json(json!({ "error": message }))
}
