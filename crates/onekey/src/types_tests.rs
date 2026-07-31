use super::*;

#[test]
fn test_onekey_kind_as_db_str() {
    assert_eq!(OneKeyKind::Password.as_db_str(), "password");
    assert_eq!(OneKeyKind::Key.as_db_str(), "key");
}

#[test]
fn test_onekey_kind_parse_valid() {
    assert_eq!(OneKeyKind::parse("password"), Some(OneKeyKind::Password));
    assert_eq!(OneKeyKind::parse("key"), Some(OneKeyKind::Key));
}

#[test]
fn test_onekey_kind_parse_invalid() {
    assert_eq!(OneKeyKind::parse("invalid"), None);
    assert_eq!(OneKeyKind::parse(""), None);
}

#[test]
fn test_onekey_kind_parse_case_sensitive() {
    assert_eq!(OneKeyKind::parse("PASSWORD"), None);
    assert_eq!(OneKeyKind::parse("KEY"), None);
}

#[test]
fn test_onekey_credential_default_fields() {
    let cred = OneKeyCredential {
        id: String::new(),
        label: "test".into(),
        username: "user".into(),
        notes: String::new(),
        password: Zeroizing::new("secret".into()),
        kind: OneKeyKind::Password,
        key_path: None,
    };
    assert_eq!(cred.kind, OneKeyKind::Password);
    assert_eq!(cred.key_path, None);
}

#[test]
fn test_onekey_credential_with_key_path() {
    let cred = OneKeyCredential {
        id: String::new(),
        label: "SSH Key".into(),
        username: "git".into(),
        notes: String::new(),
        password: Zeroizing::new("".into()),
        kind: OneKeyKind::Key,
        key_path: Some("/home/user/.ssh/id_rsa".into()),
    };
    assert_eq!(cred.kind, OneKeyKind::Key);
    assert_eq!(cred.key_path.as_deref(), Some("/home/user/.ssh/id_rsa"));
}
