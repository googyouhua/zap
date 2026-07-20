use anyhow::Result;
use keyring::Entry;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use zeroize::Zeroizing;

const SERVICE: &str = "zap.quick-credential";

static FALLBACK_STORE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
static PASSWORD_FILE: OnceLock<PathBuf> = OnceLock::new();

fn fallback_store() -> &'static Mutex<HashMap<String, String>> {
    FALLBACK_STORE.get_or_init(|| {
        let map = if let Some(path) = PASSWORD_FILE.get() {
            load_password_file(path).unwrap_or_default()
        } else {
            HashMap::new()
        };
        Mutex::new(map)
    })
}

fn load_password_file(path: &PathBuf) -> Option<HashMap<String, String>> {
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_password_file() {
    let path = match PASSWORD_FILE.get() {
        Some(p) => p,
        None => return,
    };
    let guard = match FALLBACK_STORE.get().and_then(|s| s.lock().ok()) {
        Some(g) => g,
        None => return,
    };
    if let Ok(data) = serde_json::to_string(&*guard) {
        drop(guard);
        let _ = std::fs::write(path, data);
    }
}

pub fn set_password_file_path(db_path: &PathBuf) {
    let pw_path = db_path.with_extension("sqlite.passwords");
    let _ = PASSWORD_FILE.set(pw_path);
}

#[cfg(test)]
pub(crate) fn clear_fallback_store() {
    if let Some(store) = FALLBACK_STORE.get() {
        store.lock().unwrap().clear();
    }
}

pub struct QuickCredentialSecretStore;

impl QuickCredentialSecretStore {
    pub fn set(id: &str, secret: &str) -> Result<()> {
        let key = format!("{id}:password");
        match Entry::new(SERVICE, &key) {
            Ok(entry) => {
                if entry.set_password(secret).is_ok() {
                    return Ok(());
                }
            }
            Err(_) => {}
        }
        fallback_store()
            .lock()
            .map_err(|e| anyhow::anyhow!("fallback store poisoned: {e}"))?
            .insert(id.to_string(), secret.to_string());
        save_password_file();
        Ok(())
    }

    pub fn get(id: &str) -> Result<Option<Zeroizing<String>>> {
        let key = format!("{id}:password");
        if let Ok(entry) = Entry::new(SERVICE, &key) {
            match entry.get_password() {
                Ok(p) => return Ok(Some(Zeroizing::new(p))),
                Err(keyring::Error::NoEntry) => {}
                Err(_) => {}
            }
        }
        let guard = fallback_store()
            .lock()
            .map_err(|e| anyhow::anyhow!("fallback store poisoned: {e}"))?;
        Ok(guard.get(id).map(|s| Zeroizing::new(s.clone())))
    }

    pub fn delete(id: &str) -> Result<()> {
        let key = format!("{id}:password");
        if let Ok(entry) = Entry::new(SERVICE, &key) {
            let _ = entry.delete_credential();
        }
        let mut guard = fallback_store()
            .lock()
            .map_err(|e| anyhow::anyhow!("fallback store poisoned: {e}"))?;
        guard.remove(id);
        drop(guard);
        save_password_file();
        Ok(())
    }
}
