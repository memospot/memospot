import * as jsonpatch from "fast-json-patch";
import { toast } from "svelte-sonner";
import { m } from "./i18n";
import { setAppConfig } from "./tauri";
import type { Config } from "./types/gen/Config";

/**
 * Generate a configuration patch (RFC 6902) and send it to the Tauri back-end.
 * @param config
 */
export async function patchConfig(initial: Config, current: Config) {
    const diff = jsonpatch.compare(initial, current);

    if (Object.keys(diff).length === 0) return false;
    if (import.meta.env.DEV) console.log(diff);

    return await setAppConfig(JSON.stringify(diff)).then(
        (_ok) => {
            toast.success(m.settingsConfigSaveSuccess());
            return Promise.resolve();
        },
        (_err) => {
            toast.error(m.settingsConfigSaveFail());
            return Promise.reject();
        }
    );
}
