use axum::{
    extract::State,
    routing::{get, patch},
    Json, Router,
};
use serde::Deserialize;
use sqlx::MySqlPool;

use crate::{
    auth::AuthUser,
    errors::{AppError, AppResult},
    models::User,
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me).patch(update_me))
}

#[derive(Deserialize)]
struct UpdateMeRequest {
    visible_username: Option<String>,
    password:         Option<String>,
}

async fn get_me(
    State(db): State<MySqlPool>,
    auth:      AuthUser,
) -> AppResult<Json<User>> {
    sqlx::query_as!(
        User,
        "SELECT id, visible_username, password, total_nbr_point
         FROM users WHERE id = ?",
        auth.user_id
    )
    .fetch_optional(&db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))
    .map(Json)
}

async fn update_me(
    State(db):  State<MySqlPool>,
    auth:       AuthUser,
    Json(body): Json<UpdateMeRequest>,
) -> AppResult<Json<User>> {
    if let Some(ref username) = body.visible_username {
        // Check uniqueness
        let exists: Option<String> = sqlx::query_scalar!(
            "SELECT id FROM users WHERE visible_username = ? AND id != ?",
            username,
            auth.user_id
        )
        .fetch_optional(&db)
        .await?;

        if exists.is_some() {
            return Err(AppError::Conflict("Username already taken".into()));
        }

        sqlx::query!(
            "UPDATE users SET visible_username = ? WHERE id = ?",
            username,
            auth.user_id
        )
        .execute(&db)
        .await?;
    }

    if let Some(ref password) = body.password {
        let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
        sqlx::query!(
            "UPDATE users SET password = ? WHERE id = ?",
            hashed,
            auth.user_id
        )
        .execute(&db)
        .await?;
    }

    sqlx::query_as!(
        User,
        "SELECT id, visible_username, password, total_nbr_point
         FROM users WHERE id = ?",
        auth.user_id
    )
    .fetch_optional(&db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))
    .map(Json)
}
