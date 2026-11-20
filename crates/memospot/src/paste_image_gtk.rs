//! Linux GTK clipboard backend for the gated paste-image read.
//!
//! Platform integration only: every request starts on the main thread and
//! the reply hops back through a oneshot under the shared read deadline. A
//! request starting after the deadline, or a reply arriving after it, is
//! dropped with the read.

use gtk::{gdk, glib};
use tauri::{AppHandle, Runtime};

use super::paste_image::{
    ClipboardRead, ContentsRead, FallbackRead, classify_contents, within_image_cap,
    within_pixel_cap,
};

/// GTK clipboard backend. Holds the app handle for main-thread dispatch
/// and the shared read deadline; a hop arriving after the deadline is
/// dropped with the reply.
pub(crate) struct GtkRead<R: Runtime> {
    pub(crate) app: AppHandle<R>,
    pub(crate) deadline: tokio::time::Instant,
}

impl<R: Runtime> GtkRead<R> {
    /// Start `begin` on the main thread and wait for its oneshot reply up to
    /// the deadline. Never blocks the main thread: the GTK callback answers
    /// from a later main-loop turn. A start past the deadline never runs.
    async fn roundtrip<T>(
        &self,
        begin: impl FnOnce(tokio::sync::oneshot::Sender<T>) + Send + 'static,
        expired: T,
    ) -> T
    where
        T: Send + 'static,
    {
        if tokio::time::Instant::now() >= self.deadline {
            return expired;
        }
        let (send, reply) = tokio::sync::oneshot::channel();
        let started = self.app.run_on_main_thread(move || begin(send)).is_ok();
        if !started {
            return expired;
        }
        tokio::time::timeout_at(self.deadline, reply)
            .await
            .ok()
            .and_then(|result| result.ok())
            .unwrap_or(expired)
    }
}

impl<R: Runtime> ClipboardRead for GtkRead<R> {
    async fn read_contents(&self, target: &'static str) -> ContentsRead {
        self.roundtrip(
            move |send| {
                let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                let atom = gdk::Atom::intern(target);
                clipboard.request_contents(&atom, move |_, selection| {
                    let length = selection.length();
                    // A negative length means retrieval failed, so the target
                    // counts as missing and the read falls through. The
                    // length pre-check rejects over-cap targets before the
                    // extra copy; the classifier re-checks after it.
                    let read = if length < 0 {
                        ContentsRead::Missing
                    } else if !within_image_cap(length as u64) {
                        ContentsRead::Oversize
                    } else {
                        classify_contents(&selection.data())
                    };
                    let _ = send.send(read);
                });
            },
            ContentsRead::Missing,
        )
        .await
    }

    async fn read_fallback_png(&self) -> FallbackRead {
        self.roundtrip(
            move |send| {
                let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                clipboard.request_image(move |_, pixbuf| {
                    let read = match pixbuf {
                        None => FallbackRead::Missing,
                        Some(pixbuf) => {
                            if !within_pixel_cap(
                                pixbuf.width(),
                                pixbuf.height(),
                                pixbuf.n_channels(),
                            ) {
                                FallbackRead::Oversize
                            } else {
                                match pixbuf.save_to_bufferv("png", &[]) {
                                    Ok(png) if within_image_cap(png.len() as u64) => {
                                        FallbackRead::Bytes(png)
                                    }
                                    Ok(_) => FallbackRead::Oversize,
                                    Err(_) => FallbackRead::Missing,
                                }
                            }
                        }
                    };
                    let _ = send.send(read);
                });
            },
            FallbackRead::Missing,
        )
        .await
    }
}

/// Base64-encode clipboard bytes off the main thread. The join error is
/// preserved for the caller to log; it carries no clipboard content.
pub(crate) async fn encode_image(bytes: Vec<u8>) -> Result<String, tokio::task::JoinError> {
    tokio::task::spawn_blocking(move || glib::base64_encode(&bytes).to_string()).await
}
