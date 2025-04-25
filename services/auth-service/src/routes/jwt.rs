use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use sqlx::PgPool;

use crate::{auth::jwt::JwtService, schema::{LoginRequest, TokenResponse, UserId}};

pub async fn login(
    credentials: web::Json<LoginRequest>,
    pool: web::Data<PgPool>,
    jwt_service: web::Data<JwtService>
) -> impl Responder {
    let user_id = match sqlx::query_as!(
        UserId,
        "SELECT id FROM users WHERE email = $1 AND password_hash = $2",
        credentials.email,
        credentials.password
    )
    .fetch_optional(pool.as_ref())
    .await {
        Ok(id) => match id {
            Some(id) => id,
            None => {
                return HttpResponse::Unauthorized().finish();
            },
        },
        Err(e) => {
            tracing::error!("Failed to select user id: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        },
    }.id;

    let access_token = match jwt_service.create_access_token(user_id, "") {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Failed to create access token: {:?}", e);
            return HttpResponse::InternalServerError().finish()
            
        },
    };

    let refresh_token = jwt_service.create_refresh_token();

    let resp = TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_owned(),
    };

    HttpResponse::Ok().json(resp)
}

pub async fn refresh_token(
    jwt_service: web::Data<JwtService>,
    auth: BearerAuth
) -> impl Responder {
    let token = auth.token();
    
    match jwt_service.validate_token(&token) {
        Ok(claims) => {
            let user_id = match &claims.sub.parse::<i32>() {
                Ok(id) => *id,
                Err(e) => {
                    tracing::error!("Could not parse user_id from sub: {:?}", e);
                    return HttpResponse::InternalServerError().finish()
                },
            };

            let access_token = match jwt_service.create_access_token(user_id, &claims.scope) {
                Ok(token) => token,
                Err(e) => {
                    tracing::error!("Failed to create access token: {:?}", e);
                    return HttpResponse::InternalServerError().body("Could not generate token")
                },
            };

            let refresh_token = jwt_service.create_refresh_token();

            let resp = TokenResponse {
                access_token,
                refresh_token,
                token_type: "Bearer".to_owned(),
            };

            HttpResponse::Ok().json(resp)
        }
        Err(e) => HttpResponse::Unauthorized().body(e),
    }
}

async fn verify_token(
    jwt_service: web::Data<JwtService>,
    auth: BearerAuth
) -> impl Responder {
    let token = auth.token();

    match jwt_service.validate_token(&token) {
        Ok(claims) => HttpResponse::Ok().json(claims),
        Err(e) => HttpResponse::Unauthorized().body(e),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh_token))
            .route("/verify", web::post().to(verify_token))
    );
}