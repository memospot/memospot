/*
 * Register fallback shortcuts via JS injected on each page load.
 *
 * This ensures shortcuts like Ctrl+H, Ctrl+, and Numpad zoom still work
 * even when the native menu bar (and its hardware accelerators) is hidden.
 */
document.addEventListener("keydown", (e) => {
    if (!(e.ctrlKey || e.metaKey)) return;
    var cmd = null;
    if (e.code === "NumpadAdd") cmd = "zoom_in";
    else if (e.code === "NumpadSubtract") cmd = "zoom_out";
    else if (e.code === "Numpad0") cmd = "reset_zoom";
    else if (e.code === "KeyH") cmd = "toggle_menu_bar";
    else if (e.code === "Comma" || e.key === ",") cmd = "open_settings";

    if (cmd) {
        e.preventDefault();
        window.__TAURI_INTERNALS__.invoke(cmd);
    }
});
