use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    user_id: u32,
    username: String,
    iat: u64,
    exp: u64,
}

impl Claims {
    pub fn get_user_id(&self) -> u32 {
        self.user_id
    }

    pub fn get_user_name(&self) -> &str {
        &self.username
    }
}

#[derive(Debug, Clone)]
pub struct JwtService {
    encode_key: EncodingKey,
    decode_key: DecodingKey,
    ttl: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("failed to generate expiration. {0}")]
    ExpGenError(String),
    #[error("failed to encode token. {0}")]
    EncodeError(String),
    #[error("failed to decode token. {0}")]
    DecodeError(String),
}

impl JwtService {
    pub fn new(secret: String, ttl: u32) -> Self {
        JwtService {
            encode_key: EncodingKey::from_secret(secret.as_bytes()),
            decode_key: DecodingKey::from_secret(secret.as_bytes()),
            ttl,
        }
    }

    pub fn generate_token(&self, user_id: u32, username: &str) -> Result<String, JwtError> {
        let header = Header::new(Algorithm::HS512);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| JwtError::ExpGenError(e.to_string()))?
            .as_secs();
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| JwtError::ExpGenError(e.to_string()))?
            .as_secs()
            + self.ttl as u64;
        let claims = Claims {
            user_id,
            username: username.to_owned(),
            iat: now,
            exp,
        };
        encode(&header, &claims, &self.encode_key).map_err(|e| JwtError::EncodeError(e.to_string()))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = Validation::new(Algorithm::HS512);
        validation.set_required_spec_claims(&["exp"]);
        validation.validate_exp = true;
        validation.leeway = 1;
        match decode::<Claims>(&token, &self.decode_key, &validation) {
            Ok(token_data) => Ok(token_data.claims),
            Err(e) => Err(JwtError::DecodeError(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;
    #[test]
    fn test_basic() {
        let jwt_service = JwtService::new("sdlkjsfjl3;dsfkj".into(), 300);
        let _token = jwt_service.generate_token(1, "Tom").unwrap();
    }

    #[test]
    fn test_should_be_valid() {
        let jwt_service = JwtService::new("sdlkjsfjl3;dsfkj".into(), 300);
        let token = jwt_service.generate_token(1, "Tom").unwrap();
        assert!(jwt_service.verify_token(&token).is_ok());
    }

    #[test]
    fn test_should_be_expired() {
        let jwt_service = JwtService::new("sdlkjsfjl3;dsfkj".into(), 1);
        let token = jwt_service.generate_token(1, "Tom").unwrap();
        thread::sleep(Duration::from_secs(3));
        assert!(!jwt_service.verify_token(&token).is_ok());
    }

    #[actix_web::test]
    async fn test_actix_basic() {}
}
