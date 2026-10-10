import { describe, expect, it } from "bun:test";
import { collectSettingsEntries } from "./settingsSearch";

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
