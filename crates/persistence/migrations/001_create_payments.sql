-- ============ enums ============
CREATE TYPE merchant_status AS ENUM ('active', 'suspended', 'pending');

CREATE TYPE payment_status AS ENUM (
    'initiated', 'processing', 'authorized', 'captured', 'settled',
    'failed', 'cancelled', 'expired', 'refunded'
);

CREATE TYPE refund_status AS ENUM ('pending', 'success', 'failed');

CREATE TYPE gateway_provider AS ENUM ('stripe', 'razorpay');

CREATE TYPE payment_attempt_status AS ENUM ('pending', 'success', 'failed', 'unknown');

-- ============ merchants & auth ============
CREATE TABLE merchants (
    id          UUID PRIMARY KEY,
    name        TEXT NOT NULL,
    status      merchant_status NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE api_keys (
    id            UUID PRIMARY KEY,
    merchant_id   UUID NOT NULL REFERENCES merchants(id),
    key_prefix    TEXT NOT NULL,              -- e.g. "sk_live_8f3k", safe to show in the dashboard
    key_hash      TEXT NOT NULL UNIQUE,       -- SHA-256 of the full key
    last_used_at  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at    TIMESTAMPTZ
);

-- ============ payments ============
CREATE TABLE payments (
    id                 UUID PRIMARY KEY,
    merchant_id        UUID NOT NULL REFERENCES merchants(id),
    amount             BIGINT NOT NULL CHECK (amount > 0),
    currency           TEXT NOT NULL CHECK (char_length(currency) = 3),
    status             payment_status NOT NULL DEFAULT 'initiated',
    metadata           JSONB,
    description        TEXT,
    authorized_amount  BIGINT NOT NULL DEFAULT 0,
    captured_amount    BIGINT NOT NULL DEFAULT 0,
    refunded_amount    BIGINT NOT NULL DEFAULT 0,
    expires_at         TIMESTAMPTZ NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),

    CHECK (0 <= captured_amount AND captured_amount <= authorized_amount AND authorized_amount <= amount),
    CHECK (0 <= refunded_amount AND refunded_amount <= captured_amount)
);
-- list endpoint: WHERE merchant_id = $1 AND id < $cursor ORDER BY id DESC
CREATE INDEX payments_merchant_id_idx ON payments (merchant_id, id DESC);

CREATE TABLE payment_attempts (
    id                   UUID PRIMARY KEY,
    payment_id           UUID NOT NULL REFERENCES payments(id),
    provider             gateway_provider NOT NULL,
    operation            TEXT NOT NULL CHECK (operation IN ('authorize', 'capture', 'void', 'refund')),
    status               payment_attempt_status NOT NULL DEFAULT 'pending',
    provider_request_id  TEXT NOT NULL UNIQUE,   -- idempotency key we send to the provider
    provider_payment_id  TEXT,                   -- provider's id, known after the call
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at         TIMESTAMPTZ
);
CREATE INDEX payment_attempts_payment_id_idx ON payment_attempts (payment_id);

CREATE TABLE refunds (
    id                  UUID PRIMARY KEY,
    payment_id          UUID NOT NULL REFERENCES payments(id),
    merchant_id         UUID NOT NULL REFERENCES merchants(id),
    amount              BIGINT NOT NULL CHECK (amount > 0),
    status              refund_status NOT NULL DEFAULT 'pending',
    reason              TEXT,
    provider_refund_id  TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX refunds_payment_id_idx ON refunds (payment_id);

-- ============ async work ============
CREATE TABLE outbox (
    id           UUID PRIMARY KEY,
    payment_id   UUID NOT NULL REFERENCES payments(id),
    refund_id    UUID REFERENCES refunds(id),            -- only for action = 'refund'
    action       TEXT NOT NULL CHECK (action IN ('authorize', 'capture', 'void', 'refund')),
    amount       BIGINT CHECK (amount IS NULL OR amount > 0),
    status       TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'done', 'failed')),
    attempts     INT NOT NULL DEFAULT 0,
    next_run_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX outbox_due_idx ON outbox (next_run_at) WHERE status = 'pending';
CREATE UNIQUE INDEX outbox_one_pending_per_payment ON outbox (payment_id) WHERE status = 'pending';

CREATE TABLE idempotency_keys (
    merchant_id    UUID NOT NULL REFERENCES merchants(id),
    idemp_key      TEXT NOT NULL CHECK (char_length(idemp_key) BETWEEN 1 AND 255),
    request_hash   TEXT NOT NULL,
    resource_type  TEXT NOT NULL CHECK (resource_type IN ('payment', 'refund')),
    resource_id    UUID NOT NULL,
    expires_at     TIMESTAMPTZ NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (merchant_id, idemp_key)
);
CREATE INDEX idempotency_keys_expires_idx ON idempotency_keys (expires_at);

CREATE TABLE webhook_events (
    id                 UUID PRIMARY KEY,
    provider           gateway_provider NOT NULL,
    provider_event_id  TEXT NOT NULL,
    event_type         TEXT NOT NULL,
    payload            TEXT NOT NULL,          -- raw body, exactly as received, for signature checks
    received_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    processed_at       TIMESTAMPTZ,
    process_error      TEXT,
    attempts           INT NOT NULL DEFAULT 0,
    UNIQUE (provider, provider_event_id)
);
CREATE INDEX webhook_events_unprocessed_idx ON webhook_events (received_at) WHERE processed_at IS NULL;

-- ============ ledger (append-only) ============
CREATE TABLE ledger_accounts (
    id            UUID PRIMARY KEY,
    merchant_id   UUID REFERENCES merchants(id),   -- NULL = platform-owned account
    code          TEXT NOT NULL,                   -- e.g. 'merchant_payable', 'provider_receivable'
    account_type  TEXT NOT NULL CHECK (account_type IN ('asset', 'liability', 'revenue', 'expense')),
    currency      TEXT NOT NULL CHECK (char_length(currency) = 3),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (merchant_id, code, currency)
);

CREATE TABLE ledger_transactions (
    id          UUID PRIMARY KEY,
    payment_id  UUID REFERENCES payments(id),
    event_type  TEXT NOT NULL,                     -- 'capture', 'refund', 'fee', 'settlement'
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE ledger_entries (
    id              UUID PRIMARY KEY,
    transaction_id  UUID NOT NULL REFERENCES ledger_transactions(id),
    account_id      UUID NOT NULL REFERENCES ledger_accounts(id),
    direction       TEXT NOT NULL CHECK (direction IN ('debit', 'credit')),
    amount          BIGINT NOT NULL CHECK (amount > 0),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ledger_entries_account_idx ON ledger_entries (account_id);
CREATE INDEX ledger_entries_transaction_idx ON ledger_entries (transaction_id);