export type SettingSearchEntry = {
    id: string;
    label: string;
    aliases: string[];
    sectionId: string;
    sectionLabel: string;
};

function parseAliases(raw: string | undefined): string[] {
    if (!raw) return [];
    return raw
        .split("|")
        .map((value) => value.trim())
        .filter((value) => value.length > 0);
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
            aliases: parseAliases(row.dataset.settingAliases),
            sectionId,
            sectionLabel
        });
    }

    return entries;
}
