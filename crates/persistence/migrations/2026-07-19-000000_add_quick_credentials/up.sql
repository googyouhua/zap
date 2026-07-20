CREATE TABLE IF NOT EXISTS quick_credentials (
    id                TEXT PRIMARY KEY NOT NULL,
    label             TEXT NOT NULL,
    username          TEXT NOT NULL DEFAULT '',
    send_mode         TEXT NOT NULL DEFAULT 'password_only'
                      CHECK (send_mode IN ('password_only', 'username_then_password')),
    notes             TEXT NOT NULL DEFAULT '',
    encrypted_password TEXT NOT NULL DEFAULT '',
    created_at        TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at        TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
