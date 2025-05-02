use actix_session::Session;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use urlencoding::encode;

use crate::{auth::{client_store::ClientStore, code_store::CodeStore, jwt::JwtService, pkce, token_store::TokenStore}, schema::{AuthorizationRequest, ErrorResponse, RefreshToken, RefreshTokenRequest, TokenRequest}, services::user::UserService};

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

pub async fn authorize(
    req: HttpRequest,
    query: web::Query<AuthorizationRequest>,
    client_store: web::Data<ClientStore>,
    code_store: web::Data<CodeStore>,
    session: Session,
) -> impl Responder {
    let query = query.into_inner();
    if query.response_type != "code" {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "invalid_request",
            error_description: "response_type must be 'code'",
        });
    }
    
    let client = match client_store.get_client(&query.client_id).await {
        Ok(Some(client)) => client,
        Ok(None) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "invalid_client",
                error_description: "Client not found",
            });
        },
        Err(e) => {
            tracing::error!("Failed to fetch client: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if !client.redirect_uris.contains(&query.redirect_uri) {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "invalid_request",
            error_description: "Invalid redirect_uri",
        });
    }

    let user_id: Option<i32> = session.get("user_id").unwrap_or(None);
    
    if user_id.is_none() {
        let return_to = format!("/oauth/authorize?{}", req.query_string());
        session.insert("return_to", return_to).unwrap();
        
        return HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish();
    }
    
    let code = match code_store.create_code(
        query.client_id,
        user_id.unwrap(),
        query.redirect_uri.clone(),
        query.code_challenge,
        query.code_challenge_method,
    ).await {
        Ok(code) => code,
        Err(e) => {
            tracing::error!("Failed to create authorization code: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    let mut redirect_url = format!("{}?code={}", query.redirect_uri, encode(&code));
    
    if let Some(state) = &query.state {
        redirect_url.push_str(&format!("&state={}", encode(state)));
    }
    
    HttpResponse::Found()
        .append_header(("Location", redirect_url))
        .finish()
}

pub async fn exchange_token(
    req: web::Json<TokenRequest>,
    code_store: web::Data<CodeStore>,
    token_store: web::Data<TokenStore>,
    jwt_service: web::Data<JwtService>,
    client_store: web::Data<ClientStore>,
    user_service: web::Data<UserService>,
) -> impl Responder {
    let req = req.into_inner();

    if req.grant_type != "authorization_code" {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "unsupported_grant_type",
            error_description: "grant_type must be 'authorization_code'",
        });
    }
    
    let client_exists = match client_store.client_exists(&req.client_id).await {
        Ok(exists) => exists,
        Err(e) => {
            tracing::error!("Failed to check client: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    if !client_exists {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "invalid_client",
            error_description: "Client not found",
        });
    }
    
    let auth_code = match code_store.consume_code(&req.code).await {
        Ok(code) => code,
        Err(e) => {
            tracing::warn!("Failed to consume authorization code: {:?}", e);
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "invalid_grant",
                error_description: "Invalid or expired authorization code",
            });
        }
    };
    
    if auth_code.client_id != req.client_id {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "invalid_grant",
            error_description: "client_id mismatch",
        });
    }
    
    if auth_code.redirect_uri != req.redirect_uri {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "invalid_grant",
            error_description: "redirect_uri mismatch",
        });
    }
    
    if let Err(err) = pkce::verify_code_challenge(
        &req.code_verifier, 
        &auth_code.code_challenge, 
        &auth_code.code_challenge_method
    ) {
        tracing::warn!("PKCE verification failed: {:?}", err);
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "invalid_grant",
            error_description: "PKCE verification failed",
        });
    }

    let roles = match user_service.get_user_roles(auth_code.user_id).await {
        Ok(roles) => roles,
        Err(e) => {
            tracing::error!("Failed to fetch user roles: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let access_token = match jwt_service.create_access_token(
        auth_code.user_id, 
        "",
        roles,
    ) {
        Ok(token) => token,
        Err(_) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    let refresh_token = match token_store.generate_refresh_token(RefreshToken {
        user_id: auth_code.user_id,
        fingerprint: req.fingerprint,
    }).await {
        Ok(token) => token,
        Err(_) => {
            tracing::error!("Failed to generate refresh token");
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    HttpResponse::Ok().json(crate::schema::TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: jwt_service.access_token_lifetime.num_seconds(),
        refresh_token,
    })
}

pub async fn refresh_token(
    req: web::Json<RefreshTokenRequest>,
    token_store: web::Data<TokenStore>,
    jwt_service: web::Data<JwtService>,
    client_store: web::Data<ClientStore>,
    user_service: web::Data<UserService>,
) -> impl Responder {
    let req = req.into_inner();

    if req.grant_type != "refresh_token" {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "invalid_request",
            error_description: "grant_type must be 'refresh_token'",
        });
    }
    
    let client_exists = match client_store.client_exists(&req.client_id).await {
        Ok(exists) => exists,
        Err(e) => {
            tracing::error!("Failed to check client: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    if !client_exists {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "invalid_client",
            error_description: "Client not found",
        });
    }
    
    let refresh_data = match token_store.validate_refresh_token(
        &req.refresh_token,
        &req.fingerprint
    ).await {
        Ok(data) => data,
        Err(e) => {
            tracing::warn!("Invalid refresh token: {:?}", e);
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "invalid_grant",
                error_description: "Invalid refresh token",
            });
        }
    };
    
    let roles = match user_service.get_user_roles(refresh_data.user_id).await {
        Ok(roles) => roles,
        Err(e) => {
            tracing::error!("Failed to fetch user roles: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    let access_token = match jwt_service.create_access_token(
        refresh_data.user_id, 
        "",
        roles,
    ) {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Failed to create access token: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    let refresh_token = match token_store.rotate_refresh_token(
        &req.refresh_token,
        RefreshToken {
            user_id: refresh_data.user_id,
            fingerprint: req.fingerprint,
        }
    ).await {
        Ok(new_token) => new_token,
        Err(e) => {
            tracing::error!("Failed to rotate refresh token: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    HttpResponse::Ok().json(crate::schema::TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: jwt_service.access_token_lifetime.num_seconds(),
        refresh_token,
    })
}

pub async fn me(
    auth: BearerAuth,
    jwt_service: web::Data<JwtService>,
    user_service: web::Data<UserService>,
) -> impl Responder {
    let token = auth.token();
    
    let claims = match jwt_service.validate_token(token) {
        Ok(claims) => claims,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "invalid_token",
                error_description: &e,
            });
        }
    };
    
    let user_id = claims.sub.parse::<i32>().unwrap_or_default();
    
    let roles = match user_service.get_user_roles(user_id).await {
        Ok(roles) => roles,
        Err(e) => {
            tracing::error!("Failed to fetch user roles: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    let username = match user_service.get_username(user_id).await {
        Ok(username) => username,
        Err(e) => {
            tracing::error!("Failed to get username: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        },
    };
    
    HttpResponse::Ok().json(serde_json::json!({
        "id": user_id,
        "username": username,
        "avatar_url": "", // TODO
        "roles": roles
    }))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/oauth")
            .route("/authorize", web::get().to(authorize))
            .route("/token", web::post().to(exchange_token))
            .route("/verify", web::post().to(verify_token))
            .route("/refresh", web::post().to(refresh_token))
            .route("/me", web::get().to(me))
    );
}