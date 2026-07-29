use crate::terminal::view::TerminalView;
use std::time::Duration;
use warp_quick_credential::{QuickCredential, SendMode};
use warpui::ViewContext;
use warpui::r#async::Timer;

pub fn send_quick_credential(
    terminal_view: &mut TerminalView,
    credential: &QuickCredential,
    mode: SendMode,
    ctx: &mut ViewContext<TerminalView>,
) {
    match mode {
        SendMode::PasswordOnly => {
            terminal_view.write_to_pty(
                format!("{}\n", *credential.password).into_bytes(),
                ctx,
            );
        }
        SendMode::UsernameThenPassword => {
            terminal_view.clear_line_editor_and_write_to_pty(
                format!("{}\n", credential.username).into_bytes(),
                ctx,
            );
            let password = credential.password.clone();
            ctx.spawn(
                Timer::after(Duration::from_millis(150)),
                move |me, _, ctx| {
                    me.write_to_pty(format!("{}\n", *password).into_bytes(), ctx);
                },
            );
        }
    }
}

#[cfg(test)]
#[path = "quick_credential_sender_tests.rs"]
mod tests;
