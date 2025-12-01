import { describe, expect, mock, test } from "bun:test";
import * as crypto from "node:crypto";
import * as os from "node:os";
import * as path from "node:path";
import { findRepositoryRoot, makeTripletFromFileName, runSync } from "../lib/util";

describe("runSync()", () => {
    test("validate error on invalid command", () => {
        expect(() => {
            runSync("/bin/invalid-command", []);
        }).toThrowError(/failed to execute/i);
    });
});

describe("findRepositoryRoot()", async () => {
    const repoRoot = findRepositoryRoot();
    test("validate git repository root", () => {
        // Repository root is "builder" when running in the Docker builder environment.
        expect(repoRoot.endsWith("memospot") || repoRoot.endsWith("builder")).toBeTrue();
    });

    test("validate absolute path", async () => {
        expect(path.isAbsolute(repoRoot)).toBeTrue();
    });

    test("validate error on non-repo cwd", async () => {
        expect(() => {
            const cwd = os.tmpdir();
            findRepositoryRoot(cwd);
        }).toThrowError(/fatal: not a git repository/i);
    });
});

test("test makeTripletFromFileName output", () => {
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
