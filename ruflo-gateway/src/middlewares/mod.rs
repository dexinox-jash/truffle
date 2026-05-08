//! HTTP middlewares

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::{StatusCode, header::AUTHORIZATION},
};
use std::sync::Arc;
use jsonwebtoken::{decode, Validation, Algorithm};

use crate::{
    models::auth::Claims,
    models::user::CurrentUser,
    services::ServiceRegistry,
};

/// Authentication middleware
pub async fn auth_middleware(
    State(services): State<Arc<ServiceRegistry>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate token
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(services.config.jwt_secret.as_bytes()),
        &validation,
    ).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let claims = token_data.claims;

    // Create current user and add to request extensions
    let current_user = CurrentUser {
        id: claims.sub,
        tenant_id: claims.tenant_id,
        email: claims.email,
        role: claims.role,
    };

    request.extensions_mut().insert(current_user);

    Ok(next.run(request).await)
}
