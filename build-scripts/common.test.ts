import { assertThrows, assert, isAbsolute, osPaths } from "./deps.ts";
import { findRepositoryRoot } from "./common.ts";

Deno.test("find repository root", async (t) => {
    await t.step("validate git repository root", () => {
        const repoRoot = findRepositoryRoot();
        assert(repoRoot.endsWith("memospot"));
    });

    await t.step("validate absolute path", () => {
        const repoRoot = findRepositoryRoot();
        assert(isAbsolute(repoRoot));
    });

    await t.step("validate error on non-repo cwd", () => {
        const cwd = Deno.cwd();

        Deno.chdir(osPaths.temp()!);
        assertThrows(
            (): void => {
                findRepositoryRoot();
            },
            Error,
            "fatal: not a git repository"
        );

        Deno.chdir(cwd);
    });
});
