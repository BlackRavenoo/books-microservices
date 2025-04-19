use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use jsonwebtoken::{encode, decode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,         // ID пользователя
    roles: Vec<String>,  // Роли пользователя
    permissions: Vec<String>, // Разрешения
    exp: usize,          // Время истечения токена
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn login(login_req: web::Json<LoginRequest>) -> impl Responder {
    // В реальном приложении проверка должна быть против базы данных
    if login_req.username == "admin" && login_req.password == "password" {
        // В реальном приложении роли и разрешения должны быть загружены из БД
        let roles = vec!["ADMIN".to_string()];
        let permissions = vec![
            "users:read".to_string(),
            "users:write".to_string(),
            "reports:read".to_string(),
            "reports:write".to_string()
        ];
        
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize + 3600; // Токен действителен 1 час
        
        let claims = Claims {
            sub: "1".to_string(), // ID пользователя
            roles,
            permissions,
            exp: expiration,
        };
        
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("secret_key".as_ref()),
        ).unwrap();
        
        HttpResponse::Ok().json(web::Json(serde_json::json!({
            "token": token,
            "token_type": "Bearer",
            "expires_in": 3600
        })))
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

async fn validate_token(token: String) -> impl Responder {
    let token = token.trim_start_matches("Bearer ").to_string();
    
    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret("secret_key".as_ref()),
        &Validation::default(),
    ) {
        Ok(token_data) => {
            HttpResponse::Ok().json(web::Json(token_data.claims))
        }
        Err(_) => {
            HttpResponse::Unauthorized().finish()
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/auth/login", web::post().to(login))
            .route("/auth/validate", web::post().to(validate_token))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}