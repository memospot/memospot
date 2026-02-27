/*
 * Register numpad zoom shortcuts via JS injected on each page load.
 *
 * Regular zoom shortcuts (Ctrl+=/Ctrl+-/Ctrl+0) are handled by menu accelerators.
 */
document.addEventListener("keydown", (e) => {
    if (!(e.ctrlKey || e.metaKey)) return;
    var cmd = null;
    if (e.code === "NumpadAdd") cmd = "zoom_in";
    else if (e.code === "NumpadSubtract") cmd = "zoom_out";
    else if (e.code === "Numpad0") cmd = "reset_zoom";
    if (cmd) {
        e.preventDefault();
        window.__TAURI_INTERNALS__.invoke(cmd);
    }
});
