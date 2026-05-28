use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use sqlx::MySqlPool;

use crate::{errors::AppResult, models::Team, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/",    get(list_teams))
        .route("/:id", get(get_team))
}

async fn list_teams(State(db): State<MySqlPool>) -> AppResult<Json<Vec<Team>>> {
    let rows = sqlx::query_as!(Team, "SELECT id, name FROM teams ORDER BY name")
        .fetch_all(&db)
        .await?;
    Ok(Json(rows))
}

async fn get_team(
    State(db): State<MySqlPool>,
    Path(id):  Path<u64>,
) -> AppResult<Json<Team>> {
    sqlx::query_as!(Team, "SELECT id, name FROM teams WHERE id = ?", id)
        .fetch_optional(&db)
        .await?
        .ok_or_else(|| crate::errors::AppError::NotFound(format!("Team {} not found", id)))
        .map(Json)
}
