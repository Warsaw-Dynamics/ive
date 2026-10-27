//! `ive`: on Windows a GUI program, so a start from the Start menu opens no console window.
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    #[cfg(windows)]
    attach_parent_console();
    #[cfg(windows)]
    set_app_user_model_id();
    ive::main()
}

/// The task bar groups IVE's windows under this id, and under the Start menu shortcut `install.ps1` gives the same one,
/// so a pinned IVE is the running one. gpui ignores `app_id` on Windows.
#[cfg(windows)]
const APP_USER_MODEL_ID: &str = "WarsawDynamics.IVE";

#[cfg(windows)]
fn set_app_user_model_id() {
    use windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;
    let id: Vec<u16> = APP_USER_MODEL_ID.encode_utf16().chain([0]).collect();
    // SAFETY: `id` is null-terminated and outlives the call; a failure only leaves Windows' default grouping
    unsafe { SetCurrentProcessExplicitAppUserModelID(id.as_ptr()) };
}

/// Started from a terminal, IVE prints its MCP address and errors there; started otherwise there is none to attach.
#[cfg(windows)]
fn attach_parent_console() {
    use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
    // SAFETY: no arguments to check; a failure only means IVE has no console
    unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
}
