use std::collections::HashMap;

use crate::db::DbPool;
use common::state::PaymentStatus;
use sqlx::types::Json;
use time::OffsetDateTime;
use uuid::Uuid;

pub struct NewPayment<'a> {
    pub merchant_id: Uuid,
    pub idemp_key: &'a str,
    pub amount: i64,
    pub metadata: Option<&'a HashMap<String, String>>,
    pub description: Option<&'a str>,
    pub currency: &'a str,
    pub request_hash: &'a str,
}
#[derive(Debug, sqlx::FromRow)]
pub struct PaymentRow {
    pub id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub merchant_id: Uuid,
    pub metadata: Option<Json<HashMap<String, String>>>,
    pub description: Option<String>,
    pub gateway_provider: String,
    pub authorized_amount: i64,
    pub captured_amount: i64,
    pub refunded_amount: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("payment not found!")]
    NotFound,
    #[error("payment is {0}; only authorized payment can be captured!")]
    InvalidStatus(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("Invalid amount!")]
    InvalidAmount,
    #[error("request hash not matched!")]
    InvalidRequestHash,
}
#[derive(Debug, thiserror::Error)]
pub enum RefundError {
    #[error("Payment not found!")]
    NotFound,
    #[error("payment is {0}; only captured amount can be refundable!")]
    NotRefundable(String),
    #[error("request hash not matched!")]
    InvalidRequestHash,
    #[error("already refunded!")]
    AlreadyRefunded,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("Already in Progress!")]
    AlreadyInProgress,
    #[error("Invalid amount!")]
    InvalidAmount,
    #[error("Zero amount left in your account!")]
    NothingToRefund,
    #[error("refund of {requested} exceeds the refundable balance of {available}")]
    ExceedsRefundable { requested: i64, available: i64 },
}

pub struct New<'a> {
    pub payment_id: Uuid,
    pub reason: Option<&'a str>,
    pub merchant_id: Uuid,
    pub idemp_key: &'a str,
    pub amount: Option<i64>,
    pub request_hash: &'a str,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RefundRow {
    pub id: Uuid,
    pub payment_id: Uuid,
    pub amount: i64,
    pub status: String,
    pub reason: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RefundablePayment {
    pub id: Uuid,
    pub status: String,
    pub captured_amount: i64,
    pub refunded_amount: i64,
}
/// Filters for one page. Every field except `limit` is optional:
/// `None` means "don't filter on this".
pub struct PaymentListFilter<'a> {
    pub cursor: Option<Uuid>,
    pub limit: i64,
    pub status: Option<&'a str>,
    pub created_gte: Option<OffsetDateTime>,
    pub created_lt: Option<OffsetDateTime>,
}

pub struct PaymentPage {
    pub rows: Vec<PaymentRow>,
    pub has_more: bool,
}

pub struct CancelPaymemt<'a> {
    pub payment_id:Uuid,
    pub request_hash:&'a str,
    pub idemp_key:&'a str,
    pub merchant_id:Uuid,
    pub resource_id:Uuid
}

pub async fn insert_payment(pool: &DbPool, new: NewPayment<'_>) -> Result<PaymentRow, sqlx::Error> {
    let payment_id = Uuid::now_v7();
    // why we are not inserting row in payment_attempts
    let mut tx = pool.begin().await?;

    let payment = sqlx::query_as::<_,PaymentRow>(
        r#"
            INSERT INTO payments
                ( id , amount , currency , status , merchant_id, metadata , description , expired_at)
            VALUES
                ( $1 , $2 , $3 , "initiated" , $4 , $5 , $6 , now() + interval '30 minutes')
            RETURNING 
                id, merchant_id, amount, currency, status,
                authorized_amount, captured_amount, refunded_amount,
                description, metadata, expires_at, created_at
        "#
    )
    .bind(payment_id)
    .bind(new.amount)
    .bind(new.currency)
    .bind(new.merchant_id)
    .bind(new.metadata.map(Json))
    .bind(new.description)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO idempotency_keys
            ( id , merchant_id , idemp_key , request_hash , payment_id , expired_at )
        VALUES 
            ($1 , $2 , $3 , $4 , $5 ,now() + interval '24 hours')
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(new.merchant_id)
    .bind(new.idemp_key)
    .bind(new.request_hash)
    .bind(payment_id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
    INSERT INTO outbox (id, payment_id, action)
    VALUES ($1, $2, 'authorize')
    "#,
    )
    .bind(Uuid::now_v7())
    .bind(payment_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(payment)
}

pub async fn find_payment(
    pool: &DbPool,
    payment_id: &Uuid,
    merchant_id: &Uuid,
) -> Result<Option<PaymentRow>, sqlx::Error> {
    let payment = sqlx::query_as(
        r#"
        SELECT id, merchant_id, amount, currency, status,
               authorized_amount, captured_amount, refunded_amount,
               description, metadata, expires_at, created_at
        FROM payments
        WHERE id = $1 AND merchant_id = $2
        "#,
    )
    .bind(payment_id)
    .bind(merchant_id)
    .fetch_optional(pool)
    .await?;
    Ok(payment)
}

pub async fn list_payments(
    pool: &DbPool,
    merchant_id: &Uuid,
    filter_params: &PaymentListFilter<'_>,
) -> Result<PaymentPage, sqlx::Error> {
    let mut payment_list = sqlx::query_as(
        r#"
            SELECT id, merchant_id, amount, currency, status,
               authorized_amount, captured_amount, refunded_amount,
               description, metadata, expires_at, created_at
            FROM payments
            WHERE merchant_id = $1
                AND ($2::uuid IS NULL OR id < $2)
                AND ($3::payment_status IS NULL OR status = $3::payment_status)
                AND ($4::timestamptz IS NULL OR created_at >= $4)
                AND ($5::timestamptz) IS NULL OR created_at < $5)  
            ORDER BY id DESC
            LIMIT $6
        "#,
    )
    .bind(merchant_id)
    .bind(filter_params.cursor)
    .bind(filter_params.status)
    .bind(filter_params.created_gte)
    .bind(filter_params.created_lt)
    .bind(filter_params.limit + 1)
    .fetch_all(pool)
    .await?;
    let has_more = payment_list.len() as i64 > filter_params.limit;
    payment_list.truncate(filter_params.limit as usize);
    Ok(PaymentPage {
        rows: payment_list,
        has_more,
    })
}

pub async fn find_authorized_payment_to_capture(
    pool: &DbPool,
    payment_id: &Uuid,
    merchant_id: &Uuid,
    amount: Option<i64>,
) -> Result<PaymentRow, CaptureError> {
    let mut tx = pool.begin().await?;

    let payment = sqlx::query_as::<_, PaymentRow>(
        r#"
            SELECT id, merchant_id, amount, currency, status,
               authorized_amount, captured_amount, refunded_amount,
               description, metadata, expires_at, created_at
            FROM payments
            WHERE id = $1 AND merchant_id = $2
        "#,
    )
    .bind(payment_id)
    .bind(merchant_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(CaptureError::NotFound)?;

    // make sure db status matching with your custom status
    if payment.status != PaymentStatus::Authorized.as_str() {
        return Err(CaptureError::InvalidStatus(payment.status));
    }
    let amount = amount.unwrap_or(payment.authorized_amount);
    if amount <= 0 || amount > payment.authorized_amount {
        return Err(CaptureError::InvalidAmount);
    }

    sqlx::query(
        r#"
            INSERT INTO outbox (id , payment_id , action )
            VALUES ($1, $2, 'capture')
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(payment_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(payment)
}

pub async fn cancel_payment(
    pool: &DbPool,
    cancel_payment_input:CancelPaymemt<'_>,
) -> Result<(), CaptureError> {
    let mut tx = pool.begin().await?;
    let claimed : Option<Uuid>  = sqlx::query_scalar(
        r#"
            INSERT INTO idempotency_keys (
                merchant_id , idemp_key ,request_hash,resource_type , resource_id , expires_at 
            )
            VALUES ( $1 , $2 , $3 , "cancel" , $4 , now + interval '24 hours' )
            ON CONFLICT  ( merchant_id , idemp_key ) DO NOTHING
            RETURNING resource_id
        "#
    )
    .bind(cancel_payment_input.merchant_id)
    .bind(cancel_payment_input.idemp_key)
    .bind(cancel_payment_input.request_hash)
    .bind(cancel_payment_input.resource_id)
    .fetch_optional(&mut *tx)
    .await?;

    if claimed.is_none() {
        let request_hash: String  = sqlx::query_scalar(
            r#"
                SELECT request_hash  FROM idempotency_keys
                WHERE merchant_id = $1 AND idemp_key = $2
            "#
        )
        .bind(cancel_payment_input.merchant_id)
        .bind(cancel_payment_input.idemp_key)
        .fetch_one(&mut *tx)
        .await?;
    
        if request_hash != cancel_payment_input.request_hash {
            return Err(CaptureError::InvalidRequestHash);
        }
       return Ok(())
    }
    let payment_row = sqlx::query_as::<_, PaymentRow>(
        r#"
            SELECT id, merchant_id, amount, currency, status,
               authorized_amount, captured_amount, refunded_amount,
               description, metadata, expires_at, created_at
            FROM payments
            WHERE id = $1 AND merchant_id = $2
        "#,
    )
    .bind(cancel_payment_input.payment_id)
    .bind(cancel_payment_input.merchant_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(CaptureError::NotFound)?;

    // cross check that we are performing right action by checking only settled and authorized status
    // Is it feasible to cancel payment when status is other than  authorized and settled
    if payment_row.status == PaymentStatus::Captured.as_str() || payment_row.status == PaymentStatus::Settled.as_str() {
        return Err(CaptureError::InvalidStatus(payment_row.status));
    }
    sqlx::query(
        r#"
            INSERT INTO outbox ( id , payment_id, action )
            VALUES ($1 , $2 , "cancel" )
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(payment_row.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

// partial refund allowed
pub async fn queue_refund(new: &New<'_>, pool: &DbPool) -> Result<RefundRow, RefundError> {
    let refund_id = Uuid::now_v7();
    let amount = new.amount;

    if amount.is_none() || amount.unwrap() <= 0 {
        return Err(RefundError::InvalidAmount);
    }

    let mut tx = pool.begin().await?;
    // NOTE : i think we have to give resource_id from the input not newly generated uuid
    let claimed: Option<Uuid> = sqlx::query_scalar(
        r#"
            INSERT INTO idempotency_keys (merchant_id,idemp_key,request_hash,
            resource_type,resource_id, expires_at )
            VALUES ($1 , $2 , $3, 'refund', $4 , now() + interval '24 hours' )
            ON CONFLICT ( merchant_id , idemp_key ) DO NOTHING
            RETURNING resource_id
        "#,
    )
    .bind(new.merchant_id)
    .bind(new.idemp_key)
    .bind(new.request_hash)
    .bind(refund_id)
    .fetch_optional(&mut *tx)
    .await?;

    if claimed.is_none() {
        let (request_hash, resource_id): (String, Uuid) = sqlx::query_as(
            r#"
            SELECT  request_hash , resource_id FROM idempotency_keys
            WHERE idemp_key = $1 AND merchant_id = $2
            "#,
        )
        .bind(new.idemp_key)
        .bind(new.merchant_id)
        .fetch_one(&mut *tx)
        .await?;

        if request_hash != new.request_hash {
            return Err(RefundError::InvalidRequestHash);
        }
        let existing = sqlx::query_as::<_, RefundRow>(
            r#"
                SELECT id , payment_id, amount, status::TEXT AS status, reason, created_at
                FROM refunds
                WHERE id  = $1
            "#,
        )
        .bind(resource_id)
        .fetch_one(&mut *tx)
        .await?;
        return Ok(existing);
    }

    let payment = sqlx::query_as::<_, RefundablePayment>(
        r#"
            SELECT id , status::TEXT AS status , captured_amount, refunded_amount
            FROM payments
            WHERE id = $1 AND merchant_id = $2
            FOR UPDATE
        "#,
    )
    .bind(new.payment_id)
    .bind(new.merchant_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(RefundError::NotFound)?;

    let refundable = [
        PaymentStatus::Settled.as_str(),
        PaymentStatus::Captured.as_str(),
    ];

    if !refundable.contains(&payment.status.as_str()) {
        return Err(RefundError::NotRefundable(payment.status.to_string()));
    }
    let in_flight: i64 = sqlx::query_scalar(
        r#"
            SELECT COALESCE (SUM(amount),0)
            FROM refunds
            WHERE payment_id = $1 AND status = 'pending'
        "#,
    )
    .bind(payment.id)
    .fetch_one(&mut *tx)
    .await?;

    let available = payment
        .captured_amount
        .checked_sub(payment.refunded_amount)
        .and_then(|v| v.checked_sub(in_flight))
        .ok_or_else(|| RefundError::Database(sqlx::Error::Protocol("amount overflow".into())))?;

    if available <= 0 {
        return Err(RefundError::NothingToRefund);
    }
    let amount = amount.unwrap_or(available);
    if amount > available {
        return Err(RefundError::ExceedsRefundable {
            requested: amount,
            available: available,
        });
    }
    let refund = sqlx::query_as::<_, RefundRow>(
        r#"
            INSERT INTO refunds (id , payment_id, merchant_id, amount ,reason)
            VALUES ($1 , $2 , $3 , $4, $5 )
            RETURNING id, payment_id, amount, status::TEXT AS status, reason, created_at
        "#,
    )
    .bind(refund_id)
    .bind(payment.id)
    .bind(new.merchant_id)
    .bind(new.amount)
    .bind(new.reason)
    .fetch_one(&mut *tx)
    .await?;

    let queue =  sqlx::query(
        r#"
            INSERT INTO outbox ( id , payment_id, refund_id , status::TEXT AS status, amount , action )
            VALUES ( $1, $2 , $3 , 'pending' , $4 , 'refund' )
        "#
    )
    .bind(refund_id)
    .bind(payment.id)
    .bind(refund.id)
    .bind(refund.amount)
    .execute(&mut *tx)
    .await;
    match queue {
        Ok(_) => {}
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            return Err(RefundError::AlreadyInProgress);
        }
        Err(e) => return Err(e.into()),
    }
    tx.commit().await?;
    Ok(refund)
}
