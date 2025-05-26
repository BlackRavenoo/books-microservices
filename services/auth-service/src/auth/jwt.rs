use std::fs;

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AuthSettings;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
    pub jti: String,
    pub scope: String,
    pub roles: Vec<String>
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    pub access_token_lifetime: Duration,
    public_key_path: String,
}

impl JwtService {
    pub fn new(auth_settings: &AuthSettings) -> anyhow::Result<Self> {
        let private_key = fs::read(&auth_settings.private_key_path)?;
        
        let public_key = fs::read(&auth_settings.public_key_path)?;
        
        Ok(Self {
            encoding_key: EncodingKey::from_rsa_pem(&private_key)?,
            decoding_key: DecodingKey::from_rsa_pem(&public_key)?,
            issuer: auth_settings.issuer.clone(),
            access_token_lifetime: Duration::from_std(auth_settings.access_token_lifetime)?,
            public_key_path: auth_settings.public_key_path.clone()
        })
    }

    pub fn create_access_token(
        &self,
        user_id: i32,
        scope: &str,
        roles: Vec<String>,
    ) -> anyhow::Result<String> {
        let now = Utc::now();
        let expiry = now + self.access_token_lifetime;
        
        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            iss: self.issuer.clone(),
            jti: Uuid::new_v4().to_string(),
            scope: scope.to_string(),
            roles
        };

        let header = Header::new(Algorithm::RS256);
        
        Ok(jsonwebtoken::encode(&header, &claims, &self.encoding_key)?)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, &'static str> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        
        jsonwebtoken::decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    "Token expired"
                }
                _ => "Invalid token"
            })
    }

    pub fn get_public_key_path(&self) -> &str {
        &self.public_key_path
    }
}