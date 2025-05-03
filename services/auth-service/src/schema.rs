use bincode::{Decode, Encode};
use chrono::Utc;
use serde::{Deserialize, Serialize};

// Input

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub fingerprint: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub fingerprint: String,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterForm {
    pub name: String,
    pub email: String,
    pub password: String,
    pub password_confirm: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordForm {
    pub email: String,
}

// OAuth

#[derive(Debug, Deserialize)]
pub struct AuthorizationRequest {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub state: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub code_verifier: String,
    pub fingerprint: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub client_id: String,
    pub refresh_token: String,
    pub fingerprint: String
}

#[derive(Debug, Deserialize)]
#[serde(tag = "grant_type")]
pub enum OAuthTokenRequest {
    #[serde(rename = "authorization_code")]
    AuthorizationCode(TokenRequest),
    #[serde(rename = "refresh_token")]
    RefreshToken(RefreshTokenRequest)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Client {
    pub id: String,
    pub name: String,
    pub redirect_uris: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthCode {
    pub code: String,
    pub client_id: String,
    pub user_id: i32,
    pub redirect_uri: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

// Output

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Decode, Encode)]
pub struct RefreshToken {
    pub user_id: i32,
    pub fingerprint: String
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: &'static str,
    pub error_description: &'static str,
}

// sqlx

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password_hash: String,
}