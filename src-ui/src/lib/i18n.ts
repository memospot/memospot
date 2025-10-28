import * as messages from "$lib/paraglide/messages.js";
import { baseLocale, getLocale, isLocale, locales, setLocale } from "$lib/paraglide/runtime.js";
import { getAppLocale } from "./tauri";

export type Locale = (typeof locales)[number];

/**
 * Translated messages.
 *
 * Messages are defined in `src/i18n/{locale}.json`.
 *
 * See: [Paraglide JS Docs](https://inlang.com/m/gerre34r/library-inlang-paraglideJs/message-keys)
 */
export const m = messages;

export {
    getLocale,
    isLocale,
    locales,
    localizeHref,
    setLocale
} from "$lib/paraglide/runtime.js";

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

const rtlLocales = ["ar", "he", "ur"];

/**
 * Check whether a locale is right-to-left or left-to-right.
 */
export function getTextDirection(locale?: Locale): "rtl" | "ltr" {
    return rtlLocales.includes(locale || "") ? "rtl" : "ltr";
}

/**
 * Detect the most appropriate translation to use based on the user's preference and current browser locale.
 *
 * Order of precedence:
 * 1. User's preferred locale.
 * 2. Exact browser locale.
 * 3. Configured fallbacks.
 * 4. Configured short code fallbacks.
 * 5. Short code.
 *
 * @returns Locale to use
 */
export function detectLocale(preferredLocale?: string | Locale): Locale {
    if (isLocale(preferredLocale)) {
        return preferredLocale as Locale;
    }

    if (navigator?.languages?.length) {
        for (const lang of navigator.languages) {
            if (isLocale(lang)) {
                return lang;
            }

            const fallback = fallbacks[lang];
            if (isLocale(fallback)) {
                return fallback;
            }

            const baseLang = lang.slice(0, 2);
            const baseFallback = fallbacks[baseLang];
            if (isLocale(baseFallback)) {
                return baseFallback;
            }
        }
    }

    for (const locale of locales) {
        if (locale.slice(0, 2) === preferredLocale?.slice(0, 2)) {
            return locale;
        }
    }

    return baseLocale;
}

/**
 * Initialize internationalization settings for the application.
 *
 * This function:
 * 1. Checks if running in browser environment
 * 2. Gets the app's locale preference
 * 3. Detects the most appropriate locale
 * 4. Sets the locale if different from current
 *
 * @returns Promise that resolves when initialization is complete
 */
export async function initI18n(): Promise<void> {
    if (typeof window === "undefined") return;

    try {
        const storedLocale = await getAppLocale();
        const detectedLocale = detectLocale(storedLocale);

        if (detectedLocale === getLocale()) {
            return;
        }

        setLocale(detectedLocale);
    } catch (err) {
        console.warn("Failed to load stored app locale:", err);
        setLocale(getLocale());
    }
}
