CREATE TABLE IF NOT EXISTS quick_credentials (
    id                TEXT PRIMARY KEY NOT NULL,
    label             TEXT NOT NULL,
    username          TEXT NOT NULL DEFAULT '',
    notes             TEXT NOT NULL DEFAULT '',
    encrypted_password TEXT NOT NULL DEFAULT '',
    created_at        TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at        TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
