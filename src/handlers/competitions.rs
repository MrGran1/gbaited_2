use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use sqlx::MySqlPool;

use crate::{
    errors::{AppError, AppResult},
    models::{Competition, MatchDetail, MatchRow, Team},
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/",            get(list_competitions))
        .route("/:id",         get(get_competition))
        .route("/:id/matches", get(get_competition_matches))
}

async fn list_competitions(State(db): State<MySqlPool>) -> AppResult<Json<Vec<Competition>>> {
    let rows = sqlx::query_as!(
        Competition,
        "SELECT id, game_name, region FROM competitions ORDER BY id"
    )
    .fetch_all(&db)
    .await?;
    Ok(Json(rows))
}

async fn get_competition(
    State(db): State<MySqlPool>,
    Path(id):  Path<u64>,
) -> AppResult<Json<Competition>> {
    sqlx::query_as!(
        Competition,
        "SELECT id, game_name, region FROM competitions WHERE id = ?",
        id
    )
    .fetch_optional(&db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Competition {} not found", id)))
    .map(Json)
}

async fn get_competition_matches(
    State(db): State<MySqlPool>,
    Path(id):  Path<u64>,
) -> AppResult<Json<Vec<MatchDetail>>> {
    let rows = sqlx::query_as!(
        MatchRow,
        "SELECT id, score, BO as bo, status, competition_id, team_1, team_2, winner \
         FROM matchs WHERE competition_id = ? ORDER BY id",
        id
    )
    .fetch_all(&db)
    .await?;

    let details = enrich_matches(rows, &db).await?;
    Ok(Json(details))
}

// ─── Shared helpers ───────────────────────────────────────────────────────────

pub async fn enrich_matches(rows: Vec<MatchRow>, db: &MySqlPool) -> AppResult<Vec<MatchDetail>> {
    let mut out = Vec::with_capacity(rows.len());
    for m in rows {
        let t1 = fetch_team(m.team_1, db).await?;
        let t2 = fetch_team(m.team_2, db).await?;
        let winner = if m.winner == 0 {
            None
        } else {
            Some(fetch_team(m.winner, db).await?)
        };
        out.push(MatchDetail {
            id:             m.id,
            score:          m.score,
            bo:             m.bo,
            status:         m.status,
            competition_id: m.competition_id,
            team_1:         t1,
            team_2:         t2,
            winner,
        });
    }
    Ok(out)
}

pub async fn fetch_team(id: u64, db: &MySqlPool) -> AppResult<Team> {
    sqlx::query_as!(
        Team,
        "SELECT id, name FROM teams WHERE id = ?",
        id
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Team {} not found", id)))
}
