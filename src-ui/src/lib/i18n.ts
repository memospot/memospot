import * as messages from "$lib/paraglide/messages.js";
import * as runtime from "$lib/paraglide/runtime.js";
import { createI18n } from "@inlang/paraglide-sveltekit";

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
 */
export function detectLanguage() {
    if (typeof window === "undefined") return;

    const browserLanguage = navigator.language;
    const shortLanguage = browserLanguage.slice(0, 2);
    const userPreferredLanguage = localStorage.getItem(
        "i18n-user-preference"
    ) as runtime.AvailableLanguageTag | null;

    if (
        userPreferredLanguage &&
        runtime.availableLanguageTags.includes(userPreferredLanguage)
    ) {
        runtime.setLanguageTag(userPreferredLanguage);
        return;
    }

    for (const languageTag of runtime.availableLanguageTags) {
        if (
            [
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
}
