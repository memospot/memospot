import { beforeEach, describe, expect, it, mock } from "bun:test";
import type { Operation } from "fast-json-patch";

import type { Config } from "../src/lib/types/gen/Config";
import type { ConfigUpdateResult } from "../src/lib/types/gen/ConfigUpdateResult";

const mockedSetAppConfig = mock<(patch: Operation[]) => Promise<ConfigUpdateResult>>();
// NOTE: mocked as a whole module shared with settingsSection.test.ts in the
// same run — the factory must provide every export either suite imports.
const mockedGetAppConfig = mock<() => Promise<Config>>();
const mockedGetDefaultAppConfig = mock<() => Promise<Config>>();
const mockedPathExists = mock<(path: string) => Promise<boolean>>();

mock.module("../src/lib/tauri", () => ({
    getAppConfig: mockedGetAppConfig,
    getDefaultAppConfig: mockedGetDefaultAppConfig,
    setAppConfig: mockedSetAppConfig,
    pathExists: mockedPathExists
}));

const { patchConfig } = await import("../src/lib/settings");

function configWithTheme(theme: string): Config {
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
                locale: null
            }
        }
    };
}

describe("patchConfig", () => {
    beforeEach(() => {
        mockedSetAppConfig.mockClear();
    });

    it("propagates the back-end error message when the write is rejected", async () => {
        mockedSetAppConfig.mockRejectedValue(new Error("persistence failed"));

        await expect(
            patchConfig(configWithTheme("light"), configWithTheme("dark"))
        ).rejects.toThrow("persistence failed");
    });

    it("returns the update result for updates that apply live", async () => {
        mockedSetAppConfig.mockResolvedValue({ restart_required: false });

        const result = await patchConfig(configWithTheme("light"), configWithTheme("dark"));

        expect(result).toEqual({ restart_required: false });
    });

    it("returns the update result when the update requires a restart", async () => {
        mockedSetAppConfig.mockResolvedValue({ restart_required: true });

        const result = await patchConfig(configWithTheme("light"), configWithTheme("dark"));

        expect(result).toEqual({ restart_required: true });
    });

    it("skips the request entirely when there is no diff", async () => {
        const config = configWithTheme("light");

        const result = await patchConfig(config, config);

        expect(result).toBe(false);
        expect(mockedSetAppConfig).not.toHaveBeenCalled();
    });
});
