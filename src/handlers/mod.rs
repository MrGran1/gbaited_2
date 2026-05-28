use axum::Router;
use crate::AppState;

mod auth;
mod competitions;
mod matches;
mod paris;
mod teams;
mod tournaments;
mod users;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/auth",         auth::routes())
        .nest("/competitions", competitions::routes())
        .nest("/matches",      matches::routes())
        .nest("/teams",        teams::routes())
        .nest("/tournaments",  tournaments::routes())
        .nest("/paris",        paris::routes())
        .nest("/users",        users::routes())
}
