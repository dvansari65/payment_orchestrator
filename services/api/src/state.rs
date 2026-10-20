use persistence::db::DbPool;

/// Shared by every handler. Cloned per request, so keep it cheap:
/// the pool is already a handle, not a copy of the connections.
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
}

impl AppState {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }
}
