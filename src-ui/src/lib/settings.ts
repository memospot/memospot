import * as jsonpatch from "fast-json-patch";
import { setAppConfig } from "./tauri";
import type { Config } from "./types/gen/Config";
import type { ConfigUpdateResult } from "./types/gen/ConfigUpdateResult";

/**
 * Generate a configuration patch (RFC 6902) and send it to the Tauri back-end.
 *
 * Returns the update result on success, or `false` when there is nothing to
 * send. Throws the back-end error message on failure; callers own toasts
 * and rollback decisions.
 */
export async function patchConfig(
    initial: Config,
    current: Config
): Promise<ConfigUpdateResult | false> {
    const diff = jsonpatch.compare(initial, current);

    if (Object.keys(diff).length === 0) return false;
    if (import.meta.env.DEV) console.log(diff);

    return await setAppConfig(diff);
}
