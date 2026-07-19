use anyhow::{Context, Result};
use keyring::Entry;
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::{Mutex, OnceLock};
use zeroize::Zeroizing;

const SERVICE: &str = "zap.quick-credential";

#[cfg(test)]
static TEST_STORE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

#[cfg(test)]
fn test_store() -> &'static Mutex<HashMap<String, String>> {
    TEST_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
pub(crate) fn clear_test_store() {
    test_store().lock().unwrap().clear();
}

pub struct QuickCredentialSecretStore;

#[cfg_attr(test, allow(unreachable_code))]
impl QuickCredentialSecretStore {
    pub fn set(id: &str, secret: &str) -> Result<()> {
        #[cfg(test)]
        {
            test_store()
                .lock()
                .unwrap()
                .insert(id.to_string(), secret.to_string());
            return Ok(());
        }

        let entry =
            Entry::new(SERVICE, &format!("{id}:password")).context("keyring entry error")?;
        entry.set_password(secret).context("keyring set error")?;
        Ok(())
    }

    pub fn get(id: &str) -> Result<Option<Zeroizing<String>>> {
        #[cfg(test)]
        {
            return Ok(test_store()
                .lock()
                .unwrap()
                .get(id)
                .map(|s| Zeroizing::new(s.clone())));
        }

        let entry =
            Entry::new(SERVICE, &format!("{id}:password")).context("keyring entry error")?;
        match entry.get_password() {
            Ok(p) => Ok(Some(Zeroizing::new(p))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e).context("keyring get error"),
        }
    }

    pub fn delete(id: &str) -> Result<()> {
        #[cfg(test)]
        {
            test_store().lock().unwrap().remove(id);
            return Ok(());
        }

        let entry =
            Entry::new(SERVICE, &format!("{id}:password")).context("keyring entry error")?;
        entry.delete_credential().context("keyring delete error")?;
        Ok(())
    }
}
