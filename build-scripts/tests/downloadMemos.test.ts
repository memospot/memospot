import { describe, expect, test } from "bun:test";
import * as Bun from "bun";
import {
    CROSS_COMPILE_PLATFORMS,
    getDownloadFilesGlob,
    getRequestedTag,
    RUST2GO,
    SUPPORTED_BUILDS
} from "../bin/downloadMemos";
import { makeTripletFromFileName } from "../lib/util";

describe("match assets according to environment", async () => {
    test("dev machine", () => {
        const platformExpectations: Record<string, string[]> = {
            darwin: SUPPORTED_BUILDS.filter((p) => p.includes("darwin")),
            win32: SUPPORTED_BUILDS.filter((p) => p.includes("windows")),
            linux: SUPPORTED_BUILDS.filter((p) => p.includes("linux"))
        };

        const mockArchitectures = ["arm64", "x86_64"];
        const mockPlatforms = Object.keys(platformExpectations);

        for (const arch of mockArchitectures) {
            for (const platform of mockPlatforms) {
                const platformExpectation = platformExpectations[platform].filter((p) =>
                    p.includes(arch)
                );
                Object.defineProperty(process, "arch", {
                    value: arch
                });
                Object.defineProperty(process, "platform", {
                    value: platform
                });

                const downloadFilesGlob = getDownloadFilesGlob();
                expect(downloadFilesGlob).toEqual(platformExpectation);
            }
        }
    });

    test("CI=true", () => {
        const originalCI = process.env.CI;
        const originalArch = process.arch;
        const originalPlatform = process.platform;

        const overrideProcessProp = (prop: "arch" | "platform", value: string) => {
            Object.defineProperty(process, prop, {
                value,
                configurable: true
            });
        };

        try {
            process.env.CI = "true";

            const mockArchitectures = ["arm64", "x86_64"];
            const mockPlatforms = ["darwin", "win32", "linux"];

            for (const arch of mockArchitectures) {
                for (const platform of mockPlatforms) {
                    overrideProcessProp("arch", arch);
                    overrideProcessProp("platform", platform);

                    const downloadFilesGlob = getDownloadFilesGlob();
                    const hostTriplet = makeTripletFromFileName(
                        `${platform.replace("win32", "windows")}-${arch.replace("x64", "x86_64")}`
                    );

                    const crossTargets: readonly (keyof typeof RUST2GO)[] =
                        hostTriplet in CROSS_COMPILE_PLATFORMS
                            ? CROSS_COMPILE_PLATFORMS[
                                  hostTriplet as keyof typeof CROSS_COMPILE_PLATFORMS
                              ]
                            : [];

                    const expected = [
                        ...new Set([...crossTargets, hostTriplet as keyof typeof RUST2GO])
                    ]
                        .reduce<string[]>((acc, triplet) => {
                            if (triplet in RUST2GO) {
                                acc.push(RUST2GO[triplet as keyof typeof RUST2GO]);
                            }
                            return acc;
                        }, [])
                        .sort();

                    expect(downloadFilesGlob).toEqual(expected);
                }
            }
        } finally {
            process.env.CI = originalCI;
            overrideProcessProp("arch", originalArch);
            overrideProcessProp("platform", originalPlatform);
        }
    });

    test("RUST_TARGET env, single target", () => {
        const rust_targets = Object.keys(RUST2GO);
        for (const target of rust_targets) {
            process.env.RUST_TARGET = target;

            const downloadFilesGlob = getDownloadFilesGlob();
            const expected = [RUST2GO[target as keyof typeof RUST2GO]];

            expect(downloadFilesGlob).toEqual(expected);
        }
    });
    test("RUST_TARGET env, multiple targets", () => {
        const rust_targets = Object.keys(RUST2GO);
        const first_two = rust_targets.slice(0, 2);

        process.env.RUST_TARGET = first_two.join(",");
        console.warn(`RUST_TARGET=${process.env.RUST_TARGET}`);

        const go_targets = Object.values(RUST2GO).slice(0, 2);
        console.warn(`Go targets=${go_targets}`);

        const downloadFilesGlob = getDownloadFilesGlob();

        const expected = go_targets;
        expect(downloadFilesGlob).toEqual(expected);
    });
});

describe("test getRequestedTag", async () => {
    test("env prefixed", () => {
        process.env.MEMOS_VERSION = "v1.2.3";
        const tag = getRequestedTag();

        expect(tag).toBe("v1.2.3");
    });

    test("env no prefix", () => {
        process.env.MEMOS_VERSION = "1.2.3";
        const tag = getRequestedTag();

        expect(tag).toBe("v1.2.3");
    });

    test("env undefined", () => {
        process.env.MEMOS_VERSION = undefined;
        const tag = getRequestedTag();

        expect(tag).toBe(null);
    });

    test("argv prefixed", () => {
        const mockArgs = ["downloadMemos.ts", "--tag=v1.2.3"];
        Object.defineProperty(Bun, "argv", {
            value: mockArgs
        });

        const tag = getRequestedTag();

        expect(tag).toBe("v1.2.3");
    });

    test("argv no prefix", () => {
        const mockArgs = ["downloadMemos.ts", "--tag=1.2.3"];
        Object.defineProperty(Bun, "argv", {
            value: mockArgs
        });

        const tag = getRequestedTag();

        expect(tag).toBe("v1.2.3");
    });

    test("argv undefined", () => {
        const mockArgs = ["downloadMemos.ts"];
        Object.defineProperty(Bun, "argv", {
            value: mockArgs
        });

        const tag = getRequestedTag();

        expect(tag).toBe(null);
    });
});
