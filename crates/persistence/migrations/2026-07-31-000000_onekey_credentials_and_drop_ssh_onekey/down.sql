CREATE TABLE ssh_onekey_credentials (
    id TEXT PRIMARY KEY NOT NULL,
    label TEXT NOT NULL,
    username TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE ssh_onekey_credentials
ADD COLUMN kind TEXT NOT NULL DEFAULT 'password' CHECK(kind IN ('password','key'));

ALTER TABLE ssh_onekey_credentials
ADD COLUMN key_path TEXT DEFAULT NULL;

-- 重建带 FK 的 ssh_servers
CREATE TABLE ssh_servers_old (
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
INSERT INTO ssh_servers_old SELECT * FROM ssh_servers;
DROP TABLE ssh_servers;

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
  credential_id     TEXT REFERENCES ssh_onekey_credentials(id) ON DELETE SET NULL
);
INSERT INTO ssh_servers_new SELECT * FROM ssh_servers_old;
DROP TABLE ssh_servers_old;
ALTER TABLE ssh_servers_new RENAME TO ssh_servers;

DROP TABLE onekey_credentials;
DROP TABLE prompt_trigger_rules;
