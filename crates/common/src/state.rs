use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::errors::ApiError;

#[derive(Debug, Serialize, Deserialize)]
pub enum PaymentStatus {
    Initiated,
    Cancelled,
    Failed,
    Processing,
    Refunded,
    Success,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Currency {
    INR,
    USD,
}

impl Currency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Currency::INR => "INR",
            Currency::USD => "USD",
        }
    }
}
impl FromStr for Currency {
    type Err = ApiError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INR" => Ok(Self::INR),
            "USD" => Ok(Self::USD),
            other => Err(ApiError::Internal(format!(
                "Internal server error:{other}!"
            ))),
        }
    }
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Cancelled => "cancelled",
            PaymentStatus::Failed => "failed",
            PaymentStatus::Initiated => "initiated",
            PaymentStatus::Processing => "processing",
            PaymentStatus::Refunded => "refunded",
            PaymentStatus::Success => "success",
        }
    }
}

impl FromStr for PaymentStatus {
    type Err = ApiError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cancelled" => Ok(PaymentStatus::Cancelled),
            "failed" => Ok(PaymentStatus::Failed),
            "initiated" => Ok(PaymentStatus::Initiated),
            "processing" => Ok(PaymentStatus::Processing),
            "refunded" => Ok(PaymentStatus::Refunded),
            "success" => Ok(PaymentStatus::Success),
            other => Err(ApiError::Internal(format!(
                "Invalid payment status:{other}!"
            ))),
        }
    }
}
