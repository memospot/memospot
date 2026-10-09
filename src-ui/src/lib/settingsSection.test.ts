import { beforeEach, describe, expect, it, mock } from "bun:test";
import type { Operation } from "fast-json-patch";
import type { Config } from "./types/gen/Config";
import type { ConfigUpdateResult } from "./types/gen/ConfigUpdateResult";

type Input = { theme: string; locale: string };

function baseConfig(theme: string, locale: string | null): Config {
    return {
        memos: {
            binary_path: null,
            working_dir: null,
            data: null,
            demo: false,
            mode: "prod",
            addr: "127.0.0.1",
            port: 5230,
            env: { enabled: false, vars: null }
        },
        memospot: {
            backups: { enabled: true, path: null },
            env: { enabled: false, vars: null },
            migrations: { enabled: true },
            log: { enabled: false },
            remote: { enabled: false, url: null, user_agent: null },
            updater: { enabled: true, check_interval: "3d", last_check: null },
            window: {
                center: true,
                fullscreen: false,
                resizable: true,
                maximized: false,
                width: 1280,
                height: 720,
                x: 0,
                y: 0,
                hide_menu_bar: false,
                theme: theme,
                reduce_animation: false,
                locale: locale
            }
        }
    };
}

const defaultConfig = baseConfig("system", null);
const initialConfig = baseConfig("light", "en");

const mockedSetAppConfig = mock<(patch: Operation[]) => Promise<ConfigUpdateResult>>();
const mockedGetAppConfig = mock<() => Promise<Config>>();
const mockedGetDefaultAppConfig = mock<() => Promise<Config>>();
const mockedPathExists = mock<(path: string) => Promise<boolean>>();

// NOTE: mocked as a whole module shared with tests/settings.test.ts in the
// same run — the factory must provide every export either suite imports.
mock.module("./tauri", () => ({
    getAppConfig: mockedGetAppConfig,
    getDefaultAppConfig: mockedGetDefaultAppConfig,
    setAppConfig: mockedSetAppConfig,
    pathExists: mockedPathExists
}));

mock.module("./i18n", () => ({
    m: {
        settingsConfigSaveSuccess: () => "saved",
        settingsConfigSaveFail: () => "failed",
        settingsConfigSaveRestartRequired: () => "restart required",
        settingsErrPathDoesNotExist: () => "path does not exist",
        settingsMemospotErrInvalidServer: () => "invalid server"
    }
}));

const { createSettingsSection } = await import("./settingsSection");

function makeSection(opts?: {
    live?: (keyof Input)[];
    validate?: (input: Input) => Partial<Record<keyof Input, string>>;
}) {
    return createSettingsSection<Input>({
        mapping: {
            initial: { theme: "system", locale: "system" },
            mapToInput: (cfg) => ({
                theme: (cfg.memospot.window.theme ?? "system") as string,
                locale: (cfg.memospot.window.locale ?? "system") as string
            }),
            mapFromInput: (input, cfg) => {
                cfg.memospot.window.theme = input.theme;
                cfg.memospot.window.locale = input.locale;
            },
            validate: opts?.validate,
            live: opts?.live
        }
    });
}

beforeEach(() => {
    mockedSetAppConfig.mockClear();
    mockedSetAppConfig.mockResolvedValue({ restart_required: false });
    mockedGetAppConfig.mockClear();
    mockedGetDefaultAppConfig.mockClear();
    mockedPathExists.mockClear();
    mockedPathExists.mockResolvedValue(true);
    mockedGetAppConfig.mockResolvedValue(initialConfig);
    mockedGetDefaultAppConfig.mockResolvedValue(defaultConfig);
});

describe("settingsSection — init and pending", () => {
    it("maps baseline config to input and reports not pending", async () => {
        const section = makeSection();
        await section.init();
        expect(section.input.theme).toBe("light");
        expect(section.input.locale).toBe("en");
        expect(section.hasPendingChanges).toBe(false);
    });

    it("is not pending before init and no save before init", async () => {
        const section = makeSection();
        expect(section.hasPendingChanges).toBe(false);
        const result = await section.save();
        expect(result).toBe(false);
        expect(mockedSetAppConfig).not.toHaveBeenCalled();
    });

    it("editing flips pending, reset restores false", async () => {
        const section = makeSection();
        await section.init();
        section.input.theme = "dark";
        expect(section.hasPendingChanges).toBe(true);
        section.reset();
        expect(section.input.theme).toBe("light");
        expect(section.hasPendingChanges).toBe(false);
    });

    it("loadDefaults stages defaults as pending until save (no persistence yet)", async () => {
        const section = makeSection();
        await section.init();
        await section.loadDefaults();
        expect(section.input.theme).toBe("system");
        // loadDefaults stages defaults in input only; baseline stays saved (light)
        // so hasPendingChanges becomes true and save diffs against saved config.
        expect(section.hasPendingChanges).toBe(true);
        expect(mockedSetAppConfig).not.toHaveBeenCalled();
    });

    it("successful save advances baseline and clears pending", async () => {
        const section = makeSection();
        await section.init();
        section.input.theme = "dark";
        const result = await section.save();
        expect(result).toEqual({ restart_required: false });
        expect(mockedSetAppConfig).toHaveBeenCalledTimes(1);
        expect(section.hasPendingChanges).toBe(false);
        expect(section.baselineInput.theme).toBe("dark");
    });

    it("save returns the restart notice when backend reports restart_required", async () => {
        mockedSetAppConfig.mockResolvedValue({ restart_required: true });
        const section = makeSection();
        await section.init();
        section.input.theme = "dark";
        const result = await section.save();
        expect(result).toEqual({ restart_required: true });
    });

    it("failed save throws and leaves input for the caller to roll back", async () => {
        mockedSetAppConfig.mockRejectedValue(new Error("persistence failed"));
        const section = makeSection();
        await section.init();
        section.input.theme = "dark";
        await expect(section.save()).rejects.toThrow("persistence failed");
        expect(section.input.theme).toBe("dark");
    });

    it("empty diff performs no write and returns false", async () => {
        const section = makeSection();
        await section.init();
        const result = await section.save();
        expect(result).toBe(false);
        expect(mockedSetAppConfig).not.toHaveBeenCalled();
    });

    it("each save persists independently", async () => {
        const section = makeSection();
        await section.init();
        section.input.theme = "dark";
        expect(await section.save()).toEqual({ restart_required: false });
        expect(mockedSetAppConfig).toHaveBeenCalledTimes(1);
        section.input.theme = "light";
        expect(await section.save()).toEqual({ restart_required: false });
        expect(mockedSetAppConfig).toHaveBeenCalledTimes(2);
    });

    it("validation throws field messages without persistence", async () => {
        const section = makeSection({
            validate: (input) => (input.theme === "bad" ? { theme: "invalid" } : {})
        });
        await section.init();
        section.input.theme = "bad";
        await expect(section.save()).rejects.toEqual(["invalid"]);
        expect(mockedSetAppConfig).not.toHaveBeenCalled();
    });

    it("live field is excluded from pending and from patch", async () => {
        const section = makeSection({ live: ["locale"] });
        await section.init();
        section.input.locale = "fr-FR";
        expect(section.hasPendingChanges).toBe(false);
        const result = await section.save();
        expect(result).toBe(false);
        expect(mockedSetAppConfig).not.toHaveBeenCalled();
        section.input.theme = "dark";
        section.input.locale = "fr-FR";
        const result2 = await section.save();
        expect(result2).toEqual({ restart_required: false });
        const patch = mockedSetAppConfig.mock.calls[0][0] as Operation[];
        expect(patch.some((op) => op.path.includes("locale"))).toBe(false);
    });

    it("commitLive advances baseline so field stays non-pending and excluded", async () => {
        const section = makeSection({ live: ["locale"] });
        await section.init();
        section.commitLive("locale", "fr-FR");
        expect(section.input.locale).toBe("fr-FR");
        expect(section.baselineInput.locale).toBe("fr-FR");
        expect(section.hasPendingChanges).toBe(false);
        section.input.theme = "dark";
        const result = await section.save();
        expect(result).toEqual({ restart_required: false });
        const patch = mockedSetAppConfig.mock.calls[0][0] as Operation[];
        expect(patch.some((op: Operation) => op.path.includes("locale"))).toBe(false);
    });
});
