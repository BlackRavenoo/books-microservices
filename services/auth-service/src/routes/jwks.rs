use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use base64::{Engine as _, engine::general_purpose};
use rsa::{pkcs8::DecodePublicKey, traits::PublicKeyParts, RsaPublicKey};
use std::fs;

use crate::auth::jwt::JwtService;

async fn jwks(jwt_service: web::Data<JwtService>) -> impl Responder {
    match get_jwks(&jwt_service).await {
        Ok(jwks) => HttpResponse::Ok()
            .content_type("application/json")
            .append_header(("Cache-Control", "public, max-age=3600"))
            .json(jwks),
        Err(e) => {
            tracing::error!("Failed to generate JWKS: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn get_jwks(jwt_service: &JwtService) -> anyhow::Result<serde_json::Value> {
    let public_key_pem = fs::read_to_string(&jwt_service.get_public_key_path())?;
    
    let public_key = RsaPublicKey::from_public_key_pem(&public_key_pem)?;
    
    let n = public_key.n();
    let e = public_key.e();
    
    let n_bytes = n.to_bytes_be();
    let e_bytes = e.to_bytes_be();
    
    let n_b64 = general_purpose::URL_SAFE_NO_PAD.encode(n_bytes);
    let e_b64 = general_purpose::URL_SAFE_NO_PAD.encode(e_bytes);
    
    let kid = "default-key-1";
    
    Ok(json!({
        "keys": [
            {
                "kty": "RSA",
                "use": "sig",
                "alg": "RS256",
                "kid": kid,
                "n": n_b64,
                "e": e_b64
            }
        ]
    }))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/.well-known")
            .route("/jwks.json", web::get().to(jwks))
    );
}