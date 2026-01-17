use crate::config::Config;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AuthState {
    config: Config,
    jwks_cache: Arc<RwLock<Option<Jwks>>>,
}

impl AuthState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            jwks_cache: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        if self.config.auth_bypass {
            return Ok(Claims::bypass());
        }

        let header = decode_header(token).map_err(|_| AuthError::InvalidToken)?;
        let kid = header.kid.ok_or(AuthError::InvalidToken)?;
        let jwk = match self.find_jwk(&kid).await? {
            Some(jwk) => jwk,
            None => {
                let jwks = self.get_jwks(true).await?;
                jwks.keys
                    .iter()
                    .find(|key| key.kid == kid)
                    .cloned()
                    .ok_or(AuthError::InvalidToken)?
            }
        };

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.config.auth0_audience.clone()]);
        validation.set_issuer(&[self.config.auth0_issuer.clone()]);

        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|_| AuthError::InvalidToken)?;
        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|_| AuthError::InvalidToken)?;
        Ok(token_data.claims)
    }

    async fn get_jwks(&self, force_refresh: bool) -> Result<Jwks, AuthError> {
        if !force_refresh {
            if let Some(cached) = self.jwks_cache.read().await.clone() {
            return Ok(cached);
            }
        }

        let url = format!("https://{}/.well-known/jwks.json", self.config.auth0_domain);
        let jwks = reqwest::get(url)
            .await
            .map_err(|_| AuthError::Unavailable)?
            .json::<Jwks>()
            .await
            .map_err(|_| AuthError::Unavailable)?;

        *self.jwks_cache.write().await = Some(jwks.clone());
        Ok(jwks)
    }

    async fn find_jwk(&self, kid: &str) -> Result<Option<Jwk>, AuthError> {
        let jwks = self.get_jwks(false).await?;
        Ok(jwks.keys.iter().find(|key| key.kid == kid).cloned())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Claims {
    pub fn bypass() -> Self {
        Self {
            sub: "dev|bypass".to_string(),
            exp: 0,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum AuthError {
    InvalidToken,
    Unavailable,
}

#[derive(Debug, Deserialize, Clone)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize, Clone)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
}
