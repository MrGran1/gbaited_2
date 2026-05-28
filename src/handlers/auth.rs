use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{
    auth::create_token,
    config::Config,
    errors::{AppError, AppResult},
    models::User,
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login",    post(login))
}

// ─── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RegisterRequest {
    visible_username: String,
    password:         String,
}

#[derive(Deserialize)]
struct LoginRequest {
    visible_username: String,
    password:         String,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user:  User,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

async fn register(
    State(db):     State<MySqlPool>,
    State(config): State<Config>,
    Json(body):    Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Check duplicate username
    let exists: Option<String> = sqlx::query_scalar!(
        "SELECT id FROM users WHERE visible_username = ?",
        body.visible_username
    )
    .fetch_optional(&db)
    .await?;

    if exists.is_some() {
        return Err(AppError::Conflict("Username already taken".into()));
    }

    let hashed = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST)?;
    let user_id = Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO users (id, visible_username, password, total_nbr_point) VALUES (?, ?, ?, 0)",
        user_id,
        body.visible_username,
        hashed,
    )
    .execute(&db)
    .await?;

    let user = sqlx::query_as!(
        User,
        "SELECT id, visible_username, password, total_nbr_point FROM users WHERE id = ?",
        user_id
    )
    .fetch_one(&db)
    .await?;

    let token = create_token(&user_id, &config)?;
    Ok(Json(AuthResponse { token, user }))
}

async fn login(
    State(db):     State<MySqlPool>,
    State(config): State<Config>,
    Json(body):    Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, visible_username, password, total_nbr_point FROM users WHERE visible_username = ?",
        body.visible_username
    )
    .fetch_optional(&db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".into()))?;

    let valid = bcrypt::verify(&body.password, &user.password)
        .map_err(|_| AppError::Unauthorized("Invalid credentials".into()))?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }

    let token = create_token(&user.id, &config)?;
    Ok(Json(AuthResponse { token, user }))
}
