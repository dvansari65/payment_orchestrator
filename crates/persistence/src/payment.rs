use std::collections::HashMap;

use crate::db::DbPool;
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
    pub merchent_id: Uuid,
    pub metadata: Option<Json<HashMap<String, String>>>,
    pub gateway_provider: String,
    pub authorized_amount: i64,
    pub captured_amount: i64,
    pub refunded_amount: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub expired_at: OffsetDateTime,
}

pub async fn insert_payment(pool: &DbPool, new: NewPayment<'_>) -> Result<PaymentRow, sqlx::Error> {
    let payment_id = Uuid::now_v7();

    let mut  tx = pool.begin().await?;

    let payment = sqlx::query_as::<_,PaymentRow>(
        r#"
            INSERT INTO payments
                ( id , amount , currency , status , merchant_id, metadata , description , expired_at)
            VALUES
                ( $1 , $2 , $3 , "initiated , $4 , $5 , $6 , now() + interval '30 minutes')
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
