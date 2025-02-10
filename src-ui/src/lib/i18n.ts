import * as messages from "$lib/paraglide/messages.js";
import * as runtime from "$lib/paraglide/runtime.js";
import { createI18n } from "@inlang/paraglide-sveltekit";
import { getAppLocale } from "./tauri";

// TODO: upgrade Paraglide to v2 when it's out of beta.
// Pre-adoption of Paraglide v2 naming convention.
/**
 * The project's available locales.
 */
export const locales = runtime.availableLanguageTags;
/**
 * Check if a locale is available on the project.
 */
export const isLocale = runtime.isAvailableLanguageTag;
/**
 * Set the current locale.
 */
export const setLocale = runtime.setLanguageTag;

export type Locale = (typeof locales)[number];

/**
 * Creates an i18n instance that manages internationalization.
 */
export const i18n = createI18n(runtime, {
    defaultLanguageTag: "en"
});

/**
 * Translated messages.
 *
 * Messages are defined in `src/i18n/{locale}.json`.
 *
 * See: [Paraglide-SvelteKit Docs](https://inlang.com/m/dxnzrydw/paraglide-sveltekit-i18n/getting-started#using-messages-in-code)
 */
export const m = messages;

/**
 * Locale fallbacks.
 *
 * Key: browser locale/short locale.
 * Value: app locale.
 */
const fallbacks: Record<string, string> = {
    "zh-HK": "zh-Hant",
    "zh-TW": "zh-Hant",
    zh: "zh-Hans"
};

/**
 * Detect the most appropriate translation to use based on the user's preference and current browser locale.
 *
 * Must be called after ParaglideJS is initialized.
 *
 * Order of preference:
 * 1. User's preferred locale.
 * 2. Exact browser locale.
 * 3. Configured fallbacks.
 * 4. Configured short code fallbacks.
 * 5. Short code.
 */
export function detectLocale(preferredLocale?: string) {
    if (typeof window === "undefined") return;

    const browserLocale = navigator.language;
    const shortLocale = browserLocale.slice(0, 2);

    const priorities = [
        preferredLocale,
        browserLocale,
        fallbacks[browserLocale],
        fallbacks[shortLocale]
    ];
    for (const locale of priorities) {
        if (locale && isLocale(locale)) {
            return setLocale(locale);
        }
    }

    for (const appLocale of locales) {
        const shortAppLocale = appLocale.slice(0, 2);
        if (shortLocale === shortAppLocale) {
            return setLocale(appLocale);
        }
    }
}

export async function initI18n() {
    if (typeof window === "undefined") return;

    return getAppLocale().then(
        (locale) => {
            detectLocale(locale);
        },
        (_err) => {
            return;
        }
    );
}
