/**
 * Settings-page search orchestration.
 *
 * Scans the rendered settings DOM for `data-setting-row` entries and keeps
 * them grouped by owning section. Indexing stays lazy: a section is
 * collected only after it renders (the page calls `collectForSection` from
 * its content-pane attach callback and on section switches), so unvisited
 * sections never enter the index.
 */

import { collectSettingsEntries, type SettingSearchEntry } from "./settingsSearch";

export type SectionLabel = {
    id: string;
    label: string;
};

export type SettingsSearchIndex = {
    /** Flat view of every collected entry, for the search input. */
    readonly allEntries: SettingSearchEntry[];
    /** Sections that already contributed entries. */
    readonly collectedSectionIds: string[];
    /** (Re)collects the entries of one rendered section. Skips no-op writes. */
    collectForSection(sectionId: string, root: ParentNode | undefined): void;
};

function sameEntryIds(current: SettingSearchEntry[], next: SettingSearchEntry[]): boolean {
    return (
        current.length === next.length &&
        current.every((entry, index) => entry.id === next[index].id)
    );
}

export function createSettingsSearchIndex(
    findSection: (id: string) => SectionLabel | undefined
) {
    let entriesBySection: Record<string, SettingSearchEntry[]> = $state({});

    return {
        get allEntries() {
            return Object.values(entriesBySection).flat();
        },
        get collectedSectionIds() {
            return Object.keys(entriesBySection);
        },
        collectForSection(sectionId: string, root: ParentNode | undefined) {
            const section = findSection(sectionId);
            if (!section || !root) return;

            const entries = collectSettingsEntries(section.id, section.label, root);
            const current = entriesBySection[sectionId];
            if (current && sameEntryIds(current, entries)) return;
            entriesBySection[sectionId] = entries;
        }
    } as SettingsSearchIndex;
}
