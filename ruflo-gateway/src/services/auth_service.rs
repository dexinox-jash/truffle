//! Authentication service

use std::sync::Arc;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, Algorithm};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};

use crate::{
    config::AppConfig,
    error::ApiError,
    models::auth::{AuthResponse, AuthUser, Claims, LoginRequest, RegisterRequest},
};

pub struct AuthService {
    db: PgPool,
    config: Arc<AppConfig>,
}

impl AuthService {
    pub fn new(db: PgPool, config: Arc<AppConfig>) -> Self {
        Self { db, config }
    }

    /// Register a new user
    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, ApiError> {
        // Check if email exists
        let existing: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM users WHERE email = $1 AND deleted_at IS NULL"
        )
        .bind(&req.email)
        .fetch_optional(&self.db)
        .await?;

        if existing.is_some() {
            return Err(ApiError::BadRequest("Email already registered".to_string()));
        }

        // Hash password
        let password_hash = hash(&req.password, DEFAULT_COST)
            .map_err(|_| ApiError::InternalError)?;

        // Create tenant if not provided
        let tenant_id = if let Some(tenant_name) = req.tenant_name {
            // Create new tenant
            let tenant: (Uuid,) = sqlx::query_as(
                "INSERT INTO tenants (name, slug, plan) VALUES ($1, $2, 'free') RETURNING id"
            )
            .bind(&tenant_name)
            .bind(tenant_name.to_lowercase().replace(" ", "-"))
            .fetch_one(&self.db)
            .await?;
            tenant.0
        } else {
            // Use system tenant (for individual users)
            Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
        };

        // Create user
        let user: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO users (tenant_id, email, password_hash, first_name, last_name, role)
            VALUES ($1, $2, $3, $4, $5, 'owner')
            RETURNING id
            "#
        )
        .bind(tenant_id)
        .bind(&req.email)
        .bind(&password_hash)
        .bind(&req.first_name)
        .bind(&req.last_name)
        .fetch_one(&self.db)
        .await?;

        let user_id = user.0;

        // Generate tokens
        let tokens = self.generate_tokens(user_id, tenant_id, &req.email, "owner").await?;

        Ok(AuthResponse {
            access_token: tokens.0,
            refresh_token: tokens.1,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: AuthUser {
                id: user_id,
                email: req.email,
                first_name: req.first_name,
                last_name: req.last_name,
                tenant_id,
                role: "owner".to_string(),
            },
        })
    }

    /// Login user
    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, ApiError> {
        // Find user
        let user: (Uuid, Uuid, String, String, String, String) = sqlx::query_as(
            r#"
            SELECT id, tenant_id, email, password_hash, first_name, last_name
            FROM users
            WHERE email = $1 AND deleted_at IS NULL
            "#
        )
        .bind(&req.email)
        .fetch_optional(&self.db)
        .await?
        .ok_or(ApiError::Unauthorized)?;

        let (user_id, tenant_id, email, password_hash, first_name, last_name) = user;

        // Verify password
        let valid = verify(&req.password, &password_hash)
            .map_err(|_| ApiError::InternalError)?;

        if !valid {
            return Err(ApiError::Unauthorized);
        }

        // Update last login
        sqlx::query(
            "UPDATE users SET last_login_at = NOW() WHERE id = $1"
        )
        .bind(user_id)
        .execute(&self.db)
        .await?;

        // Generate tokens
        let tokens = self.generate_tokens(user_id, tenant_id, &email, "owner").await?;

        Ok(AuthResponse {
            access_token: tokens.0,
            refresh_token: tokens.1,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: AuthUser {
                id: user_id,
                email,
                first_name,
                last_name,
                tenant_id,
                role: "owner".to_string(),
            },
        })
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse, ApiError> {
        // Decode and validate refresh token
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(
            refresh_token,
            &jsonwebtoken::DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &validation,
        ).map_err(|_| ApiError::Unauthorized)?;

        let claims = token_data.claims;

        // Get user info
        let user: (String, String, String) = sqlx::query_as(
            "SELECT email, first_name, last_name FROM users WHERE id = $1"
        )
        .bind(claims.sub)
        .fetch_optional(&self.db)
        .await?
        .ok_or(ApiError::Unauthorized)?;

        // Generate new tokens
        let tokens = self.generate_tokens(claims.sub, claims.tenant_id, &user.0, &claims.role).await?;

        Ok(AuthResponse {
            access_token: tokens.0,
            refresh_token: tokens.1,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: AuthUser {
                id: claims.sub,
                email: user.0,
                first_name: user.1,
                last_name: user.2,
                tenant_id: claims.tenant_id,
                role: claims.role,
            },
        })
    }

    /// Generate JWT tokens
    async fn generate_tokens(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        email: &str,
        role: &str,
    ) -> Result<(String, String), ApiError> {
        let now = Utc::now().timestamp();
        
        // Access token (1 hour)
        let access_claims = Claims {
            sub: user_id,
            tenant_id,
            email: email.to_string(),
            role: role.to_string(),
            exp: now + 3600,
            iat: now,
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &jsonwebtoken::EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        ).map_err(|_| ApiError::InternalError)?;

        // Refresh token (7 days)
        let refresh_claims = Claims {
            sub: user_id,
            tenant_id,
            email: email.to_string(),
            role: role.to_string(),
            exp: now + 7 * 24 * 3600,
            iat: now,
        };

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &jsonwebtoken::EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        ).map_err(|_| ApiError::InternalError)?;

        Ok((access_token, refresh_token))
    }
}
