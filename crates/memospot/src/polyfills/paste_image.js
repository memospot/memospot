/*
 * Image paste polyfill (Linux only).
 *
 * WebKitGTK reports an empty DataTransfer for image-only clipboards, so the
 * Memos editor never sees the pasted file. When a paste carries neither
 * files nor text, ask Memospot to read the clipboard natively; the reply
 * carries the image bytes, which are replayed as a synthetic paste at the
 * captured editor. Rust placeholders are replaced once at injection.
 */
(() => {
    const EPOCH = Number("__MEMOSPOT_PASTE_EPOCH__");
    const NONCE = "__MEMOSPOT_PASTE_NONCE__";

    const scope = globalThis;
    const current = scope.__memospotPaste;
    if (current?.epoch > EPOCH) {
        return;
    }
    if (current && typeof current.detach === "function") {
        try {
            current.detach();
        } catch (_) {
            return;
        }
    }

    function editableOf(node) {
        const element = node?.nodeType === 1 ? node : node?.parentElement;
        return element?.isContentEditable ? element : null;
    }

    function textOf(transfer, type) {
        try {
            return transfer.getData(type);
        } catch (_) {
            return null;
        }
    }

    async function onPaste(event) {
        try {
            const handle = scope.__memospotPaste;
            if (!handle || handle.epoch !== EPOCH) {
                return;
            }
            if (!event.isTrusted) {
                return;
            }
            const data = event.clipboardData;
            if (data == null) {
                return;
            }
            if (data.files?.length > 0) {
                return;
            }
            const seen = new Set();
            const candidates = [];
            if (data.types) {
                for (const type of data.types) {
                    if (
                        typeof type === "string" &&
                        type.indexOf("text/") === 0 &&
                        !seen.has(type)
                    ) {
                        seen.add(type);
                        candidates.push(type);
                    }
                }
            }
            for (const type of ["text/plain", "text/html", "text/uri-list"]) {
                if (!seen.has(type)) {
                    candidates.push(type);
                }
            }
            for (const type of candidates) {
                const value = textOf(data, type);
                if (value == null || value !== "") {
                    return;
                }
            }
            const editor = editableOf(event.target);
            if (editor == null) {
                return;
            }
            const invoke = window.__TAURI__?.core?.invoke;
            if (typeof invoke !== "function") {
                return;
            }
            const payload = await invoke(
                "request_paste_image",
                { pasteAtMs: Date.now(), nonce: NONCE },
                { headers: { Origin: window.location.origin } }
            ).catch(() => null);
            if (payload == null || typeof payload !== "object") {
                return;
            }
            let bytes;
            try {
                const binary = atob(payload.base64);
                bytes = Uint8Array.from(binary, (c) => c.charCodeAt(0));
            } catch (_) {
                return;
            }
            if (bytes.length === 0) {
                return;
            }
            let file;
            let transfer;
            try {
                file = new File([bytes], payload.name, { type: payload.mime });
                transfer = new DataTransfer();
                transfer.items.add(file);
            } catch (_) {
                return;
            }
            if (!editor.isConnected || !editor.isContentEditable) {
                return;
            }
            try {
                editor.dispatchEvent(
                    new ClipboardEvent("paste", {
                        bubbles: true,
                        composed: true,
                        clipboardData: transfer
                    })
                );
            } catch (_) {
                return;
            }
        } catch (_) {
            return;
        }
    }

    document.addEventListener("paste", onPaste, true);
    scope.__memospotPaste = {
        epoch: EPOCH,
        detach: () => document.removeEventListener("paste", onPaste, true)
    };
})();
