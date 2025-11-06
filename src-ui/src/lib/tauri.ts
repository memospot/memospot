/*
 * Tauri Handlers.
 *
 * The Rust back-end is in `crates/memospot/src/cmd`.
 */

import { isTauri, invoke as TauriInvoke } from "@tauri-apps/api/core";
import { open as TauriOpen } from "@tauri-apps/plugin-shell";

const browserError = new Error("Not running in Tauri!");
const TAURI = typeof window !== "undefined" && isTauri();
const invoke = TAURI ? TauriInvoke : async () => Promise.reject(browserError.message);

/**
 * Open a URL.
 * @param url URL to open
 */
export async function open(url: string): Promise<void> {
    if (TAURI) {
        await TauriOpen(url);
    } else {
        window.open(url);
    }
}

/**
 * Get Memos URL.
 */
export async function getMemosURL(): Promise<string> {
    // if (!TAURI) return Promise.resolve("/");
    return (await invoke("get_memos_url")) as string;
}

/**
 * Ping Memos server.
 *
 * @returns true if /healthz endpoint returns "Service ready."
 */
export async function pingMemos(memosUrl: string, timeoutMillis = 1000): Promise<boolean> {
    return (await invoke("ping_memos", {
        memosUrl: memosUrl,
        timeoutMillis: timeoutMillis
    })) as boolean;
}

export async function getAppTheme(): Promise<string> {
    return (await invoke("get_theme")) as string;
}

export async function getReduceAnimationStatus(): Promise<boolean> {
    return (await invoke("get_reduce_animation_status")) as boolean;
}

/**
 * Get the current application locale.
 * @returns the current locale
 */
export async function getAppLocale(): Promise<string> {
    return (await invoke("get_locale")) as string;
}

/**
 * Set the application locale.
 * @param tag the locale to set
 * @returns true if the locale was set
 */
export async function setAppLocale(tag: string): Promise<boolean> {
    return (await invoke("set_locale", { new: tag })) as boolean;
}

/**
 * Get an environment variable.
 */
export async function getEnv(name: string): Promise<string> {
    return (await invoke("get_env", { name: name })) as string;
}

export async function getAppConfig(): Promise<string> {
    return (await invoke("get_config")) as string;
}

export async function getDefaultAppConfig(): Promise<string> {
    return (await invoke("get_default_config")) as string;
}

/**
 * Set the configuration.
 *
 * Takes a JSON Patch (RFC 6902) as the argument.
 * @param JSONPatch
 * @returns
 */
export async function setAppConfig(JSONPatch: string): Promise<boolean> {
    return (await invoke("set_config", { patch: JSONPatch })) as boolean;
}

/**
 * Check if a path exists.
 *
 * @param path
 * @returns boolean
 */
export async function pathExists(path: string): Promise<boolean> {
    return (await invoke("path_exists", { path: path })) as boolean;
}
