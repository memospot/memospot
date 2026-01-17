use log::debug;
use std::os::windows::process::CommandExt;
use std::process::Command as StdCommand;
use windows_sys::Win32::System::Console::{
    AttachConsole, CTRL_BREAK_EVENT, CTRL_C_EVENT, FreeConsole, GenerateConsoleCtrlEvent,
    SetConsoleCtrlHandler,
};

const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub trait CommandCreationFlagsExt {
    fn apply_creation_flags(&mut self) -> &mut Self;
}

impl CommandCreationFlagsExt for StdCommand {
    /// Configures the command to run in a new process group and without a window.
    fn apply_creation_flags(&mut self) -> &mut Self {
        self.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW)
    }
}

/// Windows control handler function that ignores console CTRL+C and CTRL+BREAK events.
unsafe extern "system" fn ctrl_handler(ctrl_type: u32) -> i32 {
    // Return 1 to indicate we handled the event (prevent default behavior)
    match ctrl_type {
        CTRL_C_EVENT | CTRL_BREAK_EVENT => 1,
        _ => 0,
    }
}

/// Sends CTRL+BREAK to the console process group of the given process.
pub fn send_ctrl_break(pid: u32) {
    unsafe {
        // Detach from any current console.
        FreeConsole();

        // Attach to the console of the target process.
        if AttachConsole(pid) != 0 {
            debug!("sidecar: successfully attached to console of pid {}", pid);

            // Install a control handler that ignores CTRL+C and CTRL+BREAK events.
            // This prevents the current process from being terminated.
            if SetConsoleCtrlHandler(Some(ctrl_handler), 1) == 0 {
                debug!(
                    "sidecar: failed to set console control handler, error: {}",
                    windows_sys::Win32::Foundation::GetLastError()
                );
            }

            // Send CTRL+BREAK to all processes in the console (group 0)
            // This is more reliable than trying to target a specific process group
            if GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, 0) == 0 {
                debug!(
                    "sidecar: failed to generate console ctrl event, error: {}",
                    windows_sys::Win32::Foundation::GetLastError()
                );
            } else {
                debug!("sidecar: successfully sent CTRL+BREAK to console process");
            }

            // Detach from the console.
            FreeConsole();

            // Remove our control handler.
            SetConsoleCtrlHandler(Some(ctrl_handler), 0);
        } else {
            let error = windows_sys::Win32::Foundation::GetLastError();
            debug!(
                "sidecar: failed to attach to console of pid {}, error: {}, falling back to kill",
                pid, error
            );
        }
    }
}
