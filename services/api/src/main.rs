
pub mod health;
use std::env;

use dotenv::dotenv;

use crate::health::check_db_status;


#[tokio::main]
pub async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("Failed to get db string!");
    let db  = check_db_status(&database_url).await;
    println!("Database started! {:?}",db);
}
