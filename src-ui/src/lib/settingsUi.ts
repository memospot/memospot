export type SectionActions = {
    loadDefaults?: () => Promise<void> | void;
    reloadCurrent?: () => Promise<void> | void;
    save?: () => Promise<boolean | void> | boolean | void;
    hasPendingChanges?: boolean;
};

export type OnSectionActionsChange = (actions: SectionActions) => void;

export type SectionActionsProps = {
    onActionsChange?: OnSectionActionsChange;
};

type AliasMessage = (inputs?: Record<string, never>, options?: { locale?: string }) => string;

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

function parseAliases(value: string): string[] {
    return value
        .split(",")
        .map((entry) => entry.trim())
        .filter((entry) => entry.length > 0);
}

export function aliasesFromLocale(message: AliasMessage): string[] {
    const currentAliases = parseAliases(message());
    const englishAliases = parseAliases(message({}, { locale: "en" }));

    return Array.from(new Set([...currentAliases, ...englishAliases]));
}
