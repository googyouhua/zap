mod db;
pub mod repository;
mod secret_store;
mod types;

pub use db::{set_database_path, with_conn};
#[cfg(test)]
pub use db::set_test_conn;
pub use repository::{
    add_rule, create, delete, find_all, find_by_id, list_rules, remove_rule,
    reset_rules_to_defaults, update,
};
pub use secret_store::QuickCredentialSecretStore;
pub use types::{
    PromptTriggerRule, QuickCredential, SendMode,
    DEFAULT_PASSWORD_ONLY_KEYWORDS, DEFAULT_USERNAME_AND_PASSWORD_KEYWORDS,
};
