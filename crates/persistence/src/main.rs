
use std::env;
use dotenv::dotenv;

use crate::{db::connect_db, models::{DbError}};

pub mod db;
pub mod models;
pub mod handlers;

#[tokio::main]
async fn main()->Result<(),DbError> {
    dotenv().ok();
    tracing_subscriber::fmt()
    .with_env_filter("rust_rest_api=debug,tower_http=debug")
    .init();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set!!");
    connect_db(&database_url).await;
    Ok(())
}
