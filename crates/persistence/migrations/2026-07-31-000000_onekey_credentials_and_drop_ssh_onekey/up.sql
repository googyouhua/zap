CREATE TABLE onekey_credentials (
  id                 TEXT PRIMARY KEY NOT NULL,
  label              TEXT NOT NULL,
  username           TEXT NOT NULL DEFAULT '',
  notes              TEXT NOT NULL DEFAULT '',
  encrypted_password TEXT NOT NULL DEFAULT '',
  kind               TEXT NOT NULL DEFAULT 'password',
  key_path           TEXT,
  created_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE prompt_trigger_rules (
  id        TEXT PRIMARY KEY NOT NULL,
  keyword   TEXT NOT NULL UNIQUE,
  send_mode TEXT NOT NULL DEFAULT 'password_only'
            CHECK (send_mode IN ('password_only', 'username_then_password'))
);

-- 重建 ssh_servers 去掉对 ssh_onekey_credentials 的外键约束
CREATE TABLE ssh_servers_new (
  node_id           TEXT PRIMARY KEY NOT NULL REFERENCES ssh_nodes(id) ON DELETE CASCADE,
  host              TEXT NOT NULL,
  port              INTEGER NOT NULL DEFAULT 22,
  username          TEXT NOT NULL DEFAULT '',
  auth_type         TEXT NOT NULL CHECK(auth_type IN ('password','key','onekey')) DEFAULT 'password',
  key_path          TEXT,
  startup_command   TEXT DEFAULT NULL,
  notes             TEXT DEFAULT NULL,
  last_connected_at TIMESTAMP,
  credential_id     TEXT
);
INSERT INTO ssh_servers_new SELECT * FROM ssh_servers;
DROP TABLE ssh_servers;
ALTER TABLE ssh_servers_new RENAME TO ssh_servers;
DROP TABLE IF EXISTS ssh_onekey_credentials;
