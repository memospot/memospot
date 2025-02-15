import { describe, expect, mock, test } from "bun:test";
import * as crypto from "node:crypto";
import { getDownloadFilesGlob, makeTripletFromFileName, rustToGoMap } from "./downloadMemos";

test("extensively test makeTripletFromFileName output", () => {
    const goToRustMap: Record<string, string> = {
        "darwin-arm64": "aarch64-apple-darwin",
        "darwin-x86_64": "x86_64-apple-darwin",
        "darwin-x86_64_v2": "x86_64-apple-darwin",
        "darwin-x86_64_v3": "x86_64-apple-darwin",
        "windows-x86_64": "x86_64-pc-windows-msvc",
        "windows-x86_64_v2": "x86_64-pc-windows-msvc",
        "windows-x86_64_v3": "x86_64-pc-windows-msvc",
        "windows-arm64": "aarch64-pc-windows-msvc",
        "linux-x86_64": "x86_64-unknown-linux-gnu",
        "linux-x86_64_v2": "x86_64-unknown-linux-gnu",
        "linux-x86_64_v3": "x86_64-unknown-linux-gnu",
        "linux-riscv64": "riscv64gc-unknown-linux-gnu",
        "linux-i386": "i686-unknown-linux-gnu",
        "linux-arm64": "aarch64-unknown-linux-gnu",
        "dummyos-arm64": "aarch64-unknown-unknown",
        "dummyos-x86_64": "x86_64-unknown-unknown"
    };

    const random = mock(() => crypto.randomUUID());
    for (const [key, value] of Object.entries(goToRustMap)) {
        const prefix = random();
        const goOsGoArch = [prefix, value].join("-");
        const rustTriplet = [prefix, makeTripletFromFileName(key)].join("-");
        expect(goOsGoArch).toBe(rustTriplet);
    }
});

describe("match assets according to environment", async () => {
    test("dev machine", () => {
        process.env.CI = "";

        const mockArchitectures = ["arm64", "x64"];
        const mockPlatforms = ["darwin", "win32", "linux"];
        for (const arch of mockArchitectures) {
            for (const platform of mockPlatforms) {
                Object.defineProperty(process, "arch", {
                    value: arch
                });
                Object.defineProperty(process, "platform", {
                    value: platform
                });

                const downloadFilesGlob = getDownloadFilesGlob();
                let expected: string[] = [];
                switch (arch) {
                    case "arm64":
                        switch (platform) {
                            case "darwin":
                                expected = [
                                    "memos-*-darwin-arm64.tar.gz",
                                    "memos-*-darwin-x86_64.tar.gz",
                                    "memos-*-windows-x86_64.zip"
                                ];
                                break;
                            case "win32":
                                expected = ["memos-*-windows-x86_64.zip"];
                                break;
                            case "linux":
                                expected = [
                                    "memos-*-linux-x86_64.tar.gz",
                                    "memos-*-windows-x86_64.zip"
                                ];
                                break;
                        }
                        break;
                    case "x64":
                        switch (platform) {
                            case "darwin":
                                expected = [
                                    "memos-*-darwin-arm64.tar.gz",
                                    "memos-*-darwin-x86_64.tar.gz",
                                    "memos-*-windows-x86_64.zip"
                                ];
                                break;
                            case "win32":
                                expected = ["memos-*-windows-x86_64.zip"];
                                break;
                            case "linux":
                                expected = [
                                    "memos-*-linux-x86_64.tar.gz",
                                    "memos-*-windows-x86_64.zip"
                                ];
                                break;
                        }
                        break;
                }
                console.log(
                    `\x1b[32mExpected: ${expected.toSorted()}, \nActual:   ${downloadFilesGlob.toSorted()}.\x1b[0m`
                );
                expect(downloadFilesGlob).toEqual(expected);
            }
        }
    });

    test("CI=true", () => {
        process.env.CI = "true";

        const mockArchitectures = ["arm64", "x64"];
        const mockPlatforms = ["darwin", "win32", "linux"];
        for (const arch of mockArchitectures) {
            for (const platform of mockPlatforms) {
                Object.defineProperty(process, "arch", {
                    value: arch
                });
                Object.defineProperty(process, "platform", {
                    value: platform
                });

                const downloadFilesGlob = getDownloadFilesGlob();
                let expected: string[] = [];
                switch (arch) {
                    case "arm64":
                        switch (platform) {
                            case "darwin":
                                expected = [
                                    "memos-*-darwin-arm64.tar.gz",
                                    "memos-*-darwin-x86_64.tar.gz"
                                ];
                                break;
                            case "win32":
                                expected = [];
                                break;
                            case "linux":
                                expected = [];
                                break;
                        }
                        break;
                    case "x64":
                        switch (platform) {
                            case "darwin":
                                expected = [
                                    "memos-*-darwin-arm64.tar.gz",
                                    "memos-*-darwin-x86_64.tar.gz"
                                ];
                                break;
                            case "win32":
                                expected = ["memos-*-windows-x86_64.zip"];
                                break;
                            case "linux":
                                expected = ["memos-*-linux-x86_64.tar.gz"];
                                break;
                        }
                }
                expect(downloadFilesGlob).toEqual(expected);
            }
        }
    });

    test("CI=true with RUST_TARGET env", () => {
        process.env.CI = "true";

        const rust_targets = Object.keys(rustToGoMap);
        for (const target of rust_targets) {
            process.env.RUST_TARGET = target;

            const downloadFilesGlob = getDownloadFilesGlob();
            const expected = [rustToGoMap[target]];

            expect(downloadFilesGlob).toEqual(expected);
        }
    });
});
