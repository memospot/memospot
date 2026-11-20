//! Gated Linux clipboard-image read for image-only pastes.
//!
//! WebKitGTK reports an empty `DataTransfer` for image-only clipboards, so
//! the Memos editor never sees the pasted file. The injected polyfill calls
//! [`request_paste_image`] when it observes such an empty transfer. This
//! module checks the request against the configured server origin, reads the
//! clipboard through GTK on the main thread, and returns the detected image
//! bytes. Clipboard access, MIME sniffing, encoding, and failure policy stay
//! inside this module; callers learn only the command signature, the origin
//! rule, the freshness window, and the payload shape.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use log::{info, warn};
use serde::Serialize;
use tauri::ipc::Request;
use tauri::{Manager, Runtime, State, Webview, command};
use url::{Host, Origin, Url};

use crate::runtime_config::AppState;

/// Lowest accepted age (`now_ms - paste_at_ms`), in milliseconds. A request
/// up to 200 ms early (page/backend clock skew) stays accepted.
pub(crate) const FRESHNESS_MIN_MS: i64 = -200;
/// Highest accepted age (`now_ms - paste_at_ms`), in milliseconds. A request
/// up to 2000 ms old stays accepted; older requests are stale.
pub(crate) const FRESHNESS_MAX_MS: i64 = 2000;
/// Single budget from gate acceptance to return, covering clipboard access,
/// encode, and IPC. A late reply is discarded.
pub(crate) const READ_BUDGET_MS: u64 = 2000;
/// Clipboard images holding more bytes are dropped before any copy lands.
pub(crate) const MAX_IMAGE_BYTES: u64 = 10_485_760;
/// Fallback pixbufs decoding to more pixel bytes are dropped before encoding.
pub(crate) const MAX_DECODED_PIXEL_BYTES: u64 = 67_108_864;
/// Clipboard targets tried in order; the first yielding bytes wins.
const PREFERRED_TARGETS: [&str; 3] = ["image/png", "image/jpeg", "image/webp"];
/// Length of a hex page-load nonce (`Uuid::new_v4().as_simple()`).
const NONCE_LEN: usize = 32;
/// Generic rejection surfaced to the page. Carries no bytes, URLs, origins,
/// or nonces.
pub(crate) const REJECT_MESSAGE: &str = "clipboard image request was rejected";

/// Image type detected by sniffing the clipboard bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mime {
    Png,
    Jpeg,
    Webp,
}

impl Mime {
    fn mime_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
    }

    fn file_name(self) -> &'static str {
        match self {
            Self::Png => "image.png",
            Self::Jpeg => "image.jpg",
            Self::Webp => "image.webp",
        }
    }
}

/// Detect the image type from leading magic bytes. The sniffed type wins
/// over the advertised clipboard target. Returns [`None`] for short or
/// unrecognized input.
pub(crate) fn sniff_mime(bytes: &[u8]) -> Option<Mime> {
    if bytes.len() >= 8 && bytes[..8] == [137, 80, 78, 71, 13, 10, 26, 10] {
        Some(Mime::Png)
    } else if bytes.len() >= 3 && bytes[..3] == [0xFF, 0xD8, 0xFF] {
        Some(Mime::Jpeg)
    } else if bytes.len() >= 12 && bytes[..4] == *b"RIFF" && bytes[8..12] == *b"WEBP" {
        Some(Mime::Webp)
    } else {
        None
    }
}

/// Gate rejection with a log-only reason code. The page sees only
/// [`REJECT_MESSAGE`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reject {
    WebviewUrl,
    WebviewOriginMismatch,
    RequestOriginMismatch,
    UnparseableOrigin,
    MissingOrigin,
    OpaqueOrigin,
    IneligibleOrigin,
    Stale,
    Future,
    NonceMismatch,
    MalformedNonce,
    MissingToken,
    Overlap,
}

impl Reject {
    fn reason(self) -> &'static str {
        match self {
            Self::WebviewUrl => "webview-url",
            Self::WebviewOriginMismatch => "webview-origin-mismatch",
            Self::RequestOriginMismatch => "request-origin-mismatch",
            Self::UnparseableOrigin => "unparseable-origin",
            Self::MissingOrigin => "missing-origin",
            Self::OpaqueOrigin => "opaque-origin",
            Self::IneligibleOrigin => "ineligible-origin",
            Self::Stale => "stale",
            Self::Future => "future",
            Self::NonceMismatch => "nonce-mismatch",
            Self::MalformedNonce => "malformed-nonce",
            Self::MissingToken => "missing-token",
            Self::Overlap => "overlap",
        }
    }
}

/// Distinct wrapper types keep runtime, webview, and page-supplied values apart.
#[derive(Debug)]
pub(crate) struct RuntimeUrl(pub Url);
/// Distinct wrapper types keep runtime, webview, and page-supplied values apart.
#[derive(Debug)]
pub(crate) struct WebviewUrl(pub Url);
/// Distinct wrapper types keep runtime, webview, and page-supplied values apart.
#[derive(Debug)]
pub(crate) struct ExpectedNonce(pub String);
/// Distinct wrapper types keep runtime, webview, and page-supplied values apart.
#[derive(Debug)]
pub(crate) struct PageNonce(pub String);

/// Gate inputs bundled as one request. Freshness and the nonce are staleness
/// guards only; the origin match is the trust anchor.
#[derive(Debug)]
pub(crate) struct GateInput {
    pub runtime_url: RuntimeUrl,
    pub webview_url: WebviewUrl,
    pub request_origin: Origin,
    pub paste_at_ms: i64,
    pub now_ms: i64,
    pub expected_nonce: ExpectedNonce,
    pub provided_nonce: PageNonce,
}

/// Whether a byte length fits the image cap before it is materialized.
pub(crate) fn within_image_cap(len: u64) -> bool {
    len <= MAX_IMAGE_BYTES
}

/// Whether decoded pixbuf dimensions fit the pixel cap before encoding.
/// Saturates so hostile dimensions fail closed instead of overflowing.
pub(crate) fn within_pixel_cap(width: i32, height: i32, channels: i32) -> bool {
    let pixels = (width as u64)
        .saturating_mul(height as u64)
        .saturating_mul(channels as u64);
    pixels <= MAX_DECODED_PIXEL_BYTES
}

/// Classify materialized target bytes: empty counts as missing so the read
/// falls through to the next target; over-cap bytes drop the whole read.
pub(crate) fn classify_contents(bytes: &[u8]) -> ContentsRead {
    if bytes.is_empty() {
        ContentsRead::Missing
    } else if !within_image_cap(bytes.len() as u64) {
        ContentsRead::Oversize
    } else {
        ContentsRead::Bytes(bytes.to_vec())
    }
}

/// Whether a page-load nonce has the expected hex shape.
pub(crate) fn is_hex_nonce(nonce: &str) -> bool {
    nonce.len() == NONCE_LEN && nonce.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Whether an origin may invoke the read: any `https:` origin, or `http:`
/// on exactly `localhost`, `127.0.0.1`, or `::1`.
pub(crate) fn eligible_origin(origin: &Origin) -> bool {
    let Origin::Tuple(scheme, host, _) = origin else {
        return false;
    };
    if scheme == "https" {
        return true;
    }
    if scheme != "http" {
        return false;
    }
    match host {
        Host::Domain(name) => name == "localhost",
        Host::Ipv4(address) => *address == std::net::Ipv4Addr::LOCALHOST,
        Host::Ipv6(address) => *address == std::net::Ipv6Addr::LOCALHOST,
    }
}

/// Parse the request `Origin` header and require it to match the webview.
/// A missing, opaque, or unparseable header fails closed, as does any
/// mismatch with the webview origin.
pub(crate) fn parse_request_origin(
    header: Option<&str>,
    webview_url: &Url,
) -> Result<Origin, Reject> {
    let raw = header.ok_or(Reject::MissingOrigin)?;
    if raw == "null" {
        return Err(Reject::OpaqueOrigin);
    }
    let parsed = Url::parse(raw).map_err(|_| Reject::UnparseableOrigin)?;
    let origin = parsed.origin();
    if matches!(origin, Origin::Opaque(_)) {
        return Err(Reject::OpaqueOrigin);
    }
    if origin != webview_url.origin() {
        return Err(Reject::RequestOriginMismatch);
    }
    Ok(origin)
}

/// Check one clipboard-read request. Every gate runs before any clipboard
/// access; the first failure rejects the whole request.
pub(crate) fn check_request_allowed(input: &GateInput) -> Result<(), Reject> {
    let runtime_origin = input.runtime_url.0.origin();
    let webview_origin = input.webview_url.0.origin();
    if matches!(runtime_origin, Origin::Opaque(_))
        || matches!(webview_origin, Origin::Opaque(_))
    {
        return Err(Reject::UnparseableOrigin);
    }
    if webview_origin != runtime_origin {
        return Err(Reject::WebviewOriginMismatch);
    }
    if input.request_origin != runtime_origin {
        return Err(Reject::RequestOriginMismatch);
    }
    if !eligible_origin(&runtime_origin) {
        return Err(Reject::IneligibleOrigin);
    }
    let age = input.now_ms.saturating_sub(input.paste_at_ms);
    if age > FRESHNESS_MAX_MS {
        return Err(Reject::Stale);
    }
    if age < FRESHNESS_MIN_MS {
        return Err(Reject::Future);
    }
    if !is_hex_nonce(&input.provided_nonce.0) {
        return Err(Reject::MalformedNonce);
    }
    if input.provided_nonce.0 != input.expected_nonce.0 {
        return Err(Reject::NonceMismatch);
    }
    Ok(())
}

/// Validate the request parts the command gathered. Returns the bundled
/// gate input on success.
pub(crate) fn gate_request(
    runtime_url: &str,
    webview_url: &Url,
    origin_header: Option<&str>,
    paste_at_ms: i64,
    now_ms: i64,
    expected_nonce: Option<String>,
    provided_nonce: &str,
) -> Result<GateInput, Reject> {
    let runtime = Url::parse(runtime_url).map_err(|_| Reject::UnparseableOrigin)?;
    if matches!(runtime.origin(), Origin::Opaque(_)) {
        return Err(Reject::UnparseableOrigin);
    }
    let request_origin = parse_request_origin(origin_header, webview_url)?;
    let expected = ExpectedNonce(expected_nonce.ok_or(Reject::MissingToken)?);
    let input = GateInput {
        runtime_url: RuntimeUrl(runtime),
        webview_url: WebviewUrl(webview_url.clone()),
        request_origin,
        paste_at_ms,
        now_ms,
        expected_nonce: expected,
        provided_nonce: PageNonce(provided_nonce.to_string()),
    };
    check_request_allowed(&input)?;
    Ok(input)
}

/// One clipboard target read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ContentsRead {
    /// No bytes for this target; try the next one.
    Missing,
    /// Over the image cap; the whole read drops.
    Oversize,
    Bytes(Vec<u8>),
}

/// Fallback pixbuf read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FallbackRead {
    /// No image available; the read yields nothing.
    Missing,
    /// Over a cap; the whole read drops.
    Oversize,
    /// PNG-encoded fallback bytes.
    Bytes(Vec<u8>),
}

/// System seam for clipboard reads. The fake adapter records access so tests
/// prove no read happens before every gate passes.
pub(crate) trait ClipboardRead {
    fn read_contents(&self, target: &'static str) -> impl Future<Output = ContentsRead> + Send;
    fn read_fallback_png(&self) -> impl Future<Output = FallbackRead> + Send;
}

/// Outcome of one gated clipboard read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReadResult {
    Image {
        bytes: Vec<u8>,
        mime: Mime,
    },
    /// Empty clipboard or no image available; a quiet no-op.
    NoImage,
    /// A target or the fallback exceeded a size cap.
    Oversize,
    /// Returned bytes matched no known image type.
    Unsniffable,
}

/// Run the clipboard read for an already-validated request. The gate check
/// stays with the caller; this seam exists so tests prove a failed
/// validation never reaches the backend.
pub(crate) async fn handle_validated<B: ClipboardRead>(
    backend: &B,
    validation: Result<&GateInput, Reject>,
) -> Result<ReadResult, Reject> {
    validation?;
    Ok(read_clipboard(backend).await)
}

/// Read the clipboard through a backend: preferred MIME targets in order,
/// then the pixbuf fallback. Empty targets fall through; oversize or
/// unsniffable bytes drop the whole read.
pub(crate) async fn read_clipboard<B: ClipboardRead>(backend: &B) -> ReadResult {
    for target in PREFERRED_TARGETS {
        match backend.read_contents(target).await {
            ContentsRead::Missing => continue,
            ContentsRead::Oversize => return ReadResult::Oversize,
            ContentsRead::Bytes(bytes) => {
                return match sniff_mime(&bytes) {
                    Some(mime) => ReadResult::Image { bytes, mime },
                    None => ReadResult::Unsniffable,
                };
            }
        }
    }
    match backend.read_fallback_png().await {
        FallbackRead::Missing => ReadResult::NoImage,
        FallbackRead::Oversize => ReadResult::Oversize,
        FallbackRead::Bytes(png) => match sniff_mime(&png) {
            Some(mime) => ReadResult::Image { bytes: png, mime },
            None => ReadResult::Unsniffable,
        },
    }
}

/// Image payload returned to the polyfill for replay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PasteImagePayload {
    pub base64: String,
    pub mime: String,
    pub name: String,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|span| span.as_millis().try_into().unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn fail(reason: Reject) -> String {
    warn!("paste image request rejected: {}", reason.reason());
    REJECT_MESSAGE.to_string()
}

/// Request the clipboard image for an image-only paste.
///
/// The polyfill calls this only after its empty-transfer gate passes. Every
/// gate runs before any clipboard access; failures answer with a generic
/// rejection and one reason-coded `warn!`, never with bytes, URLs, origins,
/// or nonces.
#[command]
pub(crate) async fn request_paste_image<R: Runtime>(
    webview: Webview<R>,
    request: Request<'_>,
    state: State<'_, AppState>,
    paste_at_ms: i64,
    nonce: String,
) -> Result<Option<PasteImagePayload>, String> {
    let label = webview.label().to_string();
    let webview_url = webview.url().map_err(|_| fail(Reject::WebviewUrl))?;
    let header = match request.headers().get("origin") {
        None => None,
        Some(value) => Some(
            value
                .to_str()
                .map_err(|_| fail(Reject::UnparseableOrigin))?,
        ),
    };
    let input = gate_request(
        &state.runtime.active_server.url,
        &webview_url,
        header,
        paste_at_ms,
        now_ms(),
        state.paste.token_for(&label).map(|(nonce, _)| nonce),
        &nonce,
    )
    .map_err(fail)?;
    let _guard = state
        .paste
        .try_acquire(&label)
        .ok_or_else(|| fail(Reject::Overlap))?;

    let deadline = tokio::time::Instant::now() + Duration::from_millis(READ_BUDGET_MS);
    let backend = super::paste_image_gtk::GtkRead {
        app: webview.app_handle().clone(),
        deadline,
    };
    let work = async {
        match handle_validated(&backend, Ok(&input)).await? {
            ReadResult::Image { bytes, mime } => {
                let encoded = match super::paste_image_gtk::encode_image(bytes).await {
                    Ok(encoded) => encoded,
                    Err(error) => {
                        warn!("paste image read failed to encode the image: {error}");
                        return Ok(None);
                    }
                };
                info!("paste image served");
                Ok(Some(PasteImagePayload {
                    base64: encoded,
                    mime: mime.mime_type().to_string(),
                    name: mime.file_name().to_string(),
                }))
            }
            ReadResult::NoImage => {
                warn!("paste image read found no image");
                Ok(None)
            }
            ReadResult::Oversize => {
                warn!("paste image read dropped an oversize image");
                Ok(None)
            }
            ReadResult::Unsniffable => {
                warn!("paste image read dropped undetectable bytes");
                Ok(None)
            }
        }
    };
    match tokio::time::timeout_at(deadline, work).await {
        Err(_) => {
            warn!("paste image read timed out");
            Ok(None)
        }
        Ok(Err(reason)) => Err(fail(reason)),
        Ok(Ok(payload)) => Ok(payload),
    }
}

/// Render the polyfill with its page-load nonce and epoch. The nonce is
/// hex-only, so interpolation cannot break out of the string literal.
pub(crate) fn polyfill_js(nonce: &str, epoch: u64) -> String {
    // The only caller mints `Uuid::new_v4().as_simple()`; a non-hex nonce
    // here is a programming error, so fail loudly in every build.
    assert!(is_hex_nonce(nonce));
    include_str!("polyfills/paste_image.js")
        .replacen("__MEMOSPOT_PASTE_NONCE__", nonce, 1)
        .replacen("__MEMOSPOT_PASTE_EPOCH__", &epoch.to_string(), 1)
}

/// Mint a page-load token and inject the Linux paste polyfill. The token is
/// stored before injection so a racing paste still validates.
pub(crate) fn inject_polyfill<R: Runtime>(webview: &Webview<R>) {
    let label = webview.label().to_string();
    let paste = webview.state::<AppState>().paste.clone();
    let epoch = paste.next_epoch();
    let nonce = uuid::Uuid::new_v4().as_simple().to_string();
    debug_assert!(is_hex_nonce(&nonce));
    paste.store_token(&label, nonce.clone(), epoch);
    if webview.eval(polyfill_js(&nonce, epoch)).is_err() {
        warn!("paste image polyfill injection failed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use serde_json::json;
    use tauri::ipc::{CallbackFn, InvokeBody};
    use tauri::test::{
        INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets,
    };
    use tauri::webview::InvokeRequest;
    use tauri::{App, WebviewWindow, WebviewWindowBuilder};
    use tempfile::TempDir;

    /// Fixed test fixture, not a secret: the page-load nonce is
    /// defense-in-depth only (origin is the trust anchor) and is minted per
    /// load in production via `Uuid::new_v4`; tests pin one value for
    /// deterministic assertions.
    const NONCE: &str = "0123456789abcdef0123456789abcdef";

    fn runtime_url() -> Url {
        "http://localhost:5230/".parse().expect("runtime URL")
    }

    fn passing_input() -> GateInput {
        let url = runtime_url();
        GateInput {
            runtime_url: RuntimeUrl(url.clone()),
            webview_url: WebviewUrl(url.clone()),
            request_origin: url.origin(),
            paste_at_ms: 1_000,
            now_ms: 1_000,
            expected_nonce: ExpectedNonce(NONCE.to_string()),
            provided_nonce: PageNonce(NONCE.to_string()),
        }
    }

    fn gated(
        runtime: &str,
        webview: &str,
        header: Option<&str>,
        paste_at_ms: i64,
        now_ms: i64,
        expected: Option<String>,
        provided: &str,
    ) -> Result<GateInput, Reject> {
        let webview_url: Url = webview.parse().expect("webview URL");
        gate_request(
            runtime,
            &webview_url,
            header,
            paste_at_ms,
            now_ms,
            expected,
            provided,
        )
    }

    #[test]
    fn fresh_matching_request_is_allowed() {
        // GIVEN a loopback runtime with matching origins and nonce
        let input = passing_input();

        // WHEN the gate runs THEN it allows the read
        assert!(check_request_allowed(&input).is_ok());
    }

    #[test]
    fn freshness_window_bounds_are_inclusive() {
        // GIVEN the freshness window [-200, 2000] on age (now - paste_at)
        // WHEN requests sit exactly on the bounds THEN they pass
        let mut input = passing_input();
        input.paste_at_ms = input.now_ms - FRESHNESS_MAX_MS;
        assert!(check_request_allowed(&input).is_ok());
        input.paste_at_ms = input.now_ms - FRESHNESS_MIN_MS;
        assert!(check_request_allowed(&input).is_ok());

        // AND requests one millisecond outside either bound fail
        input.paste_at_ms = input.now_ms - FRESHNESS_MAX_MS - 1;
        assert_eq!(check_request_allowed(&input), Err(Reject::Stale));
        input.paste_at_ms = input.now_ms - FRESHNESS_MIN_MS + 1;
        assert_eq!(check_request_allowed(&input), Err(Reject::Future));
    }

    #[test]
    fn mismatched_webview_origin_is_rejected() {
        // GIVEN a webview on another host
        // WHEN the gate runs THEN the webview mismatch rejects first
        let result = gated(
            "http://localhost:5230/",
            "http://127.0.0.1:5230/",
            Some("http://127.0.0.1:5230/"),
            1_000,
            1_000,
            Some(NONCE.to_string()),
            NONCE,
        );
        assert_eq!(result.unwrap_err(), Reject::WebviewOriginMismatch);
    }

    #[test]
    fn mismatched_request_origin_is_rejected() {
        // GIVEN a request origin differing from the webview origin
        // WHEN the header is parsed THEN the mismatch rejects
        let webview: Url = "http://localhost:5230/".parse().expect("webview URL");
        let result = parse_request_origin(Some("http://127.0.0.1:5230/"), &webview);
        assert_eq!(result.unwrap_err(), Reject::RequestOriginMismatch);
    }

    #[test]
    fn missing_opaque_and_unparseable_origins_fail_closed() {
        // GIVEN a loopback webview
        let webview: Url = "http://localhost:5230/".parse().expect("webview URL");

        // WHEN the header is missing, opaque, or unparseable
        // THEN each fails closed with its own reason
        assert_eq!(
            parse_request_origin(None, &webview).unwrap_err(),
            Reject::MissingOrigin
        );
        assert_eq!(
            parse_request_origin(Some("null"), &webview).unwrap_err(),
            Reject::OpaqueOrigin
        );
        assert_eq!(
            parse_request_origin(Some("not a url"), &webview).unwrap_err(),
            Reject::UnparseableOrigin
        );
        assert_eq!(
            parse_request_origin(Some("tauri://localhost"), &webview).unwrap_err(),
            Reject::OpaqueOrigin
        );
    }

    #[test]
    fn loopback_alias_and_scheme_mismatches_are_rejected() {
        // GIVEN a localhost runtime
        // WHEN the webview uses the 127.0.0.1 alias or plain-http scheme
        // THEN the origin match fails because aliases stay distinct
        let header = Some("http://127.0.0.1:5230/");
        assert_eq!(
            gated(
                "http://localhost:5230/",
                "http://127.0.0.1:5230/",
                header,
                1_000,
                1_000,
                Some(NONCE.to_string()),
                NONCE,
            )
            .unwrap_err(),
            Reject::WebviewOriginMismatch
        );
        let header = Some("http://localhost:5230/");
        assert_eq!(
            gated(
                "https://localhost:5230/",
                "http://localhost:5230/",
                header,
                1_000,
                1_000,
                Some(NONCE.to_string()),
                NONCE,
            )
            .unwrap_err(),
            Reject::WebviewOriginMismatch
        );
    }

    #[test]
    fn plain_http_remote_is_ineligible_while_https_and_loopback_pass() {
        // GIVEN the eligibility rule (https anywhere, http only on loopback)
        let https: Url = "https://memos.example:8443/".parse().expect("https URL");
        let loopback: Url = "http://localhost:5230/".parse().expect("loopback URL");
        let remote: Url = "http://192.168.1.2:5230/".parse().expect("remote URL");

        // WHEN origins are classified THEN https and loopback pass, remote fails
        assert!(eligible_origin(&https.origin()));
        assert!(eligible_origin(&loopback.origin()));
        assert!(!eligible_origin(&remote.origin()));

        // AND a plain-HTTP remote runtime rejects at the gate
        let result = gated(
            "http://192.168.1.2:5230/",
            "http://192.168.1.2:5230/",
            Some("http://192.168.1.2:5230/"),
            1_000,
            1_000,
            Some(NONCE.to_string()),
            NONCE,
        );
        assert_eq!(result.unwrap_err(), Reject::IneligibleOrigin);
    }

    #[test]
    fn malformed_mismatched_and_missing_nonces_are_rejected() {
        // GIVEN a gated request helper with a stored token
        let stored = Some(NONCE.to_string());

        // WHEN the provided nonce is malformed, mismatched, or missing
        // THEN each fails with its own reason and no clipboard access follows
        let malformed = gated(
            "http://localhost:5230/",
            "http://localhost:5230/",
            Some("http://localhost:5230/"),
            1_000,
            1_000,
            stored.clone(),
            "not-hex!",
        );
        assert_eq!(malformed.unwrap_err(), Reject::MalformedNonce);

        let mismatched = gated(
            "http://localhost:5230/",
            "http://localhost:5230/",
            Some("http://localhost:5230/"),
            1_000,
            1_000,
            stored,
            &"f".repeat(32),
        );
        assert_eq!(mismatched.unwrap_err(), Reject::NonceMismatch);

        let missing = gated(
            "http://localhost:5230/",
            "http://localhost:5230/",
            Some("http://localhost:5230/"),
            1_000,
            1_000,
            None,
            NONCE,
        );
        assert_eq!(missing.unwrap_err(), Reject::MissingToken);
    }

    #[test]
    fn sniff_mime_detects_png_jpeg_and_webp() {
        // GIVEN leading magic bytes for each supported type
        let png = [137, 80, 78, 71, 13, 10, 26, 10, 0];
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0, 0];
        let webp = *b"RIFF\x00\x00\x00\x00WEBP";

        // WHEN sniffed THEN each maps to its MIME
        assert_eq!(sniff_mime(&png), Some(Mime::Png));
        assert_eq!(sniff_mime(&jpeg), Some(Mime::Jpeg));
        assert_eq!(sniff_mime(&webp), Some(Mime::Webp));

        // AND short or unknown input detects nothing
        assert_eq!(sniff_mime(&[]), None);
        assert_eq!(sniff_mime(&[0xFF, 0xD8]), None);
        assert_eq!(sniff_mime(b"GIF89a"), None);
    }

    #[test]
    fn size_cap_predicates_reject_overflowing_dimensions() {
        // GIVEN the 10 MiB image cap and the 67 M decoded-pixel cap
        // WHEN lengths and dimensions are classified
        // THEN exact caps pass and anything above fails
        assert!(within_image_cap(MAX_IMAGE_BYTES));
        assert!(!within_image_cap(MAX_IMAGE_BYTES + 1));
        assert!(within_pixel_cap(4096, 4096, 4));
        assert!(!within_pixel_cap(8192, 8192, 4));

        // AND hostile dimensions fail closed instead of overflowing
        assert!(!within_pixel_cap(i32::MAX, i32::MAX, 4));
    }

    #[test]
    fn polyfill_js_carries_exactly_one_hex_nonce_and_epoch() {
        // GIVEN a hex nonce and an epoch
        let epoch = 7;

        // WHEN the polyfill is rendered THEN both placeholders are gone
        let rendered = polyfill_js(NONCE, epoch);
        assert!(!rendered.contains("__MEMOSPOT_PASTE_NONCE__"));
        assert!(!rendered.contains("__MEMOSPOT_PASTE_EPOCH__"));

        // AND the nonce appears verbatim exactly once, so no breakout is possible
        assert_eq!(rendered.matches(NONCE).count(), 1);
        assert!(rendered.contains(&format!("{epoch}")));

        // AND non-hex input is never a valid nonce
        assert!(!is_hex_nonce("ab\"; alert(1)//"));
        assert!(!is_hex_nonce(&"a".repeat(31)));
    }

    /// Fake clipboard recording every access for ordering assertions.
    struct FakeBackend {
        accesses: Mutex<Vec<&'static str>>,
        contents: HashMap<&'static str, ContentsRead>,
        fallback: FallbackRead,
    }

    impl FakeBackend {
        fn with(contents: &[(&'static str, ContentsRead)], fallback: FallbackRead) -> Self {
            Self {
                accesses: Mutex::new(Vec::new()),
                contents: contents.iter().cloned().collect(),
                fallback,
            }
        }

        fn accesses(&self) -> Vec<&'static str> {
            self.accesses.lock().expect("access lock poisoned").clone()
        }
    }

    impl ClipboardRead for FakeBackend {
        async fn read_contents(&self, target: &'static str) -> ContentsRead {
            self.accesses
                .lock()
                .expect("access lock poisoned")
                .push(target);
            self.contents
                .get(target)
                .cloned()
                .unwrap_or(ContentsRead::Missing)
        }

        async fn read_fallback_png(&self) -> FallbackRead {
            self.accesses
                .lock()
                .expect("access lock poisoned")
                .push("fallback");
            self.fallback.clone()
        }
    }

    fn png_bytes() -> Vec<u8> {
        vec![137, 80, 78, 71, 13, 10, 26, 10, 1, 2, 3]
    }

    fn jpeg_bytes() -> Vec<u8> {
        vec![0xFF, 0xD8, 0xFF, 0xE0, 1, 2, 3]
    }

    #[tokio::test]
    async fn preferred_target_wins_and_empty_targets_fall_through() {
        // GIVEN PNG and JPEG targets both yielding bytes
        let backend = FakeBackend::with(
            &[
                ("image/png", ContentsRead::Bytes(png_bytes())),
                ("image/jpeg", ContentsRead::Bytes(jpeg_bytes())),
            ],
            FallbackRead::Missing,
        );

        // WHEN the clipboard is read THEN the PNG bytes win
        let result = read_clipboard(&backend).await;
        assert_eq!(
            result,
            ReadResult::Image {
                bytes: png_bytes(),
                mime: Mime::Png,
            }
        );

        // GIVEN an empty PNG target with JPEG bytes behind it
        let backend = FakeBackend::with(
            &[
                ("image/png", ContentsRead::Missing),
                ("image/jpeg", ContentsRead::Bytes(jpeg_bytes())),
            ],
            FallbackRead::Missing,
        );

        // WHEN the clipboard is read THEN the JPEG bytes attach
        let result = read_clipboard(&backend).await;
        assert_eq!(
            result,
            ReadResult::Image {
                bytes: jpeg_bytes(),
                mime: Mime::Jpeg,
            }
        );
    }

    #[tokio::test]
    async fn sniffed_type_wins_and_failures_drop_the_read() {
        // GIVEN a PNG target returning JPEG bytes
        let backend = FakeBackend::with(
            &[("image/png", ContentsRead::Bytes(jpeg_bytes()))],
            FallbackRead::Missing,
        );

        // WHEN the clipboard is read THEN the detected JPEG type attaches
        let result = read_clipboard(&backend).await;
        assert_eq!(
            result,
            ReadResult::Image {
                bytes: jpeg_bytes(),
                mime: Mime::Jpeg,
            }
        );

        // GIVEN a target returning undetectable bytes
        let backend = FakeBackend::with(
            &[("image/png", ContentsRead::Bytes(vec![1, 2, 3, 4]))],
            FallbackRead::Missing,
        );

        // WHEN the clipboard is read THEN the image drops
        assert_eq!(read_clipboard(&backend).await, ReadResult::Unsniffable);

        // GIVEN an oversize target
        let backend = FakeBackend::with(
            &[("image/png", ContentsRead::Oversize)],
            FallbackRead::Missing,
        );

        // WHEN the clipboard is read THEN the image drops without fallback
        assert_eq!(read_clipboard(&backend).await, ReadResult::Oversize);
        assert_eq!(backend.accesses(), vec!["image/png"]);
    }

    #[test]
    fn target_bytes_classify_to_missing_oversize_or_image() {
        // GIVEN the target classifier
        // WHEN bytes are empty, over-cap, or well-formed
        // THEN empty falls through, over-cap drops, and the rest attaches
        assert_eq!(classify_contents(&[]), ContentsRead::Missing);
        assert_eq!(
            classify_contents(&vec![0; (MAX_IMAGE_BYTES + 1) as usize]),
            ContentsRead::Oversize
        );
        assert_eq!(
            classify_contents(&png_bytes()),
            ContentsRead::Bytes(png_bytes())
        );
    }

    #[tokio::test]
    async fn failed_validation_never_touches_the_clipboard() {
        // GIVEN a failing validation with a recording backend
        let backend = FakeBackend::with(&[], FallbackRead::Missing);

        // WHEN the validated handler runs THEN it rejects without any access
        let result = handle_validated(&backend, Err(Reject::Stale)).await;
        assert_eq!(result, Err(Reject::Stale));
        assert!(backend.accesses().is_empty());
    }

    #[tokio::test]
    async fn passed_validation_reads_through_the_backend() {
        // GIVEN a passing validation with a recording backend
        let backend = FakeBackend::with(
            &[("image/png", ContentsRead::Bytes(png_bytes()))],
            FallbackRead::Missing,
        );
        let input = passing_input();

        // WHEN the validated handler runs THEN it reads exactly one target
        let result = handle_validated(&backend, Ok(&input)).await;
        assert_eq!(
            result,
            Ok(ReadResult::Image {
                bytes: png_bytes(),
                mime: Mime::Png,
            })
        );
        assert_eq!(backend.accesses(), vec!["image/png"]);
    }

    #[tokio::test]
    async fn failed_targets_reach_the_fallback() {
        // GIVEN no MIME target yielding bytes but a valid fallback
        let backend = FakeBackend::with(&[], FallbackRead::Bytes(png_bytes()));

        // WHEN the clipboard is read THEN exactly one fallback PNG attaches
        let result = read_clipboard(&backend).await;
        assert_eq!(
            result,
            ReadResult::Image {
                bytes: png_bytes(),
                mime: Mime::Png,
            }
        );
        assert_eq!(
            backend.accesses(),
            vec!["image/png", "image/jpeg", "image/webp", "fallback"]
        );

        // GIVEN nothing anywhere THEN the read yields nothing
        let backend = FakeBackend::with(&[], FallbackRead::Missing);
        assert_eq!(read_clipboard(&backend).await, ReadResult::NoImage);
    }

    fn mock_state(store: &crate::runtime_config::ConfigStore) -> AppState {
        use crate::runtime_config::{ActiveServer, RuntimeContext, RuntimePaths};
        use config::Config;
        use std::path::PathBuf;
        use std::sync::{Arc, RwLock};

        let maximized = store
            .snapshot()
            .current
            .memospot
            .window
            .maximized
            .unwrap_or_default();
        AppState {
            runtime: RuntimeContext {
                paths: RuntimePaths {
                    memos_bin: PathBuf::new(),
                    memos_data: PathBuf::new(),
                    memos_db_file: PathBuf::new(),
                    memospot_bin: PathBuf::new(),
                    memospot_config_file: PathBuf::new(),
                    memospot_cwd: PathBuf::new(),
                    memospot_data: PathBuf::new(),
                },
                active_server: ActiveServer {
                    url: "http://localhost:5230/".into(),
                    user_agent: "test".into(),
                    managed: true,
                },
                memos: Config::default().memos,
            },
            config: store.clone(),
            memos_version: Arc::new(RwLock::new(None)),
            zoom_level: Arc::new(RwLock::new(1.0)),
            window_states: crate::window_state::WindowStateQueue::new(store.clone(), maximized),
            paste: crate::runtime_config::PasteImageState::default(),
        }
    }

    #[tokio::test]
    async fn rejected_request_answers_generic_error_without_clipboard_access() {
        // GIVEN a mock app with the paste command and a stored page-load token
        let dir = TempDir::new().expect("tempdir");
        let config = config::Config::default();
        let store = crate::runtime_config::ConfigStore::new(
            config,
            config::Config::default(),
            dir.path().join("memospot.yaml"),
        );
        let state = mock_state(&store);
        state.paste.store_token("main", NONCE.to_string(), 1);
        let app = mock_builder()
            .invoke_handler(tauri::generate_handler![super::request_paste_image])
            .manage(state)
            .build(mock_context(noop_assets()))
            .expect("mock app should build");
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("mock webview should build");
        let _app: App<MockRuntime> = app;

        // WHEN a request arrives without an Origin header
        let local = if cfg!(any(windows, target_os = "android")) {
            "https://tauri.localhost"
        } else {
            "tauri://localhost"
        };
        let error = get_ipc_response(
            &webview,
            InvokeRequest {
                cmd: "request_paste_image".into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: local.parse().expect("local URL"),
                body: InvokeBody::Json(json!({ "pasteAtMs": 1_000, "nonce": NONCE })),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .expect_err("request without origin should be rejected");

        // THEN the page sees only the generic rejection
        assert_eq!(error, json!(REJECT_MESSAGE));
        let _window: WebviewWindow<MockRuntime> = webview;
    }
}
