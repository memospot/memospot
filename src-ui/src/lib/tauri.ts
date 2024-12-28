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
    return TAURI ? invoke("get_memos_url") : Promise.resolve("/");
}

/**
 * Ping Memos server.
 */
export async function pingMemos(): Promise<boolean> {
    return TAURI ? (await invoke("ping_memos")) === "true" : Promise.resolve(true);
}

/**
 * Get an environment variable.
 */
export async function getEnv(name: string): Promise<string> {
    return TAURI ? invoke("get_env", { name: name }) : Promise.resolve("");
}
