/// Environment setup.
use config::Config;
use log::debug;
use std::env;

#[cfg(target_os = "linux")]
use log::{info, warn};
#[cfg(target_os = "linux")]
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::path::Path;

#[cfg(target_os = "linux")]
use std::{
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::Duration,
};

#[cfg(target_os = "linux")]
/// Set up WebKit2GTK hardware acceleration.
///
/// There are known issues with hardware acceleration on Nvidia GPUs.
/// See: https://github.com/tauri-apps/tauri/issues/9394.
///
/// This function mitigates those issues by preemptively setting the following environment variables heuristically:
/// - `WEBKIT_DISABLE_COMPOSITING_MODE=1`
/// - `WEBKIT_DISABLE_DMABUF_RENDERER=1`
/// - `__NV_DISABLE_EXPLICIT_SYNC=1`
/// - `GDK_BACKEND=x11` (AppImage + Wayland fallback)
///
/// The variables are only set for the current process, leaving the system untouched.
pub fn setup_hw_acceleration() {
    let mut vars: HashMap<String, String> = HashMap::new();

    if !Path::new("/dev/dri").exists() {
        warn!("No GPU renderer was detected.");
        vars.insert("WEBKIT_DISABLE_COMPOSITING_MODE".into(), "1".into());
    }

    let is_x11 = env::var("WAYLAND_DISPLAY").is_err()
        && env::var("XDG_SESSION_TYPE").unwrap_or_default() == "x11";

    // On some Wayland setups the AppImage fails to create an EGL display.
    // Fallback to X11 when running from AppImage unless the user overrides.
    let is_appimage = std::env::var("APPIMAGE").is_ok();
    if is_appimage && !is_x11 {
        warn!(
            "Running from AppImage under Wayland. Forcing X11 backend for WebKitGTK with `GDK_BACKEND=x11`."
        );
        vars.insert("GDK_BACKEND".into(), "x11".into());
    }

    let is_flatpak = env::var("FLATPAK_ID").is_ok();

    // This will return false regardless inside a Flatpak sandbox.
    let has_nvidia = || {
        if Path::new("/proc/driver/nvidia/version").exists() {
            return true;
        }
        if Command::new("nvidia-smi")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
        {
            return true;
        }

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let detected = Command::new("lshw")
                .args([
                    "-quiet", "-short", "-disable", "disk", "-disable", "volume", "-disable",
                    "usb", "-disable", "scsi", "-disable", "pnp", "-c", "display",
                ])
                .stdout(Stdio::piped())
                .output()
                .map(|cmd| {
                    let stdout = String::from_utf8_lossy(&cmd.stdout).to_lowercase();
                    ["nvidia", "geforce", "quadro", "rtx"]
                        .iter()
                        .any(|find| stdout.contains(find))
                })
                .unwrap_or(false);
            let _ = tx.send(detected);
        });

        const GPU_DETECTION_TIMEOUT_MS: u64 = 1_500;
        rx.recv_timeout(Duration::from_millis(GPU_DETECTION_TIMEOUT_MS))
            .unwrap_or(false)
    };

    if is_flatpak || has_nvidia() {
        vars.insert("WEBKIT_DISABLE_DMABUF_RENDERER".into(), "1".into());
        vars.insert("__NV_DISABLE_EXPLICIT_SYNC".into(), "1".into());
    }

    for (k, v) in vars {
        match env::var(&k) {
            Ok(ref current) if current == &v => continue,
            Ok(val) => {
                info!(
                    "hardware acceleration: `{k}={val}` is already defined and won't be overridden automatically. Remove it to let Memospot set `{k}={v}`."
                );
                continue;
            }
            Err(_) => {}
        }

        warn!(
            "hardware acceleration: setting environment variable `{k}={v}`. You can remove this variable by setting `{k}=`."
        );
        // SAFETY: There's potential for race conditions when setting environment
        // variables in a multithreaded context. Shouldn't be an issue here.
        unsafe {
            env::set_var(k, v);
        }
    }
}

/// Set Memospot environment variables.
///
/// This is intended to configure the WebView on edge cases, like passing
/// WEBKIT_DISABLE_COMPOSITING_MODE=1 to disable hardware acceleration on Linux.
///
/// Should be called after init::hw_acceleration() to allow user-defined overrides.
pub fn set_env_vars(config: &Config) {
    #[cfg(target_os = "linux")]
    setup_hw_acceleration();

    if !config.memospot.env.enabled.unwrap_or(false) {
        return;
    }

    if let Some(memospot_env) = &config.memospot.env.vars {
        for (key, value) in memospot_env {
            // SAFETY: There's potential for race conditions when setting environment
            // variables in a multithreaded context. Shouldn't be an issue here.
            unsafe {
                // Some software treats *any* value in an env var as "enabled".
                if value.is_empty() {
                    debug!("removing environment variable: {}", key);
                    env::remove_var(key);
                    continue;
                }
                debug!("setting environment variable: {}={}", key, value);
                env::set_var(key, value);
            }
        }
    }
}
