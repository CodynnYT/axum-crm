// Axum CRM API server.

// Startup: read configuration, connect to PostgreSQL, run migrations,
// create the first admin if needed, and serve HTTP.

mod config;
mod error;
mod handlers;
mod routes;
mod state;
mod auth;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let config = Config::from_env();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    tracing::info!("database is ready");

    auth::create_first_admin(&db, &config).await.expect("Failed to create first admin");

    let state = AppState {
        db,
        jwt_secret: config.jwt_secret.clone(),
    };

    let app = routes::router(state, config.static_dir.as_deref());

    let address = format!("0.0.0.0:{}", config.port);

    let listener = tokio::net::TcpListener::bind(&address).await.expect("could not bind to port");

    tracing::info!("listening on {}", address);

    axum::serve(listener, app).await.expect("could not serve");

}

