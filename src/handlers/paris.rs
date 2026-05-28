use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use sqlx::MySqlPool;

use crate::{
    auth::AuthUser,
    errors::{AppError, AppResult},
    models::Pari,
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/",                get(list_my_paris).post(create_pari))
        .route("/:id",             put(update_pari))
        .route("/match/:match_id", get(paris_for_match))
}

// ─── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CreatePariRequest {
    prediction:    String,
    match_id:      u64,
    tournament_id: u64,
}

#[derive(Deserialize)]
struct UpdatePariRequest {
    prediction: String,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

async fn list_my_paris(
    State(db): State<MySqlPool>,
    auth:      AuthUser,
) -> AppResult<Json<Vec<Pari>>> {
    let rows = sqlx::query_as!(
        Pari,
        "SELECT id, prediction, match_id, user_id, tournament_id \
         FROM paris WHERE user_id = ? ORDER BY id",
        auth.user_id
    )
    .fetch_all(&db)
    .await?;
    Ok(Json(rows))
}

async fn create_pari(
    State(db):  State<MySqlPool>,
    auth:       AuthUser,
    Json(body): Json<CreatePariRequest>,
) -> AppResult<(StatusCode, Json<Pari>)> {
    // Verify match exists and is not finished
    let status: Option<String> = sqlx::query_scalar!(
        "SELECT status FROM matchs WHERE id = ?",
        body.match_id
    )
    .fetch_optional(&db)
    .await?;

    match status.as_deref() {
        None => return Err(AppError::NotFound(format!("Match {} not found", body.match_id))),
        Some("finished") | Some("completed") =>
            return Err(AppError::BadRequest("Cannot bet on a finished match".into())),
        _ => {}
    }

    // Verify user is in the tournament
    let member: Option<u64> = sqlx::query_scalar!(
        "SELECT id FROM tournament_and_user WHERE tournament_id = ? AND user_id = ?",
        body.tournament_id,
        auth.user_id
    )
    .fetch_optional(&db)
    .await?;

    if member.is_none() {
        return Err(AppError::BadRequest(
            "You must join the tournament before placing a bet".into(),
        ));
    }

    // No duplicate bet per match / user / tournament
    let dup: Option<u64> = sqlx::query_scalar!(
        "SELECT id FROM paris WHERE match_id = ? AND user_id = ? AND tournament_id = ?",
        body.match_id,
        auth.user_id,
        body.tournament_id
    )
    .fetch_optional(&db)
    .await?;

    if dup.is_some() {
        return Err(AppError::Conflict(
            "You already placed a bet on this match in this tournament".into(),
        ));
    }

    let result = sqlx::query!(
        "INSERT INTO paris (prediction, match_id, user_id, tournament_id) VALUES (?, ?, ?, ?)",
        body.prediction,
        body.match_id,
        auth.user_id,
        body.tournament_id,
    )
    .execute(&db)
    .await?;

    let pari = sqlx::query_as!(
        Pari,
        "SELECT id, prediction, match_id, user_id, tournament_id FROM paris WHERE id = ?",
        result.last_insert_id()
    )
    .fetch_one(&db)
    .await?;

    Ok((StatusCode::CREATED, Json(pari)))
}

async fn update_pari(
    State(db):  State<MySqlPool>,
    auth:       AuthUser,
    Path(id):   Path<u64>,
    Json(body): Json<UpdatePariRequest>,
) -> AppResult<Json<Pari>> {
    let pari = sqlx::query_as!(
        Pari,
        "SELECT id, prediction, match_id, user_id, tournament_id FROM paris WHERE id = ?",
        id
    )
    .fetch_optional(&db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Pari {} not found", id)))?;

    if pari.user_id != auth.user_id {
        return Err(AppError::Unauthorized("Not your bet".into()));
    }

    // Ensure match still open
    let status: String = sqlx::query_scalar!(
        "SELECT status FROM matchs WHERE id = ?",
        pari.match_id
    )
    .fetch_one(&db)
    .await?;

    if status == "finished" || status == "completed" {
        return Err(AppError::BadRequest(
            "Cannot update bet on a finished match".into(),
        ));
    }

    sqlx::query!(
        "UPDATE paris SET prediction = ? WHERE id = ?",
        body.prediction,
        id
    )
    .execute(&db)
    .await?;

    let updated = sqlx::query_as!(
        Pari,
        "SELECT id, prediction, match_id, user_id, tournament_id FROM paris WHERE id = ?",
        id
    )
    .fetch_one(&db)
    .await?;

    Ok(Json(updated))
}

async fn paris_for_match(
    State(db):      State<MySqlPool>,
    _auth:          AuthUser,
    Path(match_id): Path<u64>,
) -> AppResult<Json<Vec<Pari>>> {
    let rows = sqlx::query_as!(
        Pari,
        "SELECT id, prediction, match_id, user_id, tournament_id \
         FROM paris WHERE match_id = ? ORDER BY id",
        match_id
    )
    .fetch_all(&db)
    .await?;
    Ok(Json(rows))
}
