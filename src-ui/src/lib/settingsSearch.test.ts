import { describe, expect, it } from "bun:test";
import { collectSettingsEntries, hasSameSearchTerms } from "./settingsSearch";

type RowSpec = {
    id?: string;
    label?: string;
    keywords?: string;
};

type FakeRow = {
    dataset: {
        settingId: string | undefined;
        settingLabel: string | undefined;
        settingKeywords: string | undefined;
    };
};

function rowElement(spec: RowSpec): FakeRow {
    return {
        dataset: {
            settingId: spec.id,
            settingLabel: spec.label,
            settingKeywords: spec.keywords
        }
    };
}

function parentWithRows(rows: FakeRow[]): ParentNode {
    return {
        querySelectorAll: () => rows
    } as unknown as ParentNode;
}

describe("collectSettingsEntries", () => {
    it("collects entries with their owning section id", () => {
        // GIVEN a rendered section with two setting rows
        const root = parentWithRows([
            rowElement({ id: "view-theme", label: "Theme", keywords: "dark|light" }),
            rowElement({ id: "view-locale", label: "Language" })
        ]);

        // WHEN entries are collected for the section
        const entries = collectSettingsEntries("view", "View", root);

        // THEN every entry carries the owning section id and label
        expect(entries).toEqual([
            {
                id: "view-theme",
                label: "Theme",
                keywords: ["dark", "light"],
                sectionId: "view",
                sectionLabel: "View"
            },
            {
                id: "view-locale",
                label: "Language",
                keywords: [],
                sectionId: "view",
                sectionLabel: "View"
            }
        ]);
    });

    it("returns no entries for an empty section DOM", () => {
        // GIVEN a section whose DOM has no setting rows yet
        const root = parentWithRows([]);

        // WHEN entries are collected for the section
        const entries = collectSettingsEntries("memos", "Memos", root);

        // THEN the section contributes nothing to the index
        expect(entries).toEqual([]);
    });

    it("isolates entries by owning section", () => {
        // GIVEN two rendered sections with distinct rows
        const viewRoot = parentWithRows([rowElement({ id: "view-theme", label: "Theme" })]);
        const memosRoot = parentWithRows([
            rowElement({ id: "memos-mode", label: "Mode", keywords: "prod|dev" })
        ]);

        // WHEN entries are collected per section
        const viewEntries = collectSettingsEntries("view", "View", viewRoot);
        const memosEntries = collectSettingsEntries("memos", "Memos", memosRoot);

        // THEN each result carries only its own section id and label
        expect(viewEntries).toEqual([
            {
                id: "view-theme",
                label: "Theme",
                keywords: [],
                sectionId: "view",
                sectionLabel: "View"
            }
        ]);
        expect(memosEntries).toEqual([
            {
                id: "memos-mode",
                label: "Mode",
                keywords: ["prod", "dev"],
                sectionId: "memos",
                sectionLabel: "Memos"
            }
        ]);
    });

    it("keeps duplicate row ids to a single entry", () => {
        // GIVEN a section rendering the same row twice
        const root = parentWithRows([
            rowElement({ id: "memospot-logging", label: "Logging" }),
            rowElement({ id: "memospot-logging", label: "Logging" })
        ]);

        // WHEN entries are collected for the section
        const entries = collectSettingsEntries("memospot", "Memospot", root);

        // THEN only one entry is indexed
        expect(entries).toEqual([
            {
                id: "memospot-logging",
                label: "Logging",
                keywords: [],
                sectionId: "memospot",
                sectionLabel: "Memospot"
            }
        ]);
    });

    it("skips rows without setting metadata", () => {
        // GIVEN a section with one unlabeled row next to a valid row
        const root = parentWithRows([
            rowElement({ label: "Missing id" }),
            rowElement({ id: "view-theme", label: "Theme" })
        ]);

        // WHEN entries are collected for the section
        const entries = collectSettingsEntries("view", "View", root);

        // THEN only the fully-described row is indexed
        expect(entries).toEqual([
            {
                id: "view-theme",
                label: "Theme",
                keywords: [],
                sectionId: "view",
                sectionLabel: "View"
            }
        ]);
    });
});

// GIVEN a previously collected entry list
// WHEN comparing against a re-collected list
// THEN identical entries are a no-op and changed terms force a rewrite
describe("hasSameSearchTerms", () => {
    const base = [
        {
            id: "view-theme",
            label: "Theme",
            keywords: ["dark", "light"],
            sectionId: "view",
            sectionLabel: "View"
        }
    ];

    it("treats identical entries as the same", () => {
        // WHEN re-collected entries are identical
        // THEN they compare as the same and no rewrite is needed
        expect(hasSameSearchTerms(base, structuredClone(base))).toBe(true);
    });

    it("treats a changed label as different", () => {
        // GIVEN a previously collected entry
        const next = structuredClone(base);
        next[0].label = "Appearance";

        // WHEN the label changes under the same id
        // THEN the entries compare as different and a rewrite is needed
        expect(hasSameSearchTerms(base, next)).toBe(false);
    });

    it("treats changed keywords as different", () => {
        // GIVEN a previously collected entry
        const next = structuredClone(base);
        next[0].keywords = ["dark", "system"];

        // WHEN the keywords change
        // THEN the entries compare as different and a rewrite is needed
        expect(hasSameSearchTerms(base, next)).toBe(false);
    });

    it("treats a changed id as different", () => {
        // GIVEN a previously collected entry
        const next = structuredClone(base);
        next[0].id = "view-mode";

        // WHEN the id changes
        // THEN the entries compare as different and a rewrite is needed
        expect(hasSameSearchTerms(base, next)).toBe(false);
    });

    it("treats an added setting as different", () => {
        // GIVEN a previously collected entry
        const next = structuredClone(base);
        next.push({ ...structuredClone(base[0]), id: "view-animation" });

        // WHEN a setting is added
        // THEN the entries compare as different and a rewrite is needed
        expect(hasSameSearchTerms(base, next)).toBe(false);
    });

    it("treats a changed section label as different", () => {
        // GIVEN a previously collected entry
        const next = structuredClone(base);
        next[0].sectionLabel = "Appearance";

        // WHEN the section label changes
        // THEN the entries compare as different and a rewrite is needed
        expect(hasSameSearchTerms(base, next)).toBe(false);
    });
});
