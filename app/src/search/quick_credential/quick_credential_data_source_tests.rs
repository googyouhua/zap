use super::filter_credentials;
use warp_quick_credential::{QuickCredential, SendMode};
use zeroize::Zeroizing;

fn cred(label: &str, username: &str) -> QuickCredential {
    QuickCredential {
        id: String::new(),
        label: label.into(),
        username: username.into(),
        send_mode: SendMode::PasswordOnly,
        notes: String::new(),
        password: Zeroizing::new("pwd".into()),
    }
}

#[test]
fn test_filter_by_label() {
    let creds = vec![cred("GitHub", "user@gh.com"), cred("AWS", "admin@aws.com")];
    let result = filter_credentials(&creds, "git");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].label, "GitHub");
}

#[test]
fn test_filter_by_username() {
    let creds = vec![cred("GitHub", "user@gh.com"), cred("AWS", "admin@aws.com")];
    let result = filter_credentials(&creds, "admin");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].label, "AWS");
}

#[test]
fn test_empty_query_returns_all() {
    let creds = vec![cred("A", "a@a.com"), cred("B", "b@b.com")];
    let result = filter_credentials(&creds, "");
    assert_eq!(result.len(), 2);
}

#[test]
fn test_no_match_returns_empty() {
    let creds = vec![cred("GitHub", "user@gh.com")];
    let result = filter_credentials(&creds, "nonexistent");
    assert!(result.is_empty());
}

#[test]
fn test_partial_label_match() {
    let creds = vec![cred("My GitHub Account", "user@gh.com")];
    let result = filter_credentials(&creds, "Git");
    assert_eq!(result.len(), 1);
}

#[test]
fn test_case_insensitive_label_match() {
    let creds = vec![cred("GitHub Production", "admin@gh.com")];
    let result = filter_credentials(&creds, "github");
    assert_eq!(result.len(), 1);
}
