use zeroize::Zeroizing;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OneKeyKind {
    Password,
    Key,
}

impl OneKeyKind {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            OneKeyKind::Password => "password",
            OneKeyKind::Key => "key",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "password" => Some(OneKeyKind::Password),
            "key" => Some(OneKeyKind::Key),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OneKeyCredential {
    pub id: String,
    pub label: String,
    pub username: String,
    pub notes: String,
    pub password: Zeroizing<String>,
    pub kind: OneKeyKind,
    pub key_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SendMode {
    PasswordOnly,
    UsernameThenPassword,
}

impl SendMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SendMode::PasswordOnly => "password_only",
            SendMode::UsernameThenPassword => "username_then_password",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PromptTriggerRule {
    pub id: String,
    pub keyword: String,
    pub send_mode: SendMode,
}

pub const DEFAULT_PASSWORD_ONLY_KEYWORDS: &[&str] = &["password", "passphrase"];
pub const DEFAULT_USERNAME_AND_PASSWORD_KEYWORDS: &[&str] =
    &["login", "username", "user", "name", "email", "account"];

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
