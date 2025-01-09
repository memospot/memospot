import * as jsonpatch from "fast-json-patch";
import { toast } from "svelte-sonner";
import type { Config } from "./bindings/Config";
import { m } from "./i18n";
import { setConfig } from "./tauri";

/**
 * Generate a configuration patch (RFC 6902) and send it to the Tauri back-end.
 * @param config
 */
export async function patchConfig(initial: Config, current: Config) {
    const diff = jsonpatch.compare(initial, current);

    if (Object.keys(diff).length === 0) return false;
    if (import.meta.env.DEV) console.log(diff);

    return await setConfig(JSON.stringify(diff)).then(
        () => {
            toast.success(m.configSaveSuccess());
            return Promise.resolve(true);
        },
        () => {
            toast.error(m.configSaveFail());
            return Promise.reject(false);
        }
    );
}
