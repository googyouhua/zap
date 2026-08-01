use super::bytes_look_like_onekey_prompt;
use super::bytes_look_like_password_prompt;

fn matches(input: &str) -> bool {
    bytes_look_like_password_prompt(input.as_bytes())
}

fn matches_onekey(input: &str) -> bool {
    bytes_look_like_onekey_prompt(input.as_bytes())
}

#[test]
fn matches_typical_password_prompt() {
    assert!(matches("user@host's password: "));
    assert!(matches("Password:"));
    assert!(matches("password: \r\n"));
}

#[test]
fn matches_sudo_password_prompt() {
    assert!(matches("[sudo] password for alice: "));
}

#[test]
fn matches_passphrase_prompt() {
    assert!(matches("Enter passphrase for key '/home/u/.ssh/id_rsa': "));
}

#[test]
fn does_not_match_motd_with_password_word() {
    assert!(!matches("Welcome! Please change your password soon.\n# "));
    assert!(!matches(
        "Last login: Mon Jan 1 password rotated yesterday\n"
    ));
}

#[test]
fn does_not_match_no_colon() {
    assert!(!matches("password\n"));
    assert!(!matches("Enter password please\n"));
}

#[test]
fn onekey_matches_password_prompt() {
    assert!(matches_onekey("user@host's password: "));
    assert!(matches_onekey("[sudo] password for alice: "));
}

#[test]
fn onekey_matches_username_prompt() {
    assert!(matches_onekey("login as: "));
    assert!(matches_onekey("Username:"));
    assert!(matches_onekey("user name: "));
    assert!(matches_onekey("Email address: "));
    assert!(matches_onekey("account: "));
}

#[test]
fn onekey_does_not_match_non_prompt_text() {
    assert!(!matches_onekey("Welcome! Please change your password soon.\n# "));
    assert!(!matches_onekey("password\n"));
    assert!(!matches_onekey("Last login: Mon Jan 1 password rotated yesterday\n"));
}

#[test]
fn strict_password_prompt_rejects_username_prompt() {
    assert!(!matches("login as: "));
    assert!(!matches("Username:"));
}
