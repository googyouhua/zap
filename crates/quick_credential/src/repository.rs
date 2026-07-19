use crate::types::QuickCredential;

pub struct QuickCredentialRepository;

impl QuickCredentialRepository {
    pub fn list() -> Result<Vec<QuickCredential>, String> {
        unimplemented!()
    }

    pub fn get(id: &str) -> Result<Option<QuickCredential>, String> {
        let _ = id;
        unimplemented!()
    }

    pub fn create(credential: &QuickCredential) -> Result<QuickCredential, String> {
        let _ = credential;
        unimplemented!()
    }

    pub fn update(credential: &QuickCredential) -> Result<QuickCredential, String> {
        let _ = credential;
        unimplemented!()
    }

    pub fn delete(id: &str) -> Result<(), String> {
        let _ = id;
        unimplemented!()
    }
}

pub fn load_saved_quick_credentials() -> Result<Vec<QuickCredential>, String> {
    QuickCredentialRepository::list()
}
