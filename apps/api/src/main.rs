use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};

use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::str::FromStr;
use std::collections::BTreeSet;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

// Error Handling Architecture
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(err) => {
                eprintln!("Database Error occurred: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database connection failure".to_string())
            }
            AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(serde_json::json!({"error": error_message}));
        (status, body).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub exp: usize,
}

pub struct UserSession {
    pub user_id: uuid::Uuid,
    pub email: Option<String>,
}

fn build_jwt_validation() -> Validation {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.required_spec_claims.insert("exp".to_string());
    validation.required_spec_claims.insert("sub".to_string());
    validation
}

fn build_allowed_origins() -> Result<AllowOrigin, anyhow::Error> {
    let mut origins = BTreeSet::new();
    origins.insert("http://localhost:5173".to_string());
    origins.insert("http://127.0.0.1:5173".to_string());

    if let Ok(configured) = std::env::var("CORS_ALLOWED_ORIGINS") {
        for origin in configured.split(',').map(str::trim).filter(|value| !value.is_empty()) {
            origins.insert(origin.to_string());
        }
    }

    if let Ok(single_origin) = std::env::var("CORS_ALLOWED_ORIGIN") {
        let trimmed = single_origin.trim();
        if !trimmed.is_empty() {
            origins.insert(trimmed.to_string());
        }
    }

    let header_values = origins
        .into_iter()
        .map(|origin| HeaderValue::from_str(&origin))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| anyhow::anyhow!("CORS_ALLOWED_ORIGINS contains an invalid header value"))?;

    Ok(AllowOrigin::list(header_values))
}

impl<S> FromRequestParts<S> for UserSession
where 
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection>
    {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Auth("Missing Authorization Header".to_string()))?;

        let mut auth_parts = auth_header.split_whitespace();
        let scheme = auth_parts.next();
        let token = auth_parts.next();
        let trailing = auth_parts.next();

        if scheme != Some("Bearer") || token.is_none() || trailing.is_some() {
            return Err(AppError::Auth(
                "Authorization header must be exactly: Bearer <token>".to_string(),
            ));
        }

        let token = token.ok_or_else(|| {
            AppError::Auth("Authorization header must be exactly: Bearer <token>".to_string())
        })?;

        let jwt_secret = std::env::var("SUPABASE_JWT_SECRET")
            .map_err(|_| AppError::Auth("Internal authentication system error".to_string()))?;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &build_jwt_validation(),
        )
        .map_err(|_| AppError::Auth("Invalid session token".to_string()))?;

        let user_id = uuid::Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::Auth("Malformed user identification payload".to_string()))?;

        Ok(UserSession {
            user_id,
            email: token_data.claims.email,
        })
    }
}

pub fn sanitize_filename(input: &str) -> String {
    let clean: String = input
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .take(255)
        .collect();

    let clean_no_traversal = clean.replace("..", "");

    if clean_no_traversal.is_empty() || clean_no_traversal == "." {
        format!("unnamed_file_{}", uuid::Uuid::new_v4())
    } else {
        clean_no_traversal
    }
}

#[derive(Clone)]
struct AppState {
    db_pool: PgPool,
}

impl From<AppState> for PgPool {
    fn from(state: AppState) -> Self {
        state.db_pool
    }
}

async fn health_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    // Perform active ping on local postgres database
    sqlx::query("SELECT 1").execute(&state.db_pool).await?;

    Ok(Json(serde_json::json!({
        "status": "healthy",
        "database": "connected"
    })))
}

// Protected resource dummy placeholder (to demonstrate the JWT extractor working)
async fn get_my_quota(session: UserSession, State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    // Query local user profile config
    let record = sqlx::query(
        "SELECT storage_quota_bytes, storage_used_bytes FROM public.profiles WHERE id = $1::uuid",
    )
    .bind(session.user_id.to_string())
    .fetch_one(&state.db_pool)
    .await?;

    let quota_bytes: i64 = record.try_get("storage_quota_bytes")?;
    let used_bytes: i64 = record.try_get("storage_used_bytes")?;

    Ok(Json(serde_json::json!({
        "user_id": session.user_id,
        "quota_bytes": quota_bytes,
        "used_bytes": used_bytes
    })))
}


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Attempt loading environmental configurations if available locally
    let _ = dotenvy::dotenv();

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@127.0.0.1:54322/postgres".to_string());

    // Connect to local database pool
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    let state = AppState { db_pool };

    // Set CORS policies with an explicit allow-list for local development.
    let allowed_origin = build_allowed_origins()?;

    let cors = CorsLayer::new()
        .allow_origin(allowed_origin)
        .allow_headers(Any)
        .allow_methods([axum::http::Method::GET]);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/quota", get(get_my_quota))
        .layer(cors)
        .with_state(state);

    let bind_addr = std::env::var("API_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8081".to_string());
    let addr = SocketAddr::from_str(&bind_addr)
        .map_err(|_| anyhow::anyhow!("API_BIND_ADDR must be a valid socket address (e.g. 127.0.0.1:8081)"))?;
    println!("File Vault API listening safely on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ==========================================
// 7. Security & Business Logic Unit Tests
// ==========================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename_sanitization_alphanumeric() {
        let dirty = "safe-File_123.txt";
        assert_eq!(sanitize_filename(dirty), "safe-File_123.txt");
    }

    #[test]
    fn test_filename_sanitization_path_traversal() {
        let dirty = "../../../etc/passwd";
        // Strips raw slashes and removes dot traversals
        assert_eq!(sanitize_filename(dirty), "etcpasswd");
    }

    #[test]
    fn test_filename_sanitization_edge_case_empty() {
        let dirty = "///";
        let clean = sanitize_filename(dirty);
        assert!(clean.starts_with("unnamed_file_"));
    }

    #[test]
    fn test_filename_sanitization_only_traversal_dots() {
        let dirty = "..";
        let clean = sanitize_filename(dirty);
        assert!(clean.starts_with("unnamed_file_"));
    }

    #[test]
    fn test_filename_sanitization_removes_unsupported_chars() {
        let dirty = "my<>:\"/\\|?*file.txt";
        assert_eq!(sanitize_filename(dirty), "myfile.txt");
    }
}
