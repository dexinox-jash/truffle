//! Authentication for WebSocket connections

use jsonwebtoken::{decode, Validation, Algorithm};
use uuid::Uuid;

/// Validate JWT token and extract user info
pub async fn validate_token(token: &str) -> anyhow::Result<(Uuid, Uuid, String)> {
    // In production, use proper secret from environment
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key".to_string());
    
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<crate::message::Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;
    
    let claims = token_data.claims;
    Ok((claims.sub, claims.tenant_id, claims.email))
}
