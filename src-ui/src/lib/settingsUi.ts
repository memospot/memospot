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

type AliasMessage = (inputs?: Record<string, never>, options?: { locale?: Locale }) => string;

export function buildSectionActions(
    loadDefaults: SectionActions["loadDefaults"],
    reloadCurrent: SectionActions["reloadCurrent"],
    save: SectionActions["save"],
    hasPendingChanges: boolean
): SectionActions {
    return {
        loadDefaults,
        reloadCurrent,
        save,
        hasPendingChanges
    };
}

function parseKeywords(value: string): string[] {
    return value
        .split(";")
        .map((entry) => entry.trim())
        .filter((entry) => entry.length > 0);
}

export function keywordsFromLocale(message: AliasMessage): string[] {
    const currentKeywords = parseKeywords(message());
    const englishKeywords = parseKeywords(message({}, { locale: "en" }));

    return Array.from(new Set([...currentKeywords, ...englishKeywords]));
}
