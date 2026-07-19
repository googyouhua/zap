use keyring::Entry;
use zeroize::Zeroizing;

const SERVICE: &str = "zap.quick-credential";

pub struct QuickCredentialSecretStore;

impl QuickCredentialSecretStore {
    pub fn set(id: &str, secret: &str) -> Result<(), String> {
        let entry = Entry::new(SERVICE, &format!("{id}:password"))
            .map_err(|e| format!("keyring entry error: {e}"))?;
        entry.set_password(secret)
            .map_err(|e| format!("keyring set error: {e}"))
    }

    pub fn get(id: &str) -> Result<Option<Zeroizing<String>>, String> {
        let entry = Entry::new(SERVICE, &format!("{id}:password"))
            .map_err(|e| format!("keyring entry error: {e}"))?;
        match entry.get_password() {
            Ok(p) => Ok(Some(Zeroizing::new(p))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(format!("keyring get error: {e}")),
        }
    }

    pub fn delete(id: &str) -> Result<(), String> {
        let entry = Entry::new(SERVICE, &format!("{id}:password"))
            .map_err(|e| format!("keyring entry error: {e}"))?;
        entry.delete_credential()
            .map_err(|e| format!("keyring delete error: {e}"))
    }
}
