import { untrack } from "svelte";
import type { Locale } from "./i18n";

export type SectionActions = {
    loadDefaults?: () => Promise<void> | void;
    reloadCurrent?: () => Promise<void> | void;
    save?: () => Promise<boolean | undefined> | boolean | undefined;
    hasPendingChanges?: boolean;
};

export type OnSectionActionsChange = (actions: SectionActions) => void;

export type SectionActionsProps = {
    onActionsChange?: OnSectionActionsChange;
};

/**
 * Stable inputs for `useSectionActions`. The pending flag is read through a
 * getter so the reporting effect subscribes to section state without
 * re-reading the report callback itself.
 */
export type SectionActionSource = {
    loadDefaults: () => Promise<void> | void;
    reloadCurrent: () => Promise<void> | void;
    save: () => Promise<boolean | undefined> | boolean | undefined;
    hasPendingChanges: () => boolean;
};

/**
 * Reports a section's header actions to the settings page.
 *
 * Call once during component setup. The reporter and the action callbacks
 * are captured a single time, so the effect below subscribes only to the
 * pending flag: parent re-renders never resubscribe it, which keeps the
 * register/report pipeline from feeding back into itself.
 */
export function useSectionActions(
    source: SectionActionSource,
    onActionsChange?: OnSectionActionsChange
): void {
    const report = (actions: SectionActions) => onActionsChange?.(actions);
    const loadDefaults = () => source.loadDefaults();
    const reloadCurrent = () => source.reloadCurrent();
    const save = () => source.save();

    $effect(() => {
        const hasPendingChanges = source.hasPendingChanges();
        // The reporter and callbacks stay untracked: the effect subscribes
        // only to the pending flag, so parent re-renders never resubscribe it.
        untrack(() => {
            report({
                loadDefaults,
                reloadCurrent,
                save,
                hasPendingChanges
            });
        });
    });
}

function parseKeywords(value: string): string[] {
    return value
        .split(";")
        .map((entry) => entry.trim())
        .filter((entry) => entry.length > 0);
}

type AliasMessage = (inputs?: Record<string, never>, options?: { locale?: Locale }) => string;

export function keywordsFromLocale(message: AliasMessage): string[] {
    const currentKeywords = parseKeywords(message());
    const englishKeywords = parseKeywords(message({}, { locale: "en" }));

    return Array.from(new Set([...currentKeywords, ...englishKeywords]));
}
