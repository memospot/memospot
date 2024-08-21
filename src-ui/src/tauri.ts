import { invoke as TauriInvoke } from "@tauri-apps/api/tauri";

const browserError = new Error("Not running in Tauri!");
const invoke = window.__TAURI__ ? TauriInvoke : async () => browserError.message;

/**
 * Tauri Handlers.
 *
 * The Rust back-end is in `src-tauri/src/js_handler.rs`.
 */
export class Tauri {
    static getMemosURL(): Promise<string> {
        return invoke("get_memos_url");
    }

    static getEnv(name: string): Promise<string> {
        return invoke("get_env", { name: name });
    }
}
