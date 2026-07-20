use anyhow::Result;
use keyring::Entry;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use zeroize::Zeroizing;

const SERVICE: &str = "zap.quick-credential";

static FALLBACK_STORE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn fallback_store() -> &'static Mutex<HashMap<String, String>> {
    FALLBACK_STORE.get_or_init(|| Mutex::new(HashMap::new()))
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
        Ok(())
    }
}
