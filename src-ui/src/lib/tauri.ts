/*
 * Tauri Handlers.
 *
 * The Rust back-end is in `src-tauri/src/cmd`.
 *
 */
import { invoke as TauriInvoke, isTauri } from "@tauri-apps/api/core";
import { open as TauriOpen } from "@tauri-apps/plugin-shell";

const browserError = new Error("Not running in Tauri!");
const TAURI = isTauri();
const invoke = TAURI ? TauriInvoke : async () => browserError.message;

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
    if (!TAURI) return Promise.resolve("/");
    return invoke("get_memos_url");
}

/**
 * Ping Memos server.
 *
 * @returns true if /healthz endpoint returns "Service ready."
 */
export async function pingMemos(memosUrl: string, timeoutMillis = 1000): Promise<boolean> {
    if (!TAURI) return Promise.reject(browserError);
    return (
        (await invoke("ping_memos", { memosUrl: memosUrl, timeoutMillis: timeoutMillis })) ===
        true
    );
}

/**
 * Get the current application language.
 * @returns the current language
 */
export async function getAppLanguage(): Promise<string> {
    if (!TAURI) return Promise.reject(browserError);
    return invoke("get_language") as Promise<string>;
}

/**
 * Set the application language.
 * @param tag the language to set
 * @returns true if the language was set
 */
export async function setAppLanguage(tag: string): Promise<boolean> {
    if (!TAURI) return Promise.reject(browserError);
    return invoke("set_language", { new: tag }) as Promise<boolean>;
}

/**
 * Get an environment variable.
 */
export async function getEnv(name: string): Promise<string> {
    if (!TAURI) return Promise.reject(browserError);
    return invoke("get_env", { name: name }) as Promise<string>;
}

export async function getAppConfig(): Promise<string> {
    if (!TAURI) return Promise.reject(browserError);
    return invoke("get_config") as Promise<string>;
}

export async function getDefaultAppConfig(): Promise<string> {
    if (!TAURI) return Promise.reject(browserError);
    return invoke("get_default_config") as Promise<string>;
}

/**
 * Set the configuration.
 *
 * Takes a JSON Patch (RFC 6902) as the argument.
 * @param JSONPatch
 * @returns
 */
export async function setAppConfig(JSONPatch: string): Promise<boolean> {
    if (!TAURI) return Promise.reject(browserError);
    return invoke("set_config", { patch: JSONPatch }) as Promise<boolean>;
}

/**
 * Check if a path exists.
 *
 * @param path
 * @returns boolean
 */
export async function pathExists(path: string): Promise<boolean> {
    if (!TAURI) return Promise.reject(browserError);
    return invoke("path_exists", { path: path }) as Promise<boolean>;
}
