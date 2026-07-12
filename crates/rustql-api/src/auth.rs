use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
}

pub struct Auth {
    secret: String,
}

impl Auth {
    pub fn new(secret: &str) -> Self {
        Auth {
            secret: secret.to_string(),
        }
    }

    pub fn generate_token(&self, user_id: &str, email: &str) -> String {
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .unwrap()
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: expiration,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .unwrap()
    }

    pub fn verify_token(&self, token: &str) -> Option<Claims> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .ok()
    }

    pub fn hash_password(password: &str) -> String {
        hash(password, DEFAULT_COST).unwrap()
    }

    pub fn verify_password(password: &str, hash: &str) -> bool {
        verify(password, hash).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let auth = Auth::new("test_secret");
        let token = auth.generate_token("1", "test@test.com");
        assert!(!token.is_empty());
    }

    #[test]
    fn test_token_verification() {
        let auth = Auth::new("test_secret");
        let token = auth.generate_token("1", "test@test.com");
        let claims = auth.verify_token(&token);
        assert!(claims.is_some());
        assert_eq!(claims.unwrap().email, "test@test.com");
    }

    #[test]
    fn test_password_hashing() {
        let hash = Auth::hash_password("password123");
        assert!(Auth::verify_password("password123", &hash));
        assert!(!Auth::verify_password("wrongpassword", &hash));
    }
}