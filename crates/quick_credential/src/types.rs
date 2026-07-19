use zeroize::Zeroizing;

#[derive(Debug, Clone)]
pub struct QuickCredential {
    pub id: String,
    pub label: String,
    pub username: String,
    pub send_mode: SendMode,
    pub notes: String,
    pub password: Zeroizing<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SendMode {
    PasswordOnly,
    UsernameThenPassword,
}

impl SendMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "username_then_password" => SendMode::UsernameThenPassword,
            _ => SendMode::PasswordOnly,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SendMode::PasswordOnly => "password_only",
            SendMode::UsernameThenPassword => "username_then_password",
        }
    }
}
