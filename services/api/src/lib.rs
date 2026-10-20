//! HTTP layer for the payment orchestrator.
//!
//! This crate only deals with HTTP: routing, extracting input, shaping output,
//! and mapping errors to status codes. Business rules live in `domain` and
//! `application`; SQL lives in `persistence`.

pub mod config;
pub mod health;
pub mod payments;
pub mod router;
pub mod state;
pub mod auth;