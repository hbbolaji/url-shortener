use std::{env, sync::Arc};

use axum::Router;
use redis::aio::MultiplexedConnection;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::net::TcpListener;

struct AppState {
    db_pool: PgPool,
    redis_conn: MultiplexedConnection,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("failed to get env var");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");

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

    let app = Router::new().with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to connect to port");

    axum::serve(listener, app)
        .await
        .expect("service unavailable")
}
