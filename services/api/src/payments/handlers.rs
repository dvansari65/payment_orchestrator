// One async fn per payments endpoint.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use persistence::payment::{
    New, NewPayment, PaymentListFilter, find_payment, insert_payment, list_payments, queue_refund,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    auth::AuthMerchant,
    error::ApiError,
    helpers::{hash_refund_request, read_idemp_key},
    pagination::decode_cursor,
    payments::dto::{
        CreatePaymentRequest, CreateRefundRequest, ListPaymentsQuery, PaymentListResponse,
        PaymentResponse, RefundResponse,
    },
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
    Ok((StatusCode::OK, Json(payment_row)))
}

pub async fn get_payment(
    merchant: AuthMerchant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PaymentResponse>, ApiError> {
    let payment = find_payment(&state.db, &id, &merchant.id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(PaymentResponse::try_from(payment)?))
}

pub async fn get_payment_list(
    merchant: AuthMerchant,
    State(state): State<AppState>,
    Query(params): Query<ListPaymentsQuery>,
) -> Result<Json<PaymentListResponse>, ApiError> {
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let cursor = params.cursor.as_deref().map(decode_cursor).transpose()?;

    if let (Some(from), Some(to)) = (params.created_gte, params.created_lt) {
        if from >= to {
            return Err(ApiError::BadRequest(
                "created_gte must be before created_lt".into(),
            ));
        }
    }
    let payment_filter = PaymentListFilter {
        cursor,
        limit: limit as i64,
        status: params.status.as_deref(),
        created_gte: params.created_gte,
        created_lt: params.created_lt,
    };
    let page = list_payments(&state.db, &merchant.id, &payment_filter).await?;
    Ok(Json(PaymentListResponse::try_from(page)?))
}

pub async fn create_refund(
    merchant: AuthMerchant,
    State(state): State<AppState>,
    header: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateRefundRequest>,
) -> Result<Json<RefundResponse>, ApiError> {
    let idemp_key = read_idemp_key(&header)?;
    let request_hash = hash_refund_request(id, &body)?;
    let inputs = New {
        payment_id: id,
        reason: body.reason.as_deref(),
        merchant_id: merchant.id,
        idemp_key: idemp_key.as_str(),
        amount: body.amount,
        request_hash: &request_hash,
    };
    let refund = queue_refund(&inputs, &state.db).await?;
    Ok(Json(RefundResponse::try_from(refund)?))
}
