/**
 * Settings-page search orchestration.
 *
 * Scans the rendered settings DOM for `data-setting-row` entries and keeps
 * them grouped by owning section. Every section stays mounted inside its own
 * `[data-section-root]` container (inactive sections are hidden), so the page
 * collects all sections on load and search covers every section immediately.
 */

import {
    collectSettingsEntriesBySection,
    hasSameSearchTerms,
    type SettingSearchEntry,
    type SettingsSectionLabel
} from "./settingsSearch";

export type SectionLabel = SettingsSectionLabel;

export type SettingsSearchIndex = {
    /** Flat view of every collected entry, for the search input. */
    readonly allEntries: SettingSearchEntry[];
    /** Sections that already contributed entries. */
    readonly collectedSectionIds: string[];
    /** (Re)collects every section rooted under `root`. Skips no-op writes. */
    collectAllSections(root: ParentNode | undefined): void;
};

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
        collectAllSections(root: ParentNode | undefined) {
            if (!root) return;

            const grouped = collectSettingsEntriesBySection(root, findSection);
            for (const [sectionId, entries] of Object.entries(grouped)) {
                const current = entriesBySection[sectionId];
                if (current && hasSameSearchTerms(current, entries)) continue;
                entriesBySection[sectionId] = entries;
            }
            for (const sectionId of Object.keys(entriesBySection)) {
                if (!(sectionId in grouped)) {
                    delete entriesBySection[sectionId];
                }
            }
        }
    } as SettingsSearchIndex;
}
