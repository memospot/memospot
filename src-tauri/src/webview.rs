use std::env::consts;
use std::io::{Error, ErrorKind, Result};
use std::process::Command;

#[cfg(windows)]
use {
    std::io::{BufWriter, Cursor, Write},
    tauri_plugin_http::reqwest,
    tempfile::TempDir,
    winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    winreg::RegKey,
};

#[cfg(windows)]
pub fn is_available() -> bool {
    const KEY_WOW64: &str = r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
    const KEY: &str =
        r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    !(hklm.open_subkey(KEY_WOW64).is_err()
        && hkcu.open_subkey(KEY_WOW64).is_err()
        && hklm.open_subkey(KEY).is_err()
        && hkcu.open_subkey(KEY).is_err())
}

#[cfg(not(windows))]
pub fn is_available() -> bool {
    true
}

pub fn open_install_website() -> Result<()> {
    const TAURI_WEBVIEW_REF: &str = "https://tauri.app/v1/references/webview-versions/";
    const WINDOWS_WEBVIEW_URL: &str =
        "https://developer.microsoft.com/microsoft-edge/webview2/#download-section";

    match consts::OS {
        "linux" => Command::new("xdg-open").arg(TAURI_WEBVIEW_REF).spawn(),
        "windows" => Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", WINDOWS_WEBVIEW_URL])
            .spawn(),
        "macos" => Command::new("open").arg(TAURI_WEBVIEW_REF).spawn(),
        _ => Err(Error::new(ErrorKind::Other, "unsupported operating system")),
    }
    .map(|_| ())
}

#[cfg(windows)]
pub async fn install() -> Result<()> {
    const WEBVIEW2_BOOTSTRAPPER_URL: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";
    const DEFAULT_FILENAME: &str = "MicrosoftEdgeWebview2Setup.exe";

    let client = reqwest::Client::builder()
        .user_agent("Tauri")
        .gzip(true)
        .build()
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    let response = client
        .get(WEBVIEW2_BOOTSTRAPPER_URL)
        .send()
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("unable to download WebView2 installer:\n{}", e),
            )
        })?;

    if !response.status().is_success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "unable to download WebView2 installer: server responded `{}`:\n",
                response.status()
            ),
        ));
    }

    let mut filename = DEFAULT_FILENAME.to_owned();
    // get filename from response headers
    if let Some(value) = response.headers().get("content-disposition") {
        if let Ok(value) = value.to_str() {
            if let Some(last) = value.split("filename=").last() {
                let name = last.trim().replace('\"', "");
                if !&name.is_empty() {
                    filename = name;
                }
            }
        }
    } else {
        // get filename from url
        if let Some(name) = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
        {
            if !name.is_empty() {
                filename = name.to_string();
            }
        }
    }

    let tmp_dir = TempDir::with_prefix("WebView-setup-")?;
    let installer_path = tmp_dir.path().join(filename);

    let mut content = Cursor::new(response.bytes().await.map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("unable to download WebView2 installer:\n{}", e),
        )
    })?);

    let file = std::fs::File::create(&installer_path)?;
    let mut writer = BufWriter::new(file);
    std::io::copy(&mut content, &mut writer)?;
    Write::flush(&mut writer)?;
    drop(content);
    drop(writer);

    let mut child = Command::new(installer_path).args(["/install"]).spawn()?;
    let status = child.wait()?;

    if let Some(code) = status.code() {
        if code != 0 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("installer exited with code `{}`.", code),
            ));
        }
    }

    Ok(())
}

#[cfg(not(windows))]
pub async fn install() -> Result<()> {
    Err(Error::new(
        ErrorKind::Other,
        "unable to auto-install WebView on this system.",
    ))
}
