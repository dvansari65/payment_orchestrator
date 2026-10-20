CREATE TABLE payments {
    id                        UUID PRIMARY KEY , 
    amount                          BIGINT NOT NULL CHECK (amount > 0)  
    currency                        VARCHAR(3) NOT NULL 
    status                  payment_status NOT NULL DEFAULT "pending"
    merchent_id                     UUID REFERENCES merchant(merchant_id)
    meta_data                       JSON
    description                     TEXT
    gateway_provider                gateway_provider NOT NULL DEFAULT "stripe"
    authorized_amount               BIGINT  NULL
    captured_amount                 BIGINT    NULL
    refunded_amount                 BIGINT    NULL
    created_at                      TIMESTAMP   
    updated_at                      TIMESTAMP 
    expired_at                      TEXT NOT NULL , 
}
 
CREATE TABLE merchant {
    merchant_id UUID PRIMARY KEY
    balance          BIGINT 
    merchant_name             TEXT 
    status  merchant_status NOT NULL DEFAULT "active"      
    created_at       TIMESTAMP  NOT NULL
}

-- each merchant has ledger account
CREATE TABLE ledger_account {
    id    UUID NOT NULL PRIMARY KEY
    merchant_id UUID NOT NULL REFERENCES merchant(merchant_id)
    account_code  TEXT
    account_type  TEXT
    currency      CHAR
}

-- ledger account account has entries
CREATE TABLE ledger_entries {
    id    UUID NOT NULL PRIMARY KEY
    ledger_account_id   UUID NOT NULL REFERENCES ledger_account(id)
    ledger_transaction_id  UUI REFERENCES ledger_transaction(id)
    debit_amount  BIGINT
    credit_amount  BIGINT
}

-- ledger entries contains ledger transaction
CREATE TABLE ledger_transaction {
    id   UUID  NOT NULL PRIMARY KEY
    payment_id  UUID NOT NULL
    event_type TEXT
    created_at  TIMESTAMP
}
-- idemp_key should be one row per request, many rows per merchant
-- primary key should be merchent id and idemp key , not only idemp key
CREATE TABLE idempotency_keys {
    id     UUID NOT NULL PRIMARY KEY
    merchant_id   UUID NOT NULL REFERENCES merchant(merchant_id)
    idemp_key     TEXT NOT NULL 
    request_hash  TEXT
    payment_id    UUID NOT NULL REFERENCES order(order_id)
    expired_at  TIMESTAMP
    created_at  TIMESTAMP
    updated_at  TIMESTAMP   
    UNIQUE(idemp_key,merchant_id)
}
-- every merchant owns api keys table 
CREATE TABLE api_keys {
    id   UUID NOT NULL PRIMARY KEY
    merchant_id UUID NOT NULL REFERENCES merchant(merchant_id)
    key_prefix  TEXT
    key_hash    TEXT
    last_used    TIMESTAMP
    created_at   TIMESTAMP
    revoked_at   TIMESTAMP
}

-- each payment will have number of attempts to complete
CREATE TABLE payment_attempts {
    id UUID NOT NULL PRIMARY KEY
    payment_id   UUID NOT NULL REFERENCES order(order_id)
    provider   gateway_provider NOT NULL default "stripe"
    operation TEXT
    provider_request_id  TEXT
    provider_payment_id  TEXT
    created_at    TIMESTAMP
    completed_at  TIMESTAMP
    status      payment_attempt_status NOT NULL DEFAULT "unknown"
}

CREATE TABLE refunds {
    id UUID NOT NULL PRIMARY KEY
    payment_id  UUID NOT NULL REFERENCES order(order_id)
    refunded_amount  BIGINT
    refund_status  refund_status NOT NULL default "pending"
    provider_refund_id  TEXT
    created_at    TIMESTAMP
    is_refunded   BOOLEAN NOT NULL DEFAULT FALSE
}

CREATE TABLE webhooks_events {
    id         UUID PRIMARY KEY
    provider   gateway_provider NOT NULL 
    provider_event_id  TEXT     NOT NULL
    event_type         TEXT     NOT NULL
    payload            TEXT     NOT NULL
    processed_at       TIMESTAMPTZ
    received_at        TIMESTAMPTZ NOT NULL DEFAULT now()
    process_err        TEXT
    attempts           INT     NOT NULL DEFAULT 0
    UNIQUE(provider , provider_event_id)
}

CREATE INDEX webhook_events_unprocessed_idx
    on webhooks_events (received_at)
    where processed_at is NULL;


CREATE TABLE outbox (
    id           UUID PRIMARY KEY,
    payment_id   UUID NOT NULL REFERENCES payments(id),
    action       TEXT NOT NULL,                 -- 'authorize', 'capture', 'refund'
    status       TEXT NOT NULL DEFAULT 'pending',
    attempts     INT  NOT NULL DEFAULT 0,
    next_run_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX outbox_pending_idx ON outbox (next_run_at) WHERE status = 'pending';

CREATE TYPE merchant_status AS ENUM {
    "active",
    "suspended",
    "pending"
}

CRETAE TYPE payment_status AS ENUM {
    "initiated" , 
    "processing" , 
    "cancelled" , 
    "success" , 
    "refunded", 
    "failed",
}

CREATE TYPE refunds_status AS ENUM {
    "failed",
    "pending",
    "success"
}

CREATE TYPE gateway_provider AS ENUM {
    "razor_pay",
    "stripe"
}

CREATE TYPE payment_attempt_status AS ENUM {
    "pending",
    "succeeded",
    "failed",
    "unknown"
}