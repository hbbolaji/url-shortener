use std::{env, sync::Arc};

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use chrono::{DateTime, Utc};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row, postgres::PgPoolOptions, prelude::FromRow};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    db_pool: PgPool,
    redis_conn: MultiplexedConnection,
}

#[derive(Debug, Deserialize)]
struct ShortenPayload {
    url: String,
}

#[derive(Debug, Serialize, FromRow)]
struct ShortenResponse {
    url: String,
    short_code: String,
}

#[derive(Debug, FromRow)]
struct UrlId {
    id: i32,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("failed to get env var");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "my_app=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer()) // => Format logs as human-readable text
        .init();

    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .connect(&database_url)
        .await
        .expect("unable to connect database");

    let redis_client = redis::Client::open(redis_url).unwrap();
    let redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    let state = Arc::new(AppState {
        db_pool,
        redis_conn,
    });

    let app = Router::new()
        .route("/shorten", post(shorten))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to connect to port");

    tracing::info!("Listening on port: 0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("service unavailable")
}

async fn shorten(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ShortenPayload>,
) -> impl IntoResponse {
    let url_res =
        sqlx::query_as::<_, UrlId>("INSERT INTO urls (long_url) VALUES ($1) RETURNING id")
            .bind(payload.url)
            .fetch_one(&state.db_pool)
            .await
            .unwrap();

    let short_code = base62::encode(url_res.id as u128);
    let url = format!("0.0.0.0:3000/{}", short_code);

    Json(ShortenResponse { url, short_code })
}
