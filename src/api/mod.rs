use axum::Router;

mod auth;
mod challenges;
mod profile;
mod leaderboard;

pub fn router() -> Router<crate::State> {
    Router::new()
        .nest("/admin", crate::admin::router())
        .nest("/auth", auth::router())
        .nest("/challs", challenges::router())
        .nest("/profile", profile::router())
        .nest("/leaderboard", leaderboard::router())
}
