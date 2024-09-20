/*
 * Tauri Handlers.
 *
 * The Rust back-end is in `src-tauri/src/js_handler.rs`.
 *
 */

import { invoke as TauriInvoke } from "@tauri-apps/api/core";

const browserError = new Error("Not running in Tauri!");
const invoke = globalThis.__TAURI__ ? TauriInvoke : async () => browserError.message;

/**
 * Get Memos URL.
 */
export function getMemosURL(): Promise<string> {
    return invoke("get_memos_url");
}

/**
 * Ping Memos server.
 */
export async function pingMemos(): Promise<boolean> {
    const result = (await invoke("ping_memos")) as string;
    return result === "true";
}

/**
 * Get an environment variable.
 */
export function getEnv(name: string): Promise<string> {
    return invoke("get_env", { name: name });
}
