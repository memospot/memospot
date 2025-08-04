import { describe, expect, test } from "bun:test";
import * as os from "node:os";
import * as path from "node:path";
import { findRepositoryRoot, runSync } from "./common";

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
        expect(repoRoot.endsWith("memospot")).toBeTrue();
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
