use axum::{
    routing::{get, post, put},
    Router,
};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::handlers::{activities, auth, customers, dashboard, deals, health, leads, reports, users};
use crate::state::AppState;

pub fn router(state: AppState, static_dir: Option<&str>) -> Router {
    let api = Router::new()
        .route("/health", get(health::health))
        .route("/auth/login", post(auth::login))
        .route("/auth/me", get(auth::me).put(auth::update_me))
        .route("/auth/password", put(auth::change_password))
        .route("/dashboard", get(dashboard::summary))
        .route("/leads", get(leads::list).post(leads::create))
        .route("/leads/{id}", get(leads::get_one).put(leads::update).delete(leads::delete))
        .route("/leads/{id}/convert", post(leads::convert))
        .route("/customers", get(customers::list).post(customers::create))
        .route(
            "/customers/{id}",
            get(customers::get_one).put(customers::update).delete(customers::delete),
        )
        .route("/deals", get(deals::list).post(deals::create))
        .route("/deals/{id}", get(deals::get_one).put(deals::update).delete(deals::delete))
        .route("/activities", get(activities::list).post(activities::create))
        .route(
            "/activities/{id}",
            get(activities::get_one).put(activities::update).delete(activities::delete),
        )
        .route("/activities/{id}/complete", post(activities::complete))
        .route("/reports", get(reports::list).post(reports::create))
        .route("/reports/{id}", get(reports::get_one).delete(reports::delete))
        .route("/reports/{id}/csv", get(reports::csv))
        .route("/users", get(users::list).post(users::create))
        .route("/users/{id}", axum::routing::delete(users::delete));

    let mut app = Router::new().nest("/api", api);

    if let Some(dir) = static_dir {
        let index = format!("{dir}/index.html");
        app = app.fallback_service(ServeDir::new(dir).not_found_service(ServeFile::new(index)));
    }

    app
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}