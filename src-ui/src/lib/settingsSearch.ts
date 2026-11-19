export type SettingSearchEntry = {
    id: string;
    label: string;
    keywords: string[];
    sectionId: string;
    sectionLabel: string;
};

function parseKeywords(raw: string | undefined): string[] {
    if (!raw) return [];
    return raw
        .split("|")
        .map((value) => value.trim())
        .filter((value) => value.length > 0);
}

/** Index-aligned comparison: arrays must be in the same order and lengths equal, and keyword order is significant. */
export function hasSameSearchTerms(
    current: SettingSearchEntry[],
    next: SettingSearchEntry[]
): boolean {
    return (
        current.length === next.length &&
        current.every(
            (entry, index) =>
                entry.id === next[index].id &&
                entry.label === next[index].label &&
                entry.sectionId === next[index].sectionId &&
                entry.sectionLabel === next[index].sectionLabel &&
                entry.keywords.length === next[index].keywords.length &&
                entry.keywords.every(
                    (keyword, keywordIndex) => keyword === next[index].keywords[keywordIndex]
                )
        )
    );
}

export function collectSettingsEntries(
    sectionId: string,
    sectionLabel: string,
    root: ParentNode
): SettingSearchEntry[] {
    const rows = Array.from(root.querySelectorAll<HTMLElement>("[data-setting-row='true']"));
    const seenIds = new Set<string>();
    const entries: SettingSearchEntry[] = [];

    for (const row of rows) {
        const id = row.dataset.settingId?.trim() ?? "";
        const label = row.dataset.settingLabel?.trim() ?? "";

        if (!id || !label) {
            if (import.meta.env.DEV) {
                console.warn("[settings-search] Missing setting metadata", {
                    sectionId,
                    id,
                    label,
                    row
                });
            }
            continue;
        }

        if (seenIds.has(id)) {
            continue;
        }

        seenIds.add(id);
        entries.push({
            id,
            label,
            keywords: parseKeywords(row.dataset.settingKeywords),
            sectionId,
            sectionLabel
        });
    }

    return entries;
}

export type SettingsSectionLabel = {
    id: string;
    label: string;
};

/**
 * Collects search entries grouped by owning section.
 *
 * Scans `root` for `[data-section-root]` containers, resolves each container
 * to its section, and collects that container's rows only. Containers without
 * a known section are skipped so stray markup never enters the index.
 */
export function collectSettingsEntriesBySection(
    root: ParentNode,
    findSection: (id: string) => SettingsSectionLabel | undefined
): Record<string, SettingSearchEntry[]> {
    const sectionRoots = Array.from(root.querySelectorAll<HTMLElement>("[data-section-root]"));
    const grouped: Record<string, SettingSearchEntry[]> = {};

    for (const sectionRoot of sectionRoots) {
        const sectionId = sectionRoot.dataset.sectionRoot?.trim() ?? "";
        if (!sectionId) continue;

        const section = findSection(sectionId);
        if (!section) continue;

        grouped[section.id] = collectSettingsEntries(section.id, section.label, sectionRoot);
    }

    return grouped;
}
