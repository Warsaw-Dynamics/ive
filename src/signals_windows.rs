//! Windows' stand-in for SIGUSR1: `restart.ps1` sets the named event `Local\ive-replace-<pid>` once the new IVE is
//! built, and this IVE quits as replaced, leaving its terminals to the new one. Closing the window is no substitute:
//! with terminals running it asks first.

use std::io;

use futures::channel::mpsc::{UnboundedReceiver, unbounded};
use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
use windows_sys::Win32::System::Threading::{CreateEventW, INFINITE, WaitForSingleObject};

/// What the receiver gets when the event is set, in place of a signal number.
pub const REPLACED: i32 = -1;

/// Waits for the event from now on and forwards each setting as [`REPLACED`].
pub fn forward() -> io::Result<UnboundedReceiver<i32>> {
    let name: Vec<u16> = format!("Local\\ive-replace-{}", std::process::id()).encode_utf16().chain([0]).collect();
    // SAFETY: an auto-reset event, not set, under a NUL-terminated name, without security attributes
    let event = unsafe { CreateEventW(std::ptr::null(), 0, 0, name.as_ptr()) };
    if event.is_null() {
        return Err(io::Error::last_os_error());
    }
    // a HANDLE is not Send; the event stays open, and so valid, for the rest of the process
    let event = event as usize;
    let (sender, receiver) = unbounded();
    std::thread::Builder::new().name("ive-replace-event".to_owned()).spawn(move || {
        // SAFETY: the handle is open for good, see above
        while unsafe { WaitForSingleObject(event as _, INFINITE) } == WAIT_OBJECT_0 {
            if sender.unbounded_send(REPLACED).is_err() {
                return;
            }
        }
    })?;
    Ok(receiver)
}
