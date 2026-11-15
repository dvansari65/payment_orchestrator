use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
#[error("unknown {kind}: {value}")]
pub struct ParseEnumError {
    pub kind: &'static str,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PaymentStatus {
    Initiated,
    Authorized,
    Captured,
    Expired,
    Settled,
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
    type Err = ParseEnumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INR" => Ok(Self::INR),
            "USD" => Ok(Self::USD),
            other => Err(ParseEnumError {
                kind: "Currency",
                value: other.to_string(),
            }),
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
            PaymentStatus::Authorized => "authorized",
            PaymentStatus::Captured => "captured",
            PaymentStatus::Expired => "expired",
            PaymentStatus::Settled => "settled",
        }
    }
}

impl FromStr for PaymentStatus {
    type Err = ParseEnumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cancelled" => Ok(PaymentStatus::Cancelled),
            "failed" => Ok(PaymentStatus::Failed),
            "initiated" => Ok(PaymentStatus::Initiated),
            "processing" => Ok(PaymentStatus::Processing),
            "refunded" => Ok(PaymentStatus::Refunded),
            "success" => Ok(PaymentStatus::Success),
            "authorized" => Ok(PaymentStatus::Authorized),
            "captured" => Ok(PaymentStatus::Captured),
            "settled" => Ok(PaymentStatus::Settled),
            other => Err(ParseEnumError {
                kind: "PaymentStatus",
                value: other.to_string(),
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RefundStatus {
    Pending,
    Failed,
    Success,
}

impl RefundStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RefundStatus::Failed => "failed",
            RefundStatus::Pending => "pending",
            RefundStatus::Success => "success",
        }
    }
}

impl FromStr for RefundStatus {
    type Err = ParseEnumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(RefundStatus::Pending),
            "failed" => Ok(RefundStatus::Failed),
            "success" => Ok(RefundStatus::Success),
            other => Err(ParseEnumError {
                kind: "Refund status",
                value: other.to_string(),
            }),
        }
    }
}
