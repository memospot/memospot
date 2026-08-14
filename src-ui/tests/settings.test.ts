import { beforeEach, describe, expect, it, mock } from "bun:test";

import type { ConfigUpdateResult } from "../src/lib/types/gen/ConfigUpdateResult";

const mockedSetAppConfig = mock<(patch: string) => Promise<ConfigUpdateResult>>();
const toast = {
    success: mock<(message: string) => void>(),
    error: mock<(message: string) => void>(),
    info: mock<(message: string, options: { duration: number }) => void>()
};

mock.module("../src/lib/tauri", () => ({
    setAppConfig: mockedSetAppConfig
}));

mock.module("svelte-sonner", () => ({
    toast
}));

mock.module("../src/lib/i18n", () => ({
    m: {
        settingsConfigSaveSuccess: () => "saved",
        settingsConfigSaveFail: () => "failed",
        settingsConfigSaveRestartRequired: () => "restart required"
    }
}));

import type { Config } from "../src/lib/types/gen/Config";

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
        toast.success.mockClear();
        toast.error.mockClear();
        toast.info.mockClear();
    });

    it("rejects and shows an error toast when the write is rejected", async () => {
        mockedSetAppConfig.mockRejectedValue(new Error("persistence failed"));

        await expect(
            patchConfig(configWithTheme("light"), configWithTheme("dark"))
        ).rejects.toBeUndefined();

        expect(toast.error).toHaveBeenCalledWith("failed");
        expect(toast.success).not.toHaveBeenCalled();
        expect(toast.info).not.toHaveBeenCalled();
    });

    it("shows only a success toast for updates that apply live", async () => {
        mockedSetAppConfig.mockResolvedValue({ restart_required: false });

        await patchConfig(configWithTheme("light"), configWithTheme("dark"));

        expect(toast.success).toHaveBeenCalledWith("saved");
        expect(toast.info).not.toHaveBeenCalled();
        expect(toast.error).not.toHaveBeenCalled();
    });

    it("shows a restart notice when the update requires a restart", async () => {
        mockedSetAppConfig.mockResolvedValue({ restart_required: true });

        await patchConfig(configWithTheme("light"), configWithTheme("dark"));

        expect(toast.success).toHaveBeenCalledWith("saved");
        expect(toast.info).toHaveBeenCalledWith("restart required", expect.any(Object));
        expect(toast.error).not.toHaveBeenCalled();
    });

    it("skips the request entirely when there is no diff", async () => {
        const config = configWithTheme("light");

        const result = await patchConfig(config, config);

        expect(result).toBe(false);
        expect(mockedSetAppConfig).not.toHaveBeenCalled();
        expect(toast.success).not.toHaveBeenCalled();
        expect(toast.error).not.toHaveBeenCalled();
        expect(toast.info).not.toHaveBeenCalled();
    });
});
