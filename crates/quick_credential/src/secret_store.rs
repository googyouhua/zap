use anyhow::{Context, Result};
use keyring::Entry;
use zeroize::Zeroizing;

const SERVICE: &str = "zap.quick-credential";

pub struct QuickCredentialSecretStore;

impl QuickCredentialSecretStore {
    pub fn set(id: &str, secret: &str) -> Result<()> {
        let entry =
            Entry::new(SERVICE, &format!("{id}:password")).context("keyring entry error")?;
        entry.set_password(secret).context("keyring set error")?;
        Ok(())
    }

    pub fn get(id: &str) -> Result<Option<Zeroizing<String>>> {
        let entry =
            Entry::new(SERVICE, &format!("{id}:password")).context("keyring entry error")?;
        match entry.get_password() {
            Ok(p) => Ok(Some(Zeroizing::new(p))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e).context("keyring get error"),
        }
    }

    pub fn delete(id: &str) -> Result<()> {
        let entry =
            Entry::new(SERVICE, &format!("{id}:password")).context("keyring entry error")?;
        entry.delete_credential().context("keyring delete error")?;
        Ok(())
    }
}
