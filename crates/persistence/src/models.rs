use serde::Serialize;



#[derive(Serialize)]
pub struct  HealthStatus {
    pub status:String
}

#[derive(Serialize,Debug)]
pub enum DbError {
    FailedToConnect
}