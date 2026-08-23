//! MCP (Model Context Protocol) endpoint: lets a marchand plug Claude
//! directly into their DernièreChance boutique as a "remote MCP server".
//!
//! Single `POST /mcp` endpoint speaking JSON-RPC 2.0 over the MCP
//! "Streamable HTTP" transport, stateless (every request re-authenticates
//! via the same `Authorization: Bearer <JWT>` header the REST API already
//! uses, through the existing `AuthenticatedMerchant` extractor - no
//! separate session/OAuth flow at this layer). Read AND write: every tool
//! reuses an existing use case exactly as the REST handlers do (same
//! ownership checks, same validation), so there is no new business logic
//! here, only a JSON-RPC adapter. Mutating tools rely on the MCP client's
//! own tool-call confirmation UX (Claude Desktop/Code/claude.ai all prompt
//! before running a write tool) rather than a second confirmation step here.
//!
//! Repris du pattern "mcp-oauth-maison" d'Elevia
//! (https://github.com/gilmry/elevia), étendu en lecture/écriture.

use actix_web::{error, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::application::dto::{CreateProductDto, UpdateMerchantDto};
use crate::application::use_cases::{
    DashboardError, MerchantAuthError, ProductError, ReservationError,
};
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::middleware::AuthenticatedMerchant;

const PROTOCOL_VERSION: &str = "2025-06-18";

/// Malformed request bodies would otherwise fall through to actix's default
/// plain-text 400, which isn't valid JSON-RPC and could confuse a strict MCP
/// client. `id` is unknown at this point (the body didn't even parse), so it
/// is null per the JSON-RPC spec for that case.
pub fn json_error_handler(err: error::JsonPayloadError, _req: &HttpRequest) -> error::Error {
    let body = json!({
        "jsonrpc": "2.0",
        "id": Value::Null,
        "error": { "code": -32700, "message": format!("parse error: {err}") },
    });
    error::InternalError::from_response(err, HttpResponse::BadRequest().json(body)).into()
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcErrorObj>,
}

#[derive(Debug, Serialize)]
struct JsonRpcErrorObj {
    code: i32,
    message: String,
}

impl JsonRpcResponse {
    fn result(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcErrorObj {
                code,
                message: message.into(),
            }),
        }
    }
}

pub async fn handle(
    state: web::Data<AppState>,
    merchant: AuthenticatedMerchant,
    body: web::Json<JsonRpcRequest>,
) -> HttpResponse {
    let req = body.into_inner();

    // A request with no `id` is a JSON-RPC notification: no response body.
    // The only one MCP clients send here is `notifications/initialized`.
    let Some(id) = req.id else {
        return HttpResponse::Accepted().finish();
    };

    let response = match req.method.as_str() {
        "initialize" => JsonRpcResponse::result(id, initialize_result()),
        "tools/list" => JsonRpcResponse::result(id, json!({ "tools": tool_schemas() })),
        "tools/call" => JsonRpcResponse::result(id, call_tool(&state, &merchant, req.params).await),
        other => JsonRpcResponse::error(id, -32601, format!("method not found: {other}")),
    };

    HttpResponse::Ok().json(response)
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "derniere-chance-mcp", "version": env!("CARGO_PKG_VERSION") },
    })
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
    })
}

fn empty_schema() -> Value {
    json!({ "type": "object", "properties": {} })
}

/// Shape shared by `publish_produit` and `update_produit` - the same fields
/// `CreateProductDto` expects, since both tools deserialize straight into it.
fn produit_fields() -> Value {
    json!({
        "nom": { "type": "string" },
        "description": { "type": "string" },
        "prix_initial": {
            "type": "string",
            "description": "prix initial en euros, ex. \"5.90\" (chaîne, pas un nombre)",
        },
        "prix_demarque": {
            "type": "string",
            "description": "prix démarqué en euros, doit être inférieur ou égal à prix_initial",
        },
        "quantite": { "type": "integer", "minimum": 1 },
        "retrait_debut": {
            "type": "string",
            "format": "date-time",
            "description": "début de la fenêtre de retrait, RFC 3339 (ex. 2026-08-23T18:00:00Z)",
        },
        "retrait_fin": {
            "type": "string",
            "format": "date-time",
            "description": "fin de la fenêtre de retrait, RFC 3339, doit être après retrait_debut",
        },
        "photo_url": {
            "type": "string",
            "description": "URL publique de la photo (facultative)",
        },
    })
}

/// Every tool this server exposes. Unlike `tools/list` filtering by role in
/// Elevia (where several account types exist), every MCP client here
/// connects as a single marchand (see `AuthenticatedMerchant`), so
/// discoverability is not role-dependent - `call_tool` still re-derives
/// `merchant.marchand_id` independently for every call, never trusting that
/// listing a tool implies it's safe to run.
fn tool_schemas() -> Vec<Value> {
    vec![
        tool(
            "list_my_produits",
            "Liste tous mes paniers/démarques (tous statuts confondus).",
            empty_schema(),
        ),
        tool(
            "get_my_dashboard",
            "Mon dashboard du jour : nombre de paniers sauvés et chiffre récupéré.",
            empty_schema(),
        ),
        tool(
            "get_my_profile",
            "Mon profil marchand (nom, adresse, catégorie, note, logo).",
            empty_schema(),
        ),
        tool(
            "publish_produit",
            "Publie un nouveau panier en démarque. Notifie automatiquement les \
             consommateurs abonnés à ma boutique.",
            json!({ "type": "object", "properties": produit_fields(), "required": [
                "nom", "description", "prix_initial", "prix_demarque",
                "quantite", "retrait_debut", "retrait_fin",
            ] }),
        ),
        tool(
            "update_produit",
            "Modifie un panier existant qui m'appartient (échoue si le panier \
             appartient à un autre marchand).",
            {
                let mut properties = produit_fields();
                properties["id"] = json!({ "type": "string", "format": "uuid" });
                json!({ "type": "object", "properties": properties, "required": [
                    "id", "nom", "description", "prix_initial", "prix_demarque",
                    "quantite", "retrait_debut", "retrait_fin",
                ] })
            },
        ),
        tool(
            "marquer_ecoule",
            "Marque un de mes paniers comme écoulé (n'apparaît plus dans le catalogue public).",
            json!({
                "type": "object",
                "properties": { "id": { "type": "string", "format": "uuid" } },
                "required": ["id"],
            }),
        ),
        tool(
            "valider_retrait",
            "Valide en boutique le code de retrait présenté par un consommateur \
             (échoue si le code est inconnu, déjà utilisé, ou lié à un panier \
             d'un autre marchand).",
            json!({
                "type": "object",
                "properties": { "code": { "type": "string" } },
                "required": ["code"],
            }),
        ),
        tool(
            "update_my_profile",
            "Met à jour mon profil marchand (nom, adresse, catégorie).",
            json!({
                "type": "object",
                "properties": {
                    "nom": { "type": "string" },
                    "adresse": { "type": "string" },
                    "categorie": { "type": "string" },
                },
                "required": ["nom", "adresse", "categorie"],
            }),
        ),
    ]
}

#[derive(Debug, Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

#[derive(Debug, Deserialize)]
struct IdArgs {
    id: Uuid,
}

#[derive(Debug, Deserialize)]
struct UpdateProduitArgs {
    id: Uuid,
    #[serde(flatten)]
    produit: CreateProductDto,
}

#[derive(Debug, Deserialize)]
struct CodeArgs {
    code: String,
}

async fn call_tool(state: &AppState, merchant: &AuthenticatedMerchant, params: Value) -> Value {
    let params: ToolCallParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(_) => return error_result("invalid tool call params: expected { name, arguments }"),
    };
    let marchand_id = merchant.marchand_id;

    let outcome = match params.name.as_str() {
        "list_my_produits" => to_value(
            state.product_use_cases.list_mine(marchand_id).await,
            product_error_message,
        ),
        "get_my_dashboard" => to_value(
            state.dashboard_use_cases.merchant_today(marchand_id).await,
            dashboard_error_message,
        ),
        "get_my_profile" => to_value(
            state
                .merchant_auth_use_cases
                .get_own_profile(marchand_id)
                .await,
            merchant_auth_error_message,
        ),
        "publish_produit" => match serde_json::from_value::<CreateProductDto>(params.arguments) {
            Ok(dto) => to_value(
                state.product_use_cases.publish(marchand_id, dto).await,
                product_error_message,
            ),
            Err(err) => Err(format!("invalid arguments: {err}")),
        },
        "update_produit" => match serde_json::from_value::<UpdateProduitArgs>(params.arguments) {
            Ok(args) => to_value(
                state
                    .product_use_cases
                    .update(marchand_id, args.id, args.produit)
                    .await,
                product_error_message,
            ),
            Err(err) => Err(format!("invalid arguments: {err}")),
        },
        "marquer_ecoule" => match serde_json::from_value::<IdArgs>(params.arguments) {
            Ok(args) => to_value(
                state
                    .product_use_cases
                    .mark_ecoule(marchand_id, args.id)
                    .await,
                product_error_message,
            ),
            Err(err) => Err(format!("invalid arguments: {err}")),
        },
        "valider_retrait" => match serde_json::from_value::<CodeArgs>(params.arguments) {
            Ok(args) => to_value(
                state
                    .reservation_use_cases
                    .validate_pickup(marchand_id, &args.code)
                    .await,
                reservation_error_message,
            ),
            Err(err) => Err(format!("invalid arguments: {err}")),
        },
        "update_my_profile" => {
            match serde_json::from_value::<UpdateMerchantDto>(params.arguments) {
                Ok(dto) => to_value(
                    state
                        .merchant_auth_use_cases
                        .update_profile(marchand_id, dto)
                        .await,
                    merchant_auth_error_message,
                ),
                Err(err) => Err(format!("invalid arguments: {err}")),
            }
        }
        other => Err(format!("unknown tool: {other}")),
    };

    match outcome {
        Ok(value) => success_result(&value),
        Err(message) => error_result(&message),
    }
}

/// Converts a use case `Result` into a JSON `Value` for the tool result,
/// mapping the error through the same user-facing wording the REST handlers
/// use (see `infrastructure::web::handlers::product_handlers` etc.) -
/// anything not explicitly handled by `to_message` is logged and collapsed
/// to a generic "internal error", exactly like a REST 500.
fn to_value<T: Serialize, E: std::fmt::Debug>(
    result: Result<T, E>,
    to_message: impl Fn(&E) -> Option<String>,
) -> Result<Value, String> {
    match result {
        Ok(value) => serde_json::to_value(value).map_err(|_| "serialization error".to_string()),
        Err(err) => match to_message(&err) {
            Some(message) => Err(message),
            None => {
                tracing::error!(?err, "mcp tool call failed");
                Err("internal error".to_string())
            }
        },
    }
}

fn product_error_message(err: &ProductError) -> Option<String> {
    match err {
        ProductError::InvalidInput(msg) => Some(msg.clone()),
        ProductError::NotFound => Some("produit ou marchand introuvable".to_string()),
        ProductError::Forbidden => Some("ce produit appartient à un autre marchand".to_string()),
        ProductError::Internal(_) => None,
    }
}

fn reservation_error_message(err: &ReservationError) -> Option<String> {
    match err {
        ReservationError::ReservationNotFound => Some("code de retrait introuvable".to_string()),
        ReservationError::Forbidden => Some("ce panier appartient à un autre marchand".to_string()),
        ReservationError::AlreadyRedeemed => Some(err.to_string()),
        ReservationError::ProductNotFound
        | ReservationError::SoldOut
        | ReservationError::CodeGenerationFailed
        | ReservationError::Internal(_) => None,
    }
}

fn dashboard_error_message(_err: &DashboardError) -> Option<String> {
    None
}

fn merchant_auth_error_message(err: &MerchantAuthError) -> Option<String> {
    match err {
        MerchantAuthError::NotFound => Some("marchand introuvable".to_string()),
        _ => None,
    }
}

fn success_result(value: &Value) -> Value {
    json!({
        "content": [{ "type": "text", "text": value.to_string() }],
        "isError": false,
    })
}

fn error_result(message: &str) -> Value {
    json!({
        "content": [{ "type": "text", "text": message }],
        "isError": true,
    })
}
