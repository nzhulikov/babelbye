CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email TEXT,
    phone TEXT,
    nickname TEXT NOT NULL,
    tagline TEXT,
    native_language TEXT NOT NULL,
    is_searchable BOOLEAN NOT NULL DEFAULT TRUE,
    translation_quota_remaining INTEGER NOT NULL DEFAULT 1000,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS users_email_idx ON users (email) WHERE email IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS users_phone_idx ON users (phone) WHERE phone IS NOT NULL;

CREATE TABLE IF NOT EXISTS connections (
    id UUID PRIMARY KEY,
    requester_id UUID NOT NULL REFERENCES users(id),
    addressee_id UUID NOT NULL REFERENCES users(id),
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(requester_id, addressee_id)
);

CREATE TABLE IF NOT EXISTS message_receipts (
    id UUID PRIMARY KEY,
    sender_id UUID NOT NULL REFERENCES users(id),
    recipient_id UUID NOT NULL REFERENCES users(id),
    has_translation BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS feedback_reports (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    message TEXT NOT NULL,
    issue_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
