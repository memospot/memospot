import { assertThrows, assert } from "https://deno.land/std@0.207.0/assert/mod.ts";
import { isAbsolute } from "https://deno.land/std@0.208.0/path/mod.ts";
import osPaths from "https://deno.land/x/os_paths@v7.4.0/src/mod.deno.ts";
import { findRepositoryRoot } from "./common.ts";

Deno.test("find repository root", async (t) => {
    await t.step("validate git repository root", async () => {
        const repoRoot = findRepositoryRoot();
        assert(repoRoot.endsWith("memospot"));
    });

    await t.step("validate absolute path", async () => {
        const repoRoot = findRepositoryRoot();
        assert(isAbsolute(repoRoot));
    });

    await t.step("validate error on non-repo cwd", async () => {
        const cwd = Deno.cwd();

        Deno.chdir(osPaths.temp()!);
        assertThrows(
            (): void => {
                findRepositoryRoot();
            },
            Error,
            "fatal: not a git repository",
        );

        Deno.chdir(cwd);
    });
});
