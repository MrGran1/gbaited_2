use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use sqlx::MySqlPool;

use crate::{
    auth::AuthUser,
    errors::{AppError, AppResult},
    models::{LeaderboardEntry, Tournament, TournamentAndUser},
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/",                get(list_tournaments).post(create_tournament))
        .route("/:id",             get(get_tournament))
        .route("/:id/join",        post(join_tournament))
        .route("/:id/leaderboard", get(get_leaderboard))
        .route("/:id/members",     get(get_members))
}

// ─── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateTournamentRequest {
    tournament_name: String,
    competition_id:  u64,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

async fn list_tournaments(State(db): State<MySqlPool>) -> AppResult<Json<Vec<Tournament>>> {
    let rows = sqlx::query_as!(
        Tournament,
        "SELECT id, tournament_name, competition_id FROM tournaments ORDER BY id"
    )
    .fetch_all(&db)
    .await?;
    Ok(Json(rows))
}

async fn create_tournament(
    State(db):  State<MySqlPool>,
    _auth:      AuthUser,
    Json(body): Json<CreateTournamentRequest>,
) -> AppResult<(StatusCode, Json<Tournament>)> {
    sqlx::query_scalar!("SELECT id FROM competitions WHERE id = ?", body.competition_id)
        .fetch_optional(&db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Competition {} not found", body.competition_id)))?;

    let result = sqlx::query!(
        "INSERT INTO tournaments (tournament_name, competition_id) VALUES (?, ?)",
        body.tournament_name,
        body.competition_id
    )
    .execute(&db)
    .await?;

    let tournament = sqlx::query_as!(
        Tournament,
        "SELECT id, tournament_name, competition_id FROM tournaments WHERE id = ?",
        result.last_insert_id()
    )
    .fetch_one(&db)
    .await?;

    Ok((StatusCode::CREATED, Json(tournament)))
}

async fn get_tournament(
    State(db): State<MySqlPool>,
    Path(id):  Path<u64>,
) -> AppResult<Json<Tournament>> {
    sqlx::query_as!(
        Tournament,
        "SELECT id, tournament_name, competition_id FROM tournaments WHERE id = ?",
        id
    )
    .fetch_optional(&db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Tournament {} not found", id)))
    .map(Json)
}

async fn join_tournament(
    State(db): State<MySqlPool>,
    auth:      AuthUser,
    Path(id):  Path<u64>,
) -> AppResult<(StatusCode, Json<TournamentAndUser>)> {
    sqlx::query_scalar!("SELECT id FROM tournaments WHERE id = ?", id)
        .fetch_optional(&db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Tournament {} not found", id)))?;

    let already: Option<u64> = sqlx::query_scalar!(
        "SELECT id FROM tournament_and_user WHERE tournament_id = ? AND user_id = ?",
        id,
        auth.user_id
    )
    .fetch_optional(&db)
    .await?;

    if already.is_some() {
        return Err(AppError::Conflict("Already a member of this tournament".into()));
    }

    let result = sqlx::query!(
        "INSERT INTO tournament_and_user (tournament_id, user_id) VALUES (?, ?)",
        id,
        auth.user_id
    )
    .execute(&db)
    .await?;

    let entry = TournamentAndUser {
        id:            result.last_insert_id(),
        tournament_id: id,
        user_id:       auth.user_id,
    };

    Ok((StatusCode::CREATED, Json(entry)))
}

async fn get_leaderboard(
    State(db): State<MySqlPool>,
    Path(id):  Path<u64>,
) -> AppResult<Json<Vec<LeaderboardEntry>>> {
    sqlx::query_scalar!("SELECT id FROM tournaments WHERE id = ?", id)
        .fetch_optional(&db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Tournament {} not found", id)))?;

    let rows = sqlx::query_as!(
        LeaderboardEntry,
        r#"SELECT
               u.id                 AS user_id,
               u.visible_username,
               u.total_nbr_point,
               COUNT(p.id)          AS paris_count
           FROM tournament_and_user tau
           JOIN  users u ON u.id = tau.user_id
           LEFT JOIN paris p ON p.user_id = u.id AND p.tournament_id = tau.tournament_id
           WHERE tau.tournament_id = ?
           GROUP BY u.id, u.visible_username, u.total_nbr_point
           ORDER BY u.total_nbr_point DESC"#,
        id
    )
    .fetch_all(&db)
    .await?;

    Ok(Json(rows))
}

async fn get_members(
    State(db): State<MySqlPool>,
    Path(id):  Path<u64>,
) -> AppResult<Json<Vec<TournamentAndUser>>> {
    let rows = sqlx::query_as!(
        TournamentAndUser,
        "SELECT id, tournament_id, user_id FROM tournament_and_user WHERE tournament_id = ?",
        id
    )
    .fetch_all(&db)
    .await?;
    Ok(Json(rows))
}
