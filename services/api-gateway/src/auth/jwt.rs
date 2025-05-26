use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use moka::future::Cache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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

#[derive(Debug, Deserialize)]
pub struct JwksResponse {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
pub struct Jwk {
    pub kty: String,
    #[serde(rename = "use")]
    pub key_use: Option<String>,
    pub alg: Option<String>,
    pub kid: String,
    pub n: Option<String>,
    pub e: Option<String>,
}

#[derive(Clone)]
pub struct JwtValidator {
    client: Client,
    auth_service_url: String,
    keys_cache: Cache<String, DecodingKey>,
    validation: Validation,
}

impl JwtValidator {
    pub fn new(auth_service_url: String) -> Self {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        
        let keys_cache = Cache::builder()
            .max_capacity(100)
            .time_to_live(Duration::from_secs(3600))
            .time_to_idle(Duration::from_secs(1800))
            .build();

        Self {
            client: Client::new(),
            auth_service_url,
            keys_cache,
            validation,
        }
    }

    pub async fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let header = decode_header(token)
            .map_err(|e| JwtError::InvalidToken(format!("Invalid header: {}", e)))?;

        let kid = header.kid
            .ok_or_else(|| JwtError::InvalidToken("Missing kid in header".to_string()))?;

        let decoding_key = self.keys_cache.get(&kid).await;

        let key = match decoding_key {
            Some(key) => {
                tracing::debug!("Using cached key for kid: {}", kid);
                key
            }
            None => {
                self.refresh_keys().await?;
                
                self.keys_cache.get(&kid).await
                    .ok_or_else(|| JwtError::KeyNotFound(kid.clone()))?
            }
        };

        let token_data = decode::<Claims>(token, &key, &self.validation)
            .map_err(|e| JwtError::InvalidToken(format!("Token validation failed: {}", e)))?;

        Ok(token_data.claims)
    }

    pub async fn refresh_keys(&self) -> Result<(), JwtError> {
        tracing::info!("Refreshing JWT keys from auth service");
        
        let url = format!("{}/.well-known/jwks.json", self.auth_service_url);
        
        let response = self.client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| JwtError::NetworkError(format!("Failed to fetch keys: {}", e)))?;

        if !response.status().is_success() {
            return Err(JwtError::NetworkError(format!(
                "Auth service returned status: {}", 
                response.status()
            )));
        }

        let jwks: JwksResponse = response.json().await
            .map_err(|e| JwtError::ParseError(format!("Failed to parse JWKS response: {}", e)))?;

        let mut processed_keys = 0;
        let mut valid_keys = 0;

        for jwk in jwks.keys {
            processed_keys += 1;
                
            match self.jwk_to_decoding_key(&jwk)? {
                Some(key) => {
                    valid_keys += 1;
                    self.keys_cache.insert(jwk.kid.clone(), key).await;
                    tracing::info!("Successfully loaded key: {}", jwk.kid);
                }
                None => {
                    tracing::warn!("Skipped key: {}", jwk.kid);
                }
            }
        }

        if valid_keys == 0 {
            tracing::warn!("No valid keys found in JWKS response (processed {} keys)", processed_keys);
            return Err(JwtError::NoValidKeys);
        }

        tracing::info!("Successfully refreshed {} out of {} JWT keys", valid_keys, processed_keys);
        Ok(())
    }

    fn jwk_to_decoding_key(&self, jwk: &Jwk) -> Result<Option<DecodingKey>, JwtError> {
        match jwk.kty.as_str() {
            "RSA" => {
                if let Some(key_use) = &jwk.key_use {
                    if key_use != "sig" {
                        return Ok(None);
                    }
                }

                if let Some(alg) = &jwk.alg {
                    if alg != "RS256" {
                        return Ok(None);
                    }
                }

                if let (Some(n), Some(e)) = (&jwk.n, &jwk.e) {
                    let key = DecodingKey::from_rsa_components(n, e)
                        .map_err(|e| JwtError::ParseError(format!("Failed to create RSA key: {}", e)))?;
                    Ok(Some(key))
                } else {
                    Ok(None)
                }
            }
            _ => {
                tracing::warn!("Unsupported key type: {} for kid: {}", jwk.kty, jwk.kid);
                Ok(None)
            }
        }
    }

    pub async fn invalidate_key(&self, kid: &str) {
        self.keys_cache.invalidate(kid).await;
    }

    pub async fn clear_cache(&self) {
        self.keys_cache.invalidate_all();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("No valid keys found")]
    NoValidKeys,
}

impl Claims {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scope.split_whitespace().any(|s| s == scope)
    }

    pub fn get_scopes(&self) -> Vec<&str> {
        self.scope.split_whitespace().collect()
    }

    pub fn is_valid(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.exp > now
    }
}