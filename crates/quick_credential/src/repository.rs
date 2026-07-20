use anyhow::{anyhow, Result};
use crate::db;
use crate::secret_store::QuickCredentialSecretStore;
use crate::types::{QuickCredential, SendMode};
use chrono::Utc;
use diesel::prelude::*;
use persistence::model::QuickCredentialRow;
use persistence::schema::quick_credentials;
use uuid::Uuid;
use zeroize::Zeroizing;

fn row_to_credential(
    row: QuickCredentialRow,
    password: Zeroizing<String>,
) -> QuickCredential {
    QuickCredential {
        id: row.id,
        label: row.label,
        username: row.username,
        send_mode: SendMode::from_str(&row.send_mode),
        notes: row.notes,
        password,
    }
}

fn resolve_password(row: &QuickCredentialRow) -> Result<Option<Zeroizing<String>>> {
    match QuickCredentialSecretStore::get(&row.id) {
        Ok(Some(p)) => return Ok(Some(p)),
        Ok(None) => {}
        Err(e) => log::warn!("keyring error for {}: {e}", row.id),
    }
    if !row.encrypted_password.is_empty() {
        return Ok(Some(Zeroizing::new(row.encrypted_password.clone())));
    }
    Ok(None)
}

fn store_password(conn: &mut SqliteConnection, id: &str, password: &str) -> Result<()> {
    match QuickCredentialSecretStore::set(id, password) {
        Ok(()) => return Ok(()),
        Err(e) => log::warn!("keyring write error for {id}: {e}, falling back to sqlite"),
    }
    diesel::update(quick_credentials::table.find(id))
        .set(quick_credentials::encrypted_password.eq(password))
        .execute(conn)?;
    Ok(())
}

pub fn find_all() -> Result<Vec<QuickCredential>> {
    db::with_conn(|conn| {
        let rows: Vec<QuickCredentialRow> = quick_credentials::table
            .order(quick_credentials::label.asc())
            .load(conn)?;

        let mut credentials = Vec::new();
        for row in rows {
            match resolve_password(&row) {
                Ok(Some(password)) => {
                    credentials.push(row_to_credential(row, password));
                }
                Ok(None) => {}
                Err(e) => {
                    log::warn!("failed to load secret for {}: {e}", row.id);
                }
            }
        }
        Ok(credentials)
    })
}

pub fn find_by_id(credential_id: &str) -> Result<Option<QuickCredential>> {
    db::with_conn(|conn| {
        let row: Option<QuickCredentialRow> = quick_credentials::table
            .find(credential_id)
            .first(conn)
            .optional()?;

        match row {
            Some(row) => {
                let password = resolve_password(&row)?
                    .ok_or_else(|| anyhow!("secret not found for {}", row.id))?;
                Ok(Some(row_to_credential(row, password)))
            }
            None => Ok(None),
        }
    })
}

pub fn create(credential: &QuickCredential) -> Result<QuickCredential> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    db::with_conn(|conn| {
        diesel::insert_into(quick_credentials::table)
            .values((
                quick_credentials::id.eq(&id),
                quick_credentials::label.eq(&credential.label),
                quick_credentials::username.eq(&credential.username),
                quick_credentials::send_mode.eq(credential.send_mode.as_str()),
                quick_credentials::notes.eq(&credential.notes),
                quick_credentials::created_at.eq(&now),
                quick_credentials::updated_at.eq(&now),
            ))
            .execute(conn)?;

        store_password(conn, &id, &credential.password)?;

        Ok(QuickCredential {
            id,
            label: credential.label.clone(),
            username: credential.username.clone(),
            send_mode: credential.send_mode.clone(),
            notes: credential.notes.clone(),
            password: credential.password.clone(),
        })
    })
}

pub fn update(credential: &QuickCredential) -> Result<QuickCredential> {
    let now = Utc::now().to_rfc3339();

    db::with_conn(|conn| {
        let n = diesel::update(quick_credentials::table.find(&credential.id))
            .set((
                quick_credentials::label.eq(&credential.label),
                quick_credentials::username.eq(&credential.username),
                quick_credentials::send_mode.eq(credential.send_mode.as_str()),
                quick_credentials::notes.eq(&credential.notes),
                quick_credentials::updated_at.eq(&now),
            ))
            .execute(conn)?;

        if n == 0 {
            return Err(anyhow!("credential {} not found", credential.id));
        }

        store_password(conn, &credential.id, &credential.password)?;

        Ok(credential.clone())
    })
}

pub fn delete(credential_id: &str) -> Result<()> {
    db::with_conn(|conn| {
        let n = diesel::delete(quick_credentials::table.find(credential_id))
            .execute(conn)?;

        if n == 0 {
            return Err(anyhow!("credential {credential_id} not found"));
        }

        let _ = QuickCredentialSecretStore::delete(credential_id);
        Ok(())
    })
}

#[cfg(test)]
#[path = "repository_tests.rs"]
mod tests;
