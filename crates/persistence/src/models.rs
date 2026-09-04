use serde::Serialize;
use uuid::Uuid;



#[derive(Serialize)]
pub struct  HealthStatus {
    pub status:String
}

#[derive(Serialize,Debug)]
pub enum DbError {
    FailedToConnect
}

// TODO: need to add more payment statuses
#[derive(Debug , Serialize , Clone)]
pub enum PaymentStatus {
    Initiated,
    Processing , 
    Cancelled , 
    Success
}

#[derive(Serialize,Debug,Clone)]
pub struct Payment {
    pub credit_card_num: u64,
    pub debit_card_num : u64,
    pub bank_identification_num : u64,
    pub expiration : u64,
    pub currency : String,
    pub amount : u64 , 
    pub order_id : Uuid,
    pub status : PaymentStatus
}