CREATE TABLE order {
    order_id                        UUID PRIMARY KEY , 
    expiration                      TEXT NOT NULL ,
    amount                          BIGINT NOT NULL CHECK (amount > 0)  
    currency                        VARCHAR(3) NOT NULL
    payment_status                  payment_status NOT NULL DEFAULT "pending"
    merchent_id                     UUID REFERENCES merchant(merchant_id)
    meta_data                       JSON
    gateway_provider                gateway_provider NOT NULL DEFAULT "stripe"
    authorized_amount           BIGINT  NULL
    captured_amount           BIGINT    NULL
    refunded_amount           BIGINT    NULL
    created_at                      TIMESTAMP   
    updated_at                      TIMESTAMP  
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
    order_id  UUID NOT NULL
    event_type TEXT
    created_at  TIMESTAMP
}
-- every merchent has one idemp key
CREATE TABLE idempotency_keys {
    id     UUID NOT NULL PRIMARY KEY
    merchant_id   UUID NOT NULL REFERENCES merchant(merchant_id)
    idemp_key     TEXT NOT NULL
    request_hash  TEXT
    order_id    UUID NOT NULL REFERENCES order(order_id)
    created_at  TIMESTAMP
    updated_at  TIMESTAMP   
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
    order_id   UUID NOT NULL REFERENCES order(order_id)
    gateway_provider  TEXT
    operation TEXT
    provider_request_id  TEXT
    provider_payment_id  TEXT
    created_at    TIMESTAMP
    completed_at  TIMESTAMP
}

CREATE TABLE refunds {
    id UUID NOT NULL PRIMARY KEY
    order_id  UUID NOT NULL REFERENCES order(order_id)
    refunded_amount  BIGINT
    refund_status  refund_status NOT NULL default "pending"
    provider_refund_id  TEXT
    created_at    TIMESTAMP
    is_refunded   BOOLEAN NOT NULL DEFAULT FALSE
}

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
    "pending"

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