use sqlx::{Pool, Postgres, pool, postgres};

pub type DbPool = Pool<Postgres>;

pub async fn connect_db(database_url:&str)->DbPool{
    Pool::<Postgres>::connect(database_url)
            .await
            .expect("Failed to connect database!")
}