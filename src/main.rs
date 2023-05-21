extern crate core;

mod models;
mod router;

use router::create_router;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use dotenv::dotenv;
use sea_orm::{DatabaseConnection, SqlxPostgresConnector};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    postgres: DatabaseConnection,
    key: Key,
    domain: String,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();
    let pgpool = PgPool::connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .expect("Failed to connect to Postgres.");

    let conn = SqlxPostgresConnector::from_sqlx_postgres_pool(pgpool.clone());

    // sqlx::migrate!()
    //     .run(&pgpool)
    //     .await
    //     .expect("Failed to run migrations!");

    //create AppState
    let state = AppState {
        postgres: conn,
        key: Key::generate(),
        domain: std::env::var("DOMAIN").expect("DOMAIN must be set"),
    };
    // build our application with a single route
    let app = create_router(state);

    println!("Server running on localhost:3000");
    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
