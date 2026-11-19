import { describe, expect, test } from "bun:test";
import {
    CROSS_COMPILE_PLATFORMS,
    getDownloadFilesGlob,
    getRequestedTag,
    getTagAlternatives,
    RUST2GO,
    SUPPORTED_BUILDS
} from "../bin/download-memos";
import { makeTripletFromFileName } from "../lib/util";

describe("match assets according to environment", async () => {
    const setEnvValue = (key: string, value: string | undefined) => {
        Object.defineProperty(process.env, key, {
            value,
            configurable: true,
            writable: true,
            enumerable: true
        });
    };

    test("dev machine", () => {
        setEnvValue("CI", "false");

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
            setEnvValue("CI", "true");

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
            setEnvValue("CI", originalCI);
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

    test("--tag does not affect glob", () => {
        // GIVEN any ambient environment shared by both calls
        // WHEN the file glob is read with and without `--tag`
        const base = getDownloadFilesGlob(["bun", "download-memos.ts"]);
        const withTag = getDownloadFilesGlob(["bun", "download-memos.ts", "--tag=26.11.1"]);

        // THEN the tag is tolerated and ignored for asset matching
        expect(withTag).toEqual(base);
    });

    test("--all with --tag returns all builds", () => {
        // GIVEN `--all` combined with `--tag`
        // WHEN the file glob is read
        const downloadFilesGlob = getDownloadFilesGlob([
            "bun",
            "download-memos.ts",
            "--all",
            "--tag=26.11.1"
        ]);

        // THEN all supported builds are matched without a parse error
        expect(downloadFilesGlob).toEqual([...SUPPORTED_BUILDS].sort());
    });
});

describe("test getRequestedTag", async () => {
    test("env prefixed", () => {
        // GIVEN a `v`-prefixed SemVer version in the environment
        // WHEN the requested tag is read
        const tag = getRequestedTag(["download-memos.ts"], "v1.2.3");

        // THEN the tag is preserved as-is for old SemVer releases
        expect(tag).toBe("v1.2.3");
    });

    test("env no prefix", () => {
        // GIVEN a bare SemVer version in the environment
        // WHEN the requested tag is read
        const tag = getRequestedTag(["download-memos.ts"], "1.2.3");

        // THEN the tag is preserved as-is and resolved via alternatives at fetch time
        expect(tag).toBe("1.2.3");
    });

    test("env CalVer no prefix", () => {
        // GIVEN a bare CalVer version in the environment
        // WHEN the requested tag is read
        const tag = getRequestedTag(["download-memos.ts"], "26.11.1");

        // THEN the tag is preserved as-is for new CalVer releases
        expect(tag).toBe("26.11.1");
    });

    test("env CalVer prefixed", () => {
        // GIVEN a `v`-prefixed CalVer version in the environment
        // WHEN the requested tag is read
        const tag = getRequestedTag(["download-memos.ts"], "v26.11.1");

        // THEN the tag is preserved as-is and resolved via alternatives at fetch time
        expect(tag).toBe("v26.11.1");
    });

    test("env undefined", () => {
        // GIVEN no version in the environment and no CLI flag
        // WHEN the requested tag is read
        const tag = getRequestedTag(["download-memos.ts"], undefined);

        // THEN no tag is requested
        expect(tag).toBe(null);
    });

    test("argv prefixed", () => {
        // GIVEN a `v`-prefixed SemVer tag on the command line
        // WHEN the requested tag is read
        const tag = getRequestedTag(["download-memos.ts", "--tag=v1.2.3"], undefined);

        // THEN the tag is preserved as-is
        expect(tag).toBe("v1.2.3");
    });

    test("argv no prefix", () => {
        // GIVEN a bare SemVer tag on the command line
        // WHEN the requested tag is read
        const tag = getRequestedTag(["download-memos.ts", "--tag=1.2.3"], undefined);

        // THEN the tag is preserved as-is
        expect(tag).toBe("1.2.3");
    });

    test("argv CalVer", () => {
        // GIVEN a CalVer tag on the command line
        // WHEN the requested tag is read
        const tag = getRequestedTag(["download-memos.ts", "--tag=26.11.1"], undefined);

        // THEN the CalVer tag is preserved as-is
        expect(tag).toBe("26.11.1");
    });

    test("argv undefined", () => {
        // GIVEN no CLI flag and no environment version
        // WHEN the requested tag is read
        const tag = getRequestedTag(["download-memos.ts"], undefined);

        // THEN no tag is requested
        expect(tag).toBe(null);
    });

    test("argv takes precedence over env", () => {
        // GIVEN both CLI and environment versions
        // WHEN the requested tag is read
        const tag = getRequestedTag(["download-memos.ts", "--tag=26.11.1"], "v0.26.2");

        // THEN the CLI flag wins
        expect(tag).toBe("26.11.1");
    });

    test("argv tolerates --all alongside --tag", () => {
        // GIVEN `--tag` combined with the documented `--all` flag
        // WHEN the requested tag is read
        const tag = getRequestedTag(["download-memos.ts", "--all", "--tag=26.11.1"], undefined);

        // THEN the tag is returned instead of throwing on the sibling flag
        expect(tag).toBe("26.11.1");
    });

    test("argv tolerates --rust-target and --cross-compile", () => {
        // GIVEN `--tag` combined with other script flags
        // WHEN the requested tag is read
        const tag = getRequestedTag(
            [
                "download-memos.ts",
                "--rust-target=x86_64-unknown-linux-gnu",
                "--cross-compile",
                "--tag=v0.26.2"
            ],
            undefined
        );

        // THEN the tag is returned instead of throwing on sibling flags
        expect(tag).toBe("v0.26.2");
    });

    test("argv with only sibling flags returns null", () => {
        // GIVEN only sibling flags and no tag
        // WHEN the requested tag is read
        const tag = getRequestedTag(["download-memos.ts", "--all"], undefined);

        // THEN no tag is requested instead of throwing
        expect(tag).toBe(null);
    });

    test("argv bare --tag throws", () => {
        // GIVEN `--tag` without a value
        // WHEN the requested tag is read
        // THEN parsing fails fast instead of silently falling back to latest
        expect(() => getRequestedTag(["download-memos.ts", "--tag"], undefined)).toThrow();
    });

    test("argv unknown flag throws", () => {
        // GIVEN a misspelled flag
        // WHEN the requested tag is read
        // THEN parsing fails fast instead of silently falling back to latest
        expect(() =>
            getRequestedTag(["download-memos.ts", "--tagg=26.11.1"], undefined)
        ).toThrow();
    });

    test("argv invalid tag value throws", () => {
        // GIVEN explicitly malformed `--tag` values
        // WHEN the requested tag is read
        // THEN a descriptive error names the bad value instead of using latest
        expect(() => getRequestedTag(["download-memos.ts", "--tag=foo"], undefined)).toThrow(
            /Invalid tag "foo"/
        );
        expect(() => getRequestedTag(["download-memos.ts", "--tag=v0.26"], undefined)).toThrow(
            /Invalid tag "v0.26"/
        );
    });

    test("argv latest resolves to latest", () => {
        // GIVEN an explicit `latest` alias via flag or environment,
        // or the CI `NOT_SET`/empty placeholder
        // WHEN the requested tag is read
        // THEN null signals the latest release without an error
        expect(getRequestedTag(["download-memos.ts", "--tag=latest"], undefined)).toBe(null);
        expect(getRequestedTag(["download-memos.ts"], "latest")).toBe(null);
        expect(getRequestedTag(["download-memos.ts", "--tag=NOT_SET"], undefined)).toBe(null);
        expect(getRequestedTag(["download-memos.ts"], "NOT_SET")).toBe(null);
        expect(getRequestedTag(["download-memos.ts"], "")).toBe(null);
        expect(getRequestedTag(["download-memos.ts", "--tag="], undefined)).toBe(null);
    });

    test("env invalid throws", () => {
        // GIVEN a malformed `MEMOS_VERSION` and no CLI flag
        // WHEN the requested tag is read
        // THEN a descriptive error names the bad value
        expect(() => getRequestedTag(["download-memos.ts"], "foo")).toThrow(
            /Invalid tag "foo"/
        );
    });
});

describe("test getTagAlternatives", async () => {
    test("SemVer with prefix tries bare alternative", () => {
        // GIVEN an old SemVer tag with `v` prefix
        // WHEN alternatives are listed
        // THEN the bare form is tried second for compatibility
        expect(getTagAlternatives("v0.26.2")).toEqual(["v0.26.2", "0.26.2"]);
    });

    test("SemVer bare tries prefixed alternative", () => {
        // GIVEN an old SemVer tag without prefix
        // WHEN alternatives are listed
        // THEN the `v`-prefixed form is tried second
        expect(getTagAlternatives("0.26.2")).toEqual(["0.26.2", "v0.26.2"]);
    });

    test("CalVer bare tries prefixed alternative", () => {
        // GIVEN a new CalVer tag without prefix
        // WHEN alternatives are listed
        // THEN the `v`-prefixed form is tried second
        expect(getTagAlternatives("26.11.1")).toEqual(["26.11.1", "v26.11.1"]);
    });

    test("CalVer prefixed tries bare alternative", () => {
        // GIVEN a CalVer tag with `v` prefix
        // WHEN alternatives are listed
        // THEN the bare form is tried second
        expect(getTagAlternatives("v26.11.1")).toEqual(["v26.11.1", "26.11.1"]);
    });
});
