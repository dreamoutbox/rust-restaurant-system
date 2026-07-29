use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use crate::{
    config::Config,
    error::AppError,
    models::user::{Claims, UserRole},
};

pub const COOKIE_NAME: &str = "restaurant_token";

pub fn generate_jwt(user_id: Uuid, username: &str, role: &str, config: &Config) -> Result<String, AppError> {
    let now = chrono::Utc::now().timestamp() as usize;
    let exp = now + (config.jwt_expiry_hours as usize * 3600);

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        role: role.to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("JWT generation failed: {}", e)))
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|_| AppError::Auth("Invalid or expired authentication token".to_string()))?;

    Ok(token_data.claims)
}

// Axum Extractor for Authenticated User Claims
pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Config: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = Config::from_ref(state);

        // Try extracting from Cookie first
        let token = parts
            .headers
            .get(header::COOKIE)
            .and_then(|value| value.to_str().ok())
            .and_then(|cookie_str| {
                cookie_str
                    .split(';')
                    .find_map(|cookie| {
                        let mut parts = cookie.trim().splitn(2, '=');
                        let name = parts.next()?;
                        let val = parts.next()?;
                        if name == COOKIE_NAME {
                            Some(val.to_string())
                        } else {
                            None
                        }
                    })
            })
            // Fallback to Bearer token in Authorization header
            .or_else(|| {
                parts
                    .headers
                    .get(header::AUTHORIZATION)
                    .and_then(|value| value.to_str().ok())
                    .and_then(|auth_str| {
                        if auth_str.starts_with("Bearer ") {
                            Some(auth_str[7..].to_string())
                        } else {
                            None
                        }
                    })
            });

        let token = token.ok_or_else(|| AppError::Auth("Missing authentication token".to_string()))?;
        let claims = verify_jwt(&token, &config.jwt_secret)?;

        Ok(AuthUser(claims))
    }
}

// Extractor for specific roles
pub fn require_role(claims: &Claims, allowed_roles: &[UserRole]) -> Result<(), AppError> {
    let user_role_str = &claims.role;
    let allowed = allowed_roles.iter().any(|r| r.to_string() == *user_role_str);

    if !allowed {
        return Err(AppError::Forbidden(format!(
            "Role '{}' is not authorized to access this resource",
            user_role_str
        )));
    }

    Ok(())
}
