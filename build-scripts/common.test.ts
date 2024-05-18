import { describe, test } from "bun:test";
import * as assert from "node:assert";
import * as os from "node:os";
import * as path from "node:path";
import { findRepositoryRoot, runSync } from "./common";

describe("runSync()", () => {
    test("validate error on invalid command", () => {
        assert.throws(
            (): void => {
                runSync("./invalid-command", []);
            },
            {
                name: "Error",
                message: /failed to execute/i
            }
        );
    });
});

describe("findRepositoryRoot()", async () => {
    test("validate git repository root", () => {
        const repoRoot = findRepositoryRoot();
        assert.ok(repoRoot.endsWith("memospot"));
    });

    test("validate absolute path", async () => {
        const repoRoot = findRepositoryRoot();
        assert.ok(path.isAbsolute(repoRoot));
    });

    test("validate error on non-repo cwd", async () => {
        assert.throws(
            (): void => {
                const cwd = os.tmpdir();
                findRepositoryRoot(cwd);
            },
            {
                name: "Error",
                message: /fatal: not a git repository/i
            }
        );
    });
});
