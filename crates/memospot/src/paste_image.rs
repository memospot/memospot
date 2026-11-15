//! Image paste workaround for WebKitGTK.
//!
//! WebKitGTK hands `paste` events an empty `DataTransfer` when the clipboard holds an
//! image, and denies `navigator.clipboard.read()`. The injected [`POLYFILL`] emits
//! [`PASTE_IMAGE_EVENT`] with its webview label; [`replay`] reads the image through GTK
//! and hands it back as PNG so the polyfill can re-dispatch the paste with a `File`.
use log::warn;
use tauri::{AppHandle, Manager, Runtime};

pub(crate) const PASTE_IMAGE_EVENT: &str = "memospot-paste-image";
pub(crate) static POLYFILL: &str = include_str!("polyfills/paste_image.js");

/// Replay the clipboard image as a paste in the webview whose label is in `payload`.
pub(crate) fn replay<R: Runtime>(app: &AppHandle<R>, payload: &str) {
    let Ok(label) = serde_json::from_str::<String>(payload) else {
        warn!("invalid {PASTE_IMAGE_EVENT} payload: {payload}");
        return;
    };
    let app_ = app.clone();
    // GTK clipboard access must happen on the main thread.
    let _ = app.run_on_main_thread(move || {
        gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD).request_image(move |_, pixbuf| {
            let Some(pixbuf) = pixbuf else {
                return;
            };
            let png = match pixbuf.save_to_bufferv("png", &[]) {
                Ok(png) => png,
                Err(e) => {
                    warn!("failed to encode clipboard image: {e}");
                    return;
                }
            };
            let Some(webview) = app_.get_webview_window(&label) else {
                return;
            };
            let base64 = gtk::glib::base64_encode(&png);
            webview
                .eval(format!("window.__memospotPasteImage('{base64}')"))
                .ok();
        });
    });
}
