use std::{env, sync::Arc};

use axum::Router;
use redis::aio::MultiplexedConnection;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

struct AppState {
    db_pool: PgPool,
    redis_conn: MultiplexedConnection,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("failed to get env var");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");

    // tracing_subscriber::fmt().with_target(true).init();
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

fn base62_encode(mut num: u64) -> String {
    if num == 0 {
        return "0".to_string();
    }

    let mut result = String::new();
    while num > 0 {
        let remainder = (num % 62) as usize;
        result.push(ALPHABET[remainder] as char);
        num /= 62;
    }

    result
}

fn base62_decode(encoded: &str) -> Option<u64> {
    let mut result: u64 = 0;
    for c in encoded.chars() {
        let value = match c {
            '0'..='9' => c as u64 - '0' as u64,
            'A'..='Z' => c as u64 - 'A' as u64 + 10,
            'a'..='z' => c as u64 - 'a' as u64 + 36,
            _ => return None,
        };

        result = result.checked_mul(62)?.checked_add(value)?;
    }
    Some(result)
}
