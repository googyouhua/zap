use zeroize::Zeroizing;

#[derive(Debug, Clone)]
pub struct QuickCredential {
    pub id: String,
    pub label: String,
    pub username: String,
    pub notes: String,
    pub password: Zeroizing<String>,
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
