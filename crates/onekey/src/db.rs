use anyhow::{Result, anyhow};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

fn ensure_columns(conn: &mut SqliteConnection) -> Result<()> {
    #[derive(diesel::QueryableByName)]
    struct ColName {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let cols: Vec<ColName> =
        diesel::sql_query("SELECT name FROM pragma_table_info('onekey_credentials')")
            .load(conn)?;
    if !cols.iter().any(|c| c.name == "encrypted_password") {
        conn.batch_execute(
            "ALTER TABLE onekey_credentials ADD COLUMN encrypted_password TEXT NOT NULL DEFAULT '';",
        )?;
    }
    if !cols.iter().any(|c| c.name == "kind") {
        conn.batch_execute(
            "ALTER TABLE onekey_credentials ADD COLUMN kind TEXT NOT NULL DEFAULT 'password';",
        )?;
    }
    if !cols.iter().any(|c| c.name == "key_path") {
        conn.batch_execute(
            "ALTER TABLE onekey_credentials ADD COLUMN key_path TEXT;",
        )?;
    }
    conn.batch_execute(
        "CREATE TABLE IF NOT EXISTS prompt_trigger_rules (
            id        TEXT PRIMARY KEY NOT NULL,
            keyword   TEXT NOT NULL UNIQUE,
            send_mode TEXT NOT NULL DEFAULT 'password_only'
                      CHECK (send_mode IN ('password_only', 'username_then_password'))
        );",
    )?;
    Ok(())
}

static DB_PATH: OnceLock<PathBuf> = OnceLock::new();
static CONN: OnceLock<Mutex<SqliteConnection>> = OnceLock::new();
#[cfg(test)]
thread_local! {
    static TEST_CONN: std::cell::RefCell<Option<SqliteConnection>> = std::cell::RefCell::new(None);
}

pub fn set_database_path(path: PathBuf) {
    let _ = DB_PATH.set(path);
}

#[cfg(test)]
pub fn set_test_conn(conn: SqliteConnection) {
    TEST_CONN.with(|tc| {
        tc.replace(Some(conn));
    });
}

fn open() -> Result<SqliteConnection> {
    let path = DB_PATH
        .get()
        .ok_or_else(|| anyhow!("warp_onekey db: database path not initialized"))?;
    let url = path.to_string_lossy();
    let mut conn = SqliteConnection::establish(&url)?;
    conn.batch_execute(
        "PRAGMA foreign_keys = ON; \
         PRAGMA busy_timeout = 2000; \
         PRAGMA journal_mode = WAL;",
    )?;
    conn.batch_execute(
        "CREATE TABLE IF NOT EXISTS onekey_credentials (
            id                TEXT PRIMARY KEY NOT NULL,
            label             TEXT NOT NULL,
            username          TEXT NOT NULL DEFAULT '',
            notes             TEXT NOT NULL DEFAULT '',
            encrypted_password TEXT NOT NULL DEFAULT '',
            kind              TEXT NOT NULL DEFAULT 'password',
            key_path          TEXT,
            created_at        TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at        TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );",
    )?;
    ensure_columns(&mut conn)?;
    Ok(conn)
}

pub fn with_conn<R>(f: impl FnOnce(&mut SqliteConnection) -> Result<R>) -> Result<R> {
    #[cfg(test)]
    {
        let mut conn_opt = TEST_CONN.with(|tc| tc.take());
        if let Some(ref mut conn) = conn_opt {
            let result = f(conn);
            TEST_CONN.with(|tc| tc.replace(conn_opt));
            return result;
        }
    }

    let mtx = CONN.get_or_init(|| Mutex::new(open().expect("warp_onekey db open")));
    let mut guard = mtx
        .lock()
        .map_err(|_| anyhow!("warp_onekey db mutex poisoned"))?;
    f(&mut guard)
}
