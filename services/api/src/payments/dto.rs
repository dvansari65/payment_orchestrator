// Request and response structs for the payments endpoints.

use std::collections::HashMap;

use common::{errors::ApiError, state::{Currency, PaymentStatus}};
use persistence::payment::PaymentRow;
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug,Serialize,Deserialize)]
pub struct PaymentResponse {
    pub id : Uuid,
    pub status: PaymentStatus,
    pub amount: i64,
    pub currency: Currency,
    pub authorized_amount : i64,
    pub captured_amount : i64,
    pub refunded_amount : i64,
    pub metadata: Option<HashMap<String, String>>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at : OffsetDateTime
}

#[derive(Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct CreatePaymentRequest {
    pub amount : i64,
    pub currency : Currency,
    pub metadata : Option<HashMap<String,String>>,
    pub description : Option<String>
}

impl TryFrom<PaymentRow> for PaymentResponse {
    type Error = ApiError;

    fn try_from(value: PaymentRow) -> Result<Self, Self::Error> {
        Ok(
            Self {
            id : value.id,
            status:value.status.parse()?,
            amount:value.amount,
            currency:value.currency.parse()?,
            authorized_amount:value.authorized_amount,
            captured_amount:value.captured_amount,
            refunded_amount:value.refunded_amount,
            metadata:value.metadata.map(|m| m.0),
            created_at:value.created_at
        })
    }
}