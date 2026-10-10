/**
 * Shared reader for the `reduce-animation` preference.
 *
 * The settings page, its navbar, and the loader read this as a one-shot
 * boolean at setup (the layout syncs it before they render). Missing or
 * malformed values mean animations stay enabled.
 */
export function readReduceAnimation(): boolean {
    try {
        return JSON.parse(localStorage.getItem("reduce-animation") ?? "false") === true;
    } catch {
        return false;
    }
}
