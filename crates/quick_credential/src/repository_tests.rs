use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use tempfile::NamedTempFile;
use zeroize::Zeroizing;
use crate::{QuickCredential, SendMode};
use crate::db::set_test_conn;

fn setup_db() -> NamedTempFile {
    let file = NamedTempFile::new().expect("failed to create temp db");
    let path = file.path().to_path_buf();
    let url = path.to_string_lossy();
    let mut conn = SqliteConnection::establish(&url)
        .expect("failed to open sqlite");
    conn.batch_execute(
        "CREATE TABLE IF NOT EXISTS quick_credentials (
            id                TEXT PRIMARY KEY NOT NULL,
            label             TEXT NOT NULL,
            username          TEXT NOT NULL DEFAULT '',
            send_mode         TEXT NOT NULL DEFAULT 'password_only',
            notes             TEXT NOT NULL DEFAULT '',
            encrypted_password TEXT NOT NULL DEFAULT '',
            created_at        TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at        TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );"
    ).expect("failed to init schema");
    set_test_conn(conn);
    file
}

fn clean_db() {
    let _ = crate::with_conn(|conn| {
        conn.batch_execute("DELETE FROM quick_credentials")
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(())
    });
}

fn test_cred(label: &str, username: &str, password: &str) -> QuickCredential {
    QuickCredential {
        id: String::new(),
        label: label.into(),
        username: username.into(),
        send_mode: SendMode::PasswordOnly,
        notes: String::new(),
        password: Zeroizing::new(password.into()),
    }
}

#[test]
fn test_find_all_empty() {
    let _db = setup_db();
    clean_db();
    let result = crate::find_all().expect("find_all failed");
    assert!(result.is_empty(), "expected empty, got {} results", result.len());
}

#[test]
fn test_create_and_find_by_id() {
    let _db = setup_db();
    clean_db();
    let cred = test_cred("GitHub", "user@github.com", "gh_secret");
    let created = crate::create(&cred).expect("create failed");
    assert!(!created.id.is_empty(), "id should be set");
    assert_eq!(created.label, "GitHub");
    assert_eq!(created.username, "user@github.com");

    let found = crate::find_by_id(&created.id)
        .expect("find_by_id failed")
        .expect("credential should exist");
    assert_eq!(found.label, "GitHub");
    assert_eq!(found.username, "user@github.com");
}

#[test]
fn test_update() {
    let _db = setup_db();
    clean_db();
    let cred = test_cred("AWS", "admin@aws.com", "aws_secret");
    let mut created = crate::create(&cred).expect("create failed");
    assert_eq!(created.label, "AWS");

    created.label = "AWS-Prod".into();
    let updated = crate::update(&created).expect("update failed");
    assert_eq!(updated.label, "AWS-Prod");
    assert_eq!(updated.username, "admin@aws.com");
}

#[test]
fn test_delete() {
    let _db = setup_db();
    clean_db();
    let cred = test_cred("Delete Me", "del@test.com", "del_secret");
    let created = crate::create(&cred).expect("create failed");

    crate::delete(&created.id).expect("delete failed");
    let found = crate::find_by_id(&created.id).expect("find_by_id failed");
    assert!(found.is_none(), "credential should be deleted");
}

#[test]
fn test_find_all_ordered() {
    let _db = setup_db();
    clean_db();
    let creds = vec![
        test_cred("Zebra", "z@test.com", "z_secret"),
        test_cred("Alpha", "a@test.com", "a_secret"),
        test_cred("Beta", "b@test.com", "b_secret"),
    ];
    for c in &creds {
        let _ = crate::create(c).expect("create failed");
    }

    let all = crate::find_all().expect("find_all failed");
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].label, "Alpha");
    assert_eq!(all[1].label, "Beta");
    assert_eq!(all[2].label, "Zebra");
}
