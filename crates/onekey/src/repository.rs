use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{anyhow, Result};
use crate::db;
use crate::secret_store::OneKeySecretStore;
use crate::types::{
    OneKeyCredential, OneKeyKind, PromptTriggerRule, SendMode,
    DEFAULT_PASSWORD_ONLY_KEYWORDS, DEFAULT_USERNAME_AND_PASSWORD_KEYWORDS,
};
use chrono::Utc;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use persistence::model::OneKeyCredentialRow;
use persistence::schema::onekey_credentials;
use persistence::schema::prompt_trigger_rules;
use uuid::Uuid;
use zeroize::Zeroizing;

static CREDENTIALS_VERSION: AtomicU64 = AtomicU64::new(0);

pub fn credentials_version() -> u64 {
    CREDENTIALS_VERSION.load(Ordering::Relaxed)
}

pub fn bump_credentials_version() {
    CREDENTIALS_VERSION.fetch_add(1, Ordering::Relaxed);
}

fn row_to_onekey_credential(
    row: OneKeyCredentialRow,
    password: Zeroizing<String>,
) -> OneKeyCredential {
    OneKeyCredential {
        id: row.id,
        label: row.label,
        username: row.username,
        notes: row.notes,
        password,
        kind: OneKeyKind::parse(&row.kind).unwrap_or(OneKeyKind::Password),
        key_path: row.key_path,
    }
}

fn resolve_password(row: &OneKeyCredentialRow) -> Result<Zeroizing<String>> {
    match OneKeySecretStore::get(&row.id) {
        Ok(Some(p)) => return Ok(p),
        Ok(None) => {}
        Err(e) => log::warn!("keyring error for {}: {e}", row.id),
    }
    if !row.encrypted_password.is_empty() {
        return Ok(Zeroizing::new(row.encrypted_password.clone()));
    }
    Ok(Zeroizing::new(String::new()))
}

fn store_password(conn: &mut SqliteConnection, id: &str, password: &str) -> Result<()> {
    if let Err(e) = OneKeySecretStore::set(id, password) {
        log::warn!("keyring write error for {id}: {e}");
    }
    diesel::update(onekey_credentials::table.find(id))
        .set(onekey_credentials::encrypted_password.eq(password))
        .execute(conn)?;
    Ok(())
}

pub fn find_all() -> Result<Vec<OneKeyCredential>> {
    db::with_conn(|conn| {
        let rows: Vec<OneKeyCredentialRow> = onekey_credentials::table
            .order(onekey_credentials::label.asc())
            .load(conn)?;

        let mut credentials = Vec::new();
        for row in rows {
            match resolve_password(&row) {
                Ok(password) => {
                    credentials.push(row_to_onekey_credential(row, password));
                }
                Err(e) => {
                    log::warn!("failed to load secret for {}: {e}", row.id);
                }
            }
        }
        Ok(credentials)
    })
}

pub fn find_by_id(credential_id: &str) -> Result<Option<OneKeyCredential>> {
    db::with_conn(|conn| {
        let row: Option<OneKeyCredentialRow> = onekey_credentials::table
            .find(credential_id)
            .first(conn)
            .optional()?;

        match row {
            Some(row) => {
                let password = resolve_password(&row)?;
                Ok(Some(row_to_onekey_credential(row, password)))
            }
            None => Ok(None),
        }
    })
}

pub fn create(credential: &OneKeyCredential) -> Result<OneKeyCredential> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    db::with_conn(|conn| {
        diesel::insert_into(onekey_credentials::table)
            .values((
                onekey_credentials::id.eq(&id),
                onekey_credentials::label.eq(&credential.label),
                onekey_credentials::username.eq(&credential.username),
                onekey_credentials::notes.eq(&credential.notes),
                onekey_credentials::encrypted_password.eq(&credential.password.as_str()),
                onekey_credentials::kind.eq(credential.kind.as_db_str()),
                onekey_credentials::key_path.eq(&credential.key_path),
                onekey_credentials::created_at.eq(&now),
                onekey_credentials::updated_at.eq(&now),
            ))
            .execute(conn)?;

        store_password(conn, &id, &credential.password)?;
        bump_credentials_version();

        Ok(OneKeyCredential {
            id,
            label: credential.label.clone(),
            username: credential.username.clone(),
            notes: credential.notes.clone(),
            password: credential.password.clone(),
            kind: credential.kind,
            key_path: credential.key_path.clone(),
        })
    })
}

pub fn update(credential: &OneKeyCredential) -> Result<OneKeyCredential> {
    let now = Utc::now().to_rfc3339();

    db::with_conn(|conn| {
        let n = diesel::update(onekey_credentials::table.find(&credential.id))
            .set((
                onekey_credentials::label.eq(&credential.label),
                onekey_credentials::username.eq(&credential.username),
                onekey_credentials::notes.eq(&credential.notes),
                onekey_credentials::kind.eq(credential.kind.as_db_str()),
                onekey_credentials::key_path.eq(&credential.key_path),
                onekey_credentials::updated_at.eq(&now),
            ))
            .execute(conn)?;

        if n == 0 {
            return Err(anyhow!("credential {} not found", credential.id));
        }

        store_password(conn, &credential.id, &credential.password)?;
        bump_credentials_version();

        Ok(credential.clone())
    })
}

pub fn list_rules() -> Result<Vec<PromptTriggerRule>> {
    db::with_conn(|conn| {
        let rows: Vec<persistence::model::PromptTriggerRuleRow> =
            prompt_trigger_rules::table.load(conn)?;
        Ok(rows
            .into_iter()
            .map(|r| PromptTriggerRule {
                id: r.id,
                keyword: r.keyword,
                send_mode: if r.send_mode == "username_then_password" {
                    SendMode::UsernameThenPassword
                } else {
                    SendMode::PasswordOnly
                },
            })
            .collect())
    })
}

pub fn add_rule(keyword: &str, send_mode: SendMode) -> Result<PromptTriggerRule> {
    let id = Uuid::new_v4().to_string();
    let mode_str = send_mode.as_str();

    db::with_conn(|conn| {
        diesel::insert_into(prompt_trigger_rules::table)
            .values((
                prompt_trigger_rules::id.eq(&id),
                prompt_trigger_rules::keyword.eq(keyword),
                prompt_trigger_rules::send_mode.eq(mode_str),
            ))
            .execute(conn)?;
        Ok(PromptTriggerRule {
            id,
            keyword: keyword.to_string(),
            send_mode,
        })
    })
}

pub fn remove_rule(rule_id: &str) -> Result<()> {
    db::with_conn(|conn| {
        diesel::delete(prompt_trigger_rules::table.find(rule_id))
            .execute(conn)?;
        Ok(())
    })
}

pub fn reset_rules_for_mode(mode: SendMode) -> Result<()> {
    db::with_conn(|conn| {
        let mode_str = mode.as_str();
        diesel::delete(prompt_trigger_rules::table.filter(
            prompt_trigger_rules::send_mode.eq(mode_str),
        ))
        .execute(conn)?;
        let keywords = match mode {
            SendMode::PasswordOnly => DEFAULT_PASSWORD_ONLY_KEYWORDS,
            SendMode::UsernameThenPassword => DEFAULT_USERNAME_AND_PASSWORD_KEYWORDS,
        };
        for kw in keywords {
            let id = Uuid::new_v4().to_string();
            diesel::insert_into(prompt_trigger_rules::table)
                .values((
                    prompt_trigger_rules::id.eq(&id),
                    prompt_trigger_rules::keyword.eq(kw),
                    prompt_trigger_rules::send_mode.eq(mode_str),
                ))
                .execute(conn)?;
        }
        Ok(())
    })
}

pub fn reset_rules_to_defaults() -> Result<()> {
    db::with_conn(|conn| {
        conn.batch_execute("DELETE FROM prompt_trigger_rules")?;
        for kw in DEFAULT_PASSWORD_ONLY_KEYWORDS {
            let id = Uuid::new_v4().to_string();
            diesel::insert_into(prompt_trigger_rules::table)
                .values((
                    prompt_trigger_rules::id.eq(&id),
                    prompt_trigger_rules::keyword.eq(kw),
                    prompt_trigger_rules::send_mode.eq("password_only"),
                ))
                .execute(conn)?;
        }
        for kw in DEFAULT_USERNAME_AND_PASSWORD_KEYWORDS {
            let id = Uuid::new_v4().to_string();
            diesel::insert_into(prompt_trigger_rules::table)
                .values((
                    prompt_trigger_rules::id.eq(&id),
                    prompt_trigger_rules::keyword.eq(kw),
                    prompt_trigger_rules::send_mode.eq("username_then_password"),
                ))
                .execute(conn)?;
        }
        Ok(())
    })
}

pub fn delete(credential_id: &str) -> Result<()> {
    db::with_conn(|conn| {
        let n = diesel::delete(onekey_credentials::table.find(credential_id))
            .execute(conn)?;

        if n == 0 {
            return Err(anyhow!("credential {credential_id} not found"));
        }

        let _ = OneKeySecretStore::delete(credential_id);
        bump_credentials_version();
        Ok(())
    })
}

#[cfg(test)]
#[path = "repository_tests.rs"]
mod tests;
