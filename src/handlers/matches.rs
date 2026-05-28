use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;
use sqlx::MySqlPool;

use crate::{
    auth::AuthUser,
    errors::{AppError, AppResult},
    models::{MatchDetail, MatchRow},
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/",    get(list_matches))
        .route("/:id", get(get_match).put(update_match))
}

async fn list_matches(State(db): State<MySqlPool>) -> AppResult<Json<Vec<MatchDetail>>> {
    let rows = sqlx::query_as!(
        MatchRow,
        "SELECT id, score, BO as bo, status, competition_id, team_1, team_2, winner \
         FROM matchs ORDER BY id"
    )
    .fetch_all(&db)
    .await?;
    let details = super::competitions::enrich_matches(rows, &db).await?;
    Ok(Json(details))
}

async fn get_match(
    State(db): State<MySqlPool>,
    Path(id):  Path<u64>,
) -> AppResult<Json<MatchDetail>> {
    let row = sqlx::query_as!(
        MatchRow,
        "SELECT id, score, BO as bo, status, competition_id, team_1, team_2, winner \
         FROM matchs WHERE id = ?",
        id
    )
    .fetch_optional(&db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Match {} not found", id)))?;

    let mut details = super::competitions::enrich_matches(vec![row], &db).await?;
    Ok(Json(details.remove(0)))
}

#[derive(Deserialize)]
struct UpdateMatchRequest {
    score:  Option<String>,
    status: Option<String>,
    winner: Option<u64>,
}

async fn update_match(
    State(db):  State<MySqlPool>,
    _auth:      AuthUser,
    Path(id):   Path<u64>,
    Json(body): Json<UpdateMatchRequest>,
) -> AppResult<Json<MatchDetail>> {
    // Ensure match exists
    sqlx::query_scalar!("SELECT id FROM matchs WHERE id = ?", id)
        .fetch_optional(&db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Match {} not found", id)))?;

    if let Some(ref score) = body.score {
        sqlx::query!("UPDATE matchs SET score = ? WHERE id = ?", score, id)
            .execute(&db)
            .await?;
    }
    if let Some(ref status) = body.status {
        sqlx::query!("UPDATE matchs SET status = ? WHERE id = ?", status, id)
            .execute(&db)
            .await?;
    }
    if let Some(winner) = body.winner {
        sqlx::query!("UPDATE matchs SET winner = ? WHERE id = ?", winner, id)
            .execute(&db)
            .await?;
    }

    let row = sqlx::query_as!(
        MatchRow,
        "SELECT id, score, BO as bo, status, competition_id, team_1, team_2, winner \
         FROM matchs WHERE id = ?",
        id
    )
    .fetch_one(&db)
    .await?;

    let mut details = super::competitions::enrich_matches(vec![row], &db).await?;
    Ok(Json(details.remove(0)))
}
