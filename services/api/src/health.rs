use persistence::db::connect_db;
use sqlx::{Pool, Postgres, Error};




pub async fn check_db_status(db_url:&str)-> Result<Pool<Postgres>,Error>{
    connect_db(db_url).await
}