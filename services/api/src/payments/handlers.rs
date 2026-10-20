// One async fn per payments endpoint.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use common::{errors::ApiError, helpers::read_idemp_key};
use persistence::payment::{NewPayment, insert_payment};
use sha2::{Digest, Sha256};

use crate::{
    auth::AuthMerchant,
    payments::dto::{CreatePaymentRequest, PaymentResponse},
    state::AppState,
};

pub async fn create_payment(
    merchant: AuthMerchant,
    State(state): State<AppState>,
    header: HeaderMap,
    Json(body): Json<CreatePaymentRequest>,
) -> Result<(StatusCode, Json<PaymentResponse>), ApiError> {
    let idemp_key = read_idemp_key(&header)?;
    let pool = state.db;
    let bytes =
        serde_json::to_vec(&body).map_err(|_| ApiError::BadRequest("Invalid body!".into()))?;
    let request_hash = hex::encode(Sha256::digest(&bytes));

    let inputs = NewPayment {
        merchant_id: merchant.id,
        idemp_key: idemp_key.as_ref(),
        amount: body.amount,
        metadata: body.metadata.as_ref(),
        description: body.description.as_deref(),
        currency: body.currency.as_str(),
        request_hash: request_hash.as_ref(),
    };

    let response = insert_payment(&pool, inputs).await?;
    let payment_row = PaymentResponse::try_from(response)?;
    Ok((StatusCode::ACCEPTED, Json(payment_row)))
}
