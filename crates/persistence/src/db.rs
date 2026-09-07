use std::time::Duration;

use sqlx::{Pool, Postgres, postgres::{ PgPoolOptions }};

pub type DbPool = Pool<Postgres>;

pub async fn connect_db(database_url:&str)->Result<DbPool , sqlx::Error>{
    PgPoolOptions::new()
        .min_connections(0)
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await
}

pub async fn ping_db(db_pool:&DbPool)->Result<(),sqlx::Error>{
    let _:i32 = sqlx::query_scalar("SELECT 1")
            .fetch_one(db_pool).await?;
    Ok(())
}

