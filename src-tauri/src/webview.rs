use memospot::panic_dialog;
use std::process::Command;

#[cfg(windows)]
use {
    memospot::info_dialog,
    std::io::Cursor,
    std::path::PathBuf,
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

pub fn launch_webview_install_website() {
    const TAURI_WEBVIEW_REF: &str = "https://tauri.app/v1/references/webview-versions/";
    const WINDOWS_WEBVIEW_URL: &str =
        "https://developer.microsoft.com/microsoft-edge/webview2/#download-section";

    let err = match std::env::consts::OS {
        "linux" => Command::new("xdg-open").arg(TAURI_WEBVIEW_REF).spawn(),
        "windows" => Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", WINDOWS_WEBVIEW_URL])
            .spawn(),
        "macos" => Command::new("open").arg(TAURI_WEBVIEW_REF).spawn(),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unsupported operating system",
        )),
    };
    if err.is_err() {
        panic_dialog!(
            "Unable to launch WebView reference website.\nPlease install WebView manually."
        );
    }
}

#[cfg(windows)]
pub async fn install_cleanup(
    installer_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    if installer_path.exists() {
        std::fs::remove_file(installer_path)?;
    }
    Ok(())
}

#[cfg(windows)]
pub async fn install() -> Result<(), Box<dyn std::error::Error>> {
    const WEBVIEW2_BOOTSTRAPPER_URL: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";
    let mut filename = "MicrosoftEdgeWebview2Setup.exe".to_owned();

    let client = reqwest::Client::builder()
        .user_agent("Tauri")
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .build()?;

    let response = client.get(WEBVIEW2_BOOTSTRAPPER_URL).send().await?;
    if !response.status().is_success() {
        launch_webview_install_website();
        panic_dialog!("Failed to download WebView2 installer");
    }

    // get filename from response headers
    let content_disposition = response.headers().get("content-disposition");
    if content_disposition.is_some() {
        if let Some(value) = content_disposition {
            if let Ok(value) = value.to_str() {
                if let Some(last) = value.split("filename=").last() {
                    let name = last.trim().replace('\"', "");
                    if !&name.is_empty() {
                        filename = name;
                    }
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

    let current_exe = std::env::current_exe().unwrap();
    let cwd = current_exe.parent().unwrap();
    let webview_installer = cwd.join(filename);

    let mut file = std::fs::File::create(&webview_installer)?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    std::io::Write::flush(&mut file)?;
    drop(file);
    drop(content);

    let child = Command::new(&webview_installer).args(["/install"]).spawn();
    if let Err(e) = child {
        launch_webview_install_website();
        install_cleanup(webview_installer).await?;
        panic_dialog!(
            "Failed to launch WebView2 installer:\n{}\n\nPlease install it manually.",
            e
        );
    }

    if let Ok(mut child) = child {
        let result = child.wait();
        if let Err(e) = result {
            launch_webview_install_website();
            install_cleanup(webview_installer).await?;
            panic_dialog!(
                "Failed to install WebView2:\n{}\n\nPlease install it manually.",
                e
            );
        }
        if let Ok(status) = result {
            if let Some(code) = status.code() {
                if code != 0 {
                    launch_webview_install_website();
                    install_cleanup(webview_installer).await?;
                    panic_dialog!(
                        "WebView2 installer exited with code {}.\nPlease install it manually.",
                        code
                    );
                }
            }
        }
    }

    install_cleanup(webview_installer).await?;
    info_dialog!("WebView2 installer exited successfully.");

    Ok(())
}

#[cfg(not(windows))]
pub async fn install() -> Result<(), Box<dyn std::error::Error>> {
    launch_webview_install_website();
    panic_dialog!(
        "Unable to auto-install WebView on this system.\nPlease install it manually."
    );
}
