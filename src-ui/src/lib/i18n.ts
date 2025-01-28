import * as messages from "$lib/paraglide/messages.js";
import * as runtime from "$lib/paraglide/runtime.js";
import { createI18n } from "@inlang/paraglide-sveltekit";
import { getAppLanguage } from "./tauri";

export type Language = (typeof runtime.availableLanguageTags)[number];

/**
 * Creates an i18n instance that manages internationalization.
 */
export const i18n = createI18n(runtime, {
    defaultLanguageTag: "en"
});

/**
 * Translated messages.
 *
 * Messages are defined in `src/i18n/{languageTag}.json`.
 *
 * See: [Paraglide-SvelteKit Docs](https://inlang.com/m/dxnzrydw/paraglide-sveltekit-i18n/getting-started#using-messages-in-code)
 */
export const m = messages;

/**
 * Fallbacks for languages.
 *
 * Key: browser language tag/short language tag.
 * Value: app language tag.
 */
const fallbacks: Record<string, string> = {
    "zh-HK": "zh-Hant",
    "zh-TW": "zh-Hant",
    zh: "zh-Hans"
};

/**
 * Detect the most appropriate translation to use based on the user's preference and current browser language.
 *
 * Must be called after ParaglideJS is initialized.
 */
export function detectLanguage(userPreferredLanguage?: string) {
    if (typeof window === "undefined") return;

    const browserLanguage = navigator.language;
    const shortLanguage = browserLanguage.slice(0, 2);

    for (const languageTag of runtime.availableLanguageTags) {
        if (
            [
                userPreferredLanguage === languageTag,
                browserLanguage === languageTag,
                fallbacks[browserLanguage] === languageTag,
                fallbacks[shortLanguage] === languageTag,
                shortLanguage === languageTag,
                shortLanguage === languageTag.slice(0, 2)
            ].some(Boolean)
        ) {
            runtime.setLanguageTag(languageTag);
            break;
        }
    }
    return "";
}

export async function initI18n() {
    if (typeof window === "undefined") return;

    return getAppLanguage().then(
        (language) => {
            detectLanguage(language);
        },
        (_err) => {
            return;
        }
    );
}
