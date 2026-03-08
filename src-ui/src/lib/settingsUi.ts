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

export function aliasesFromLocale(value: string): string[] {
    return value
        .split(",")
        .map((entry) => entry.trim())
        .filter((entry) => entry.length > 0);
}
