
use thiserror::Error;

#[derive(Debug,Error)]
pub enum PersistanceError{ 
    #[error("Database connection failed !")]
    Database(#[from] sqlx::Error)
}

