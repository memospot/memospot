/*
 * Image paste polyfill (Linux only).
 *
 * WebKitGTK hands `paste` events an empty `DataTransfer` when the clipboard holds an image,
 * so Memos never sees the pasted file. When a paste carries neither files nor text, ask
 * Memospot to read the clipboard image natively; it answers by calling
 * `__memospotPasteImage`, which replays the paste with the image attached.
 */
(() => {
    document.addEventListener(
        "paste",
        (event) => {
            const data = event.clipboardData;
            if (
                !event.isTrusted ||
                !data ||
                data.files.length > 0 ||
                data.types.includes("text/plain")
            ) {
                return;
            }
            const label = window.__TAURI_INTERNALS__.metadata.currentWebview.label;
            void window.__TAURI__.event.emit("memospot-paste-image", label).catch((error) => {
                console.error("Memospot image paste failed:", error);
            });
        },
        true
    );

    window.__memospotPasteImage = (base64) => {
        const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
        const transfer = new DataTransfer();
        transfer.items.add(new File([bytes], "image.png", { type: "image/png" }));
        (document.activeElement ?? document.body).dispatchEvent(
            new ClipboardEvent("paste", {
                clipboardData: transfer,
                bubbles: true,
                cancelable: true
            })
        );
    };
})();
