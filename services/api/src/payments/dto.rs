// Request and response structs for the payments endpoints.

use std::collections::HashMap;

use axum::response::Result;
use common::{
    errors::ApiError,
    state::{Currency, PaymentStatus},
};
use persistence::payment::{PaymentPage, PaymentRow};
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use uuid::Uuid;

use crate::pagination::encode_cursor;

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub id: Uuid,
    pub status: PaymentStatus,
    pub amount: i64,
    pub currency: Currency,
    pub authorized_amount: i64,
    pub captured_amount: i64,
    pub refunded_amount: i64,
    pub metadata: Option<HashMap<String, String>>,
    pub description: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CreatePaymentRequest {
    pub amount: i64,
    pub currency: Currency,
    pub metadata: Option<HashMap<String, String>>,
    pub description: Option<String>,
}

impl TryFrom<PaymentRow> for PaymentResponse {
    type Error = ApiError;

    fn try_from(value: PaymentRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            status: value.status.parse()?,
            amount: value.amount,
            currency: value.currency.parse()?,
            authorized_amount: value.authorized_amount,
            captured_amount: value.captured_amount,
            refunded_amount: value.refunded_amount,
            metadata: value.metadata.map(|m| m.0),
            description: value.description,
            created_at: value.created_at,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListPaymentsQuery {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub status: Option<String>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub created_gte: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub created_lt: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize)]
pub struct PaymentListResponse {
    pub data: Vec<PaymentResponse>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

impl TryFrom<PaymentPage> for PaymentListResponse {
    type Error = ApiError;
    fn try_from(page: PaymentPage) -> Result<Self, Self::Error> {
        let data = page
            .rows
            .into_iter()
            .map(PaymentResponse::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = if page.has_more {
            data.last().map(|p| encode_cursor(p.id))
        } else {
            None
        };
        Ok(Self {
            data,
            has_more: page.has_more,
            next_cursor,
        })
    }
}
