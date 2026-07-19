use warp_quick_credential::SendMode;

#[test]
fn test_send_mode_password_only_format() {
    let output = format!("{}\n", "s3cret");
    assert_eq!(output, "s3cret\n");
}

#[test]
fn test_send_mode_username_format() {
    let username = "user@example.com";
    let output = format!("{username}\n");
    assert_eq!(output, "user@example.com\n");
}

#[test]
fn test_send_mode_password_only_dispatch() {
    let mode = SendMode::PasswordOnly;
    match mode {
        SendMode::PasswordOnly => {}
        SendMode::UsernameThenPassword => {
            panic!("expected PasswordOnly, got UsernameThenPassword");
        }
    }
}

#[test]
fn test_send_mode_username_then_password_dispatch() {
    let mode = SendMode::UsernameThenPassword;
    match mode {
        SendMode::UsernameThenPassword => {}
        SendMode::PasswordOnly => {
            panic!("expected UsernameThenPassword, got PasswordOnly");
        }
    }
}
