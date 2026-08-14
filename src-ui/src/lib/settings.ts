import * as jsonpatch from "fast-json-patch";
import { toast } from "svelte-sonner";
import { m } from "./i18n";
import { setAppConfig } from "./tauri";
import type { Config } from "./types/gen/Config";

/**
 * Generate a configuration patch (RFC 6902) and send it to the Tauri back-end.
 *
 * Rejected writes and unsuccessful results are handled as failures; successful
 * updates that require a restart surface a restart notice.
 */
export async function patchConfig(initial: Config, current: Config) {
    const diff = jsonpatch.compare(initial, current);

    if (Object.keys(diff).length === 0) return false;
    if (import.meta.env.DEV) console.log(diff);

    try {
        const result = await setAppConfig(JSON.stringify(diff));
        toast.success(m.settingsConfigSaveSuccess());
        if (result.restart_required) {
            toast.info(m.settingsConfigSaveRestartRequired(), {
                duration: 5000
            });
        }
        return Promise.resolve();
    } catch (_err) {
        toast.error(m.settingsConfigSaveFail());
        return Promise.reject();
    }
}
