import { describe, expect, test } from "bun:test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import * as Bun from "bun";
import { getTaskStaleness, updateTaskStamp } from "../lib/taskStamps";

const CLI_PATH = path.resolve(import.meta.dir, "..", "bin", "is-stale.ts");

async function runGit(args: string[], cwd: string) {
    const env = { ...process.env };
    delete env.GIT_ALTERNATE_OBJECT_DIRECTORIES;
    delete env.GIT_CEILING_DIRECTORIES;
    delete env.GIT_COMMON_DIR;
    delete env.GIT_DIR;
    delete env.GIT_INDEX_FILE;
    delete env.GIT_OBJECT_DIRECTORY;
    delete env.GIT_WORK_TREE;

    const result = Bun.spawnSync(["git", ...args], {
        cwd,
        env,
        stderr: "pipe",
        stdout: "pipe"
    });

    if (!result.success || result.exitCode !== 0) {
        throw new Error(result.stderr.toString() || `git ${args.join(" ")} failed`);
    }
}

async function makeFixture(): Promise<string> {
    const cwd = await fs.mkdtemp(path.join(os.tmpdir(), "memospot-task-stamps-"));
    await fs.mkdir(path.join(cwd, "src-ui/src/lib"), { recursive: true });
    await fs.mkdir(path.join(cwd, "src-ui/build/assets"), { recursive: true });
    await fs.mkdir(path.join(cwd, "src-ui/build/cache"), { recursive: true });

    await Bun.write(path.join(cwd, "src-ui/src/app.html"), "<main />\n");
    await Bun.write(path.join(cwd, "src-ui/src/lib/keep.ts"), "export const keep = true;\n");
    await Bun.write(path.join(cwd, "src-ui/src/lib/ignore.generated.ts"), "ignore me\n");
    await Bun.write(path.join(cwd, "src-ui/build/index.html"), "<main />\n");
    await Bun.write(path.join(cwd, "src-ui/build/assets/app.js"), "console.log('ok');\n");
    await Bun.write(path.join(cwd, "src-ui/build/cache/nested.task"), "nested task stamp\n");

    await runGit(["init", "-q"], cwd);
    return cwd;
}

async function setMtime(targetPath: string, mtimeMs: number) {
    const time = new Date(mtimeMs);
    await fs.utimes(targetPath, time, time);
}

async function bootstrapTask(args: Parameters<typeof getTaskStaleness>[0]): Promise<void> {
    const initial = await getTaskStaleness(args);
    expect(initial).toEqual({
        isStale: true,
        stampInitialized: true
    });
    await updateTaskStamp({
        cwd: args.cwd,
        stampFile: args.stampFile
    });
}

function runCli(args: string[], cwd: string, env?: Record<string, string | undefined>) {
    const result = Bun.spawnSync([process.execPath, CLI_PATH, ...args], {
        cwd,
        env: {
            ...process.env,
            ...env
        },
        stderr: "pipe",
        stdout: "pipe"
    });

    return {
        exitCode: result.exitCode,
        stderr: result.stderr.toString(),
        stdout: result.stdout.toString()
    };
}

describe("getTaskStaleness", () => {
    test("ignores inherited git worktree env and resolves repository from cwd", async () => {
        const cwd = await makeFixture();
        const hostRepoRoot = path.resolve(import.meta.dir, "..", "..");

        const script = [
            'import { getTaskStaleness } from "./build-scripts/lib/taskStamps";',
            "const result = await getTaskStaleness({",
            "  cwd: process.env.FIXTURE_CWD,",
            '  generates: ["src-ui/build"],',
            '  sources: ["src-ui/src"],',
            '  stampFile: ".build-stamps/src-ui.task"',
            "});",
            "process.stdout.write(JSON.stringify(result));"
        ].join("\n");

        const result = Bun.spawnSync([process.execPath, "--eval", script], {
            cwd: hostRepoRoot,
            env: {
                ...process.env,
                FIXTURE_CWD: cwd,
                GIT_DIR: path.join(hostRepoRoot, ".git"),
                GIT_WORK_TREE: hostRepoRoot
            },
            stderr: "pipe",
            stdout: "pipe"
        });

        expect(result.exitCode).toBe(0);
        expect(result.stderr.toString()).toBe("");
        expect(JSON.parse(result.stdout.toString())).toEqual({
            isStale: true,
            stampInitialized: true
        });
    });

    test("returns stale and bootstraps task metadata when the stamp file is missing", async () => {
        const cwd = await makeFixture();

        const result = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build"],
            sources: ["src-ui/src"],
            stampFile: ".build-stamps/src-ui.task"
        });

        expect(result).toEqual({
            isStale: true,
            stampInitialized: true
        });
    });

    test("accepts a symlinked cwd when git reports the canonical repository path", async () => {
        const actualCwd = await makeFixture();
        const symlinkCwd = `${actualCwd}-link`;
        await fs.symlink(actualCwd, symlinkCwd, "dir");

        const result = await getTaskStaleness({
            cwd: symlinkCwd,
            generates: ["src-ui/build"],
            sources: ["src-ui/src"],
            stampFile: ".build-stamps/src-ui.task"
        });

        expect(result).toEqual({
            isStale: true,
            stampInitialized: true
        });
    });

    test("returns stale when sources change while generates still match the stored state", async () => {
        const cwd = await makeFixture();
        await bootstrapTask({
            cwd,
            generates: ["src-ui/build"],
            sources: ["src-ui/src"],
            stampFile: ".build-stamps/src-ui.task"
        });
        await Bun.write(path.join(cwd, "src-ui/src/app.html"), "<main>changed</main>\n");

        const result = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build"],
            sources: ["src-ui/src"],
            stampFile: ".build-stamps/src-ui.task"
        });

        expect(result).toEqual({
            isStale: true,
            stampInitialized: false
        });
    });

    test("returns stale when source and generated digests drift until update is called", async () => {
        const cwd = await makeFixture();
        await bootstrapTask({
            cwd,
            generates: ["src-ui/build"],
            sources: ["src-ui/src"],
            stampFile: ".build-stamps/src-ui.task"
        });

        await Bun.write(path.join(cwd, "src-ui/src/app.html"), "<main>changed</main>\n");
        await Bun.write(path.join(cwd, "src-ui/build/index.html"), "<main>rebuilt</main>\n");

        const result = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build"],
            sources: ["src-ui/src"],
            stampFile: ".build-stamps/src-ui.task"
        });

        expect(result).toEqual({
            isStale: true,
            stampInitialized: false
        });
    });

    test("updateTaskStamp syncs the stored digest using the persisted task definition", async () => {
        const cwd = await makeFixture();
        await bootstrapTask({
            cwd,
            generates: ["src-ui/build"],
            sources: ["src-ui/src"],
            stampFile: ".build-stamps/src-ui.task"
        });

        await Bun.write(path.join(cwd, "src-ui/src/app.html"), "<main>changed</main>\n");
        await Bun.write(path.join(cwd, "src-ui/build/index.html"), "<main>rebuilt</main>\n");

        const beforeUpdate = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build"],
            sources: ["src-ui/src"],
            stampFile: ".build-stamps/src-ui.task"
        });
        expect(beforeUpdate).toEqual({
            isStale: true,
            stampInitialized: false
        });

        await updateTaskStamp({
            cwd,
            stampFile: ".build-stamps/src-ui.task"
        });

        const afterUpdate = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build"],
            sources: ["src-ui/src"],
            stampFile: ".build-stamps/src-ui.task"
        });
        expect(afterUpdate).toEqual({
            isStale: false,
            stampInitialized: false
        });
    });

    test("supports negated source patterns", async () => {
        const cwd = await makeFixture();

        const result = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build/**"],
            sources: ["src-ui/src/**", "!src-ui/src/**/*.generated.ts"],
            stampFile: ".build-stamps/task.task"
        });

        expect(result).toEqual({
            isStale: true,
            stampInitialized: true
        });
    });

    test("automatically ignores the stamp file when it is nested under generated outputs", async () => {
        const cwd = await makeFixture();
        const nestedStampPath = path.join(cwd, "src-ui/build/cache/nested.task");

        await Bun.write(nestedStampPath, "updated\n");

        const result = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build/**"],
            sources: ["src-ui/src/**"],
            stampFile: "src-ui/build/cache/nested.task"
        });

        expect(result).toEqual({
            isStale: true,
            stampInitialized: true
        });
    });

    test("returns stale when generated outputs are missing", async () => {
        const cwd = await makeFixture();
        await fs.rm(path.join(cwd, "src-ui/build/index.html"));

        const result = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build/index.html"],
            sources: ["src-ui/src"],
            stampFile: ".build-stamps/src-ui.task"
        });

        expect(result).toEqual({
            isStale: true,
            stampInitialized: true
        });
    });

    test("detects source content changes even when mtime is unchanged", async () => {
        const cwd = await makeFixture();
        const sourcePath = path.join(cwd, "src-ui/src/app.html");
        const generatePath = path.join(cwd, "src-ui/build/index.html");
        const stableTime = Date.now() - 30_000;

        await setMtime(sourcePath, stableTime);
        await setMtime(generatePath, stableTime);

        await bootstrapTask({
            cwd,
            generates: ["src-ui/build/index.html"],
            sources: ["src-ui/src/app.html"],
            stampFile: ".build-stamps/src-ui.task"
        });

        await Bun.write(sourcePath, "<section>changed</section>\n");
        await setMtime(sourcePath, stableTime);

        const changedResult = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build/index.html"],
            sources: ["src-ui/src/app.html"],
            stampFile: ".build-stamps/src-ui.task"
        });

        expect(changedResult).toEqual({
            isStale: true,
            stampInitialized: false
        });
    });

    test("does not refresh the stamp file when source content changes with an older mtime", async () => {
        const cwd = await makeFixture();
        const sourcePath = path.join(cwd, "src-ui/src/app.html");
        const generatePath = path.join(cwd, "src-ui/build/index.html");
        const oldTime = Date.now() - 60_000;
        const newTime = Date.now() - 10_000;
        const stampPath = path.join(cwd, ".build-stamps", "src-ui.task");

        await setMtime(sourcePath, oldTime);
        await setMtime(generatePath, newTime);

        await bootstrapTask({
            cwd,
            generates: ["src-ui/build/index.html"],
            sources: ["src-ui/src/app.html"],
            stampFile: ".build-stamps/src-ui.task"
        });

        const stampBefore = await Bun.file(stampPath).text();

        await Bun.write(sourcePath, "<section>changed with preserved mtime</section>\n");
        await setMtime(sourcePath, oldTime);

        const changedResult = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build/index.html"],
            sources: ["src-ui/src/app.html"],
            stampFile: ".build-stamps/src-ui.task"
        });

        expect(changedResult).toEqual({
            isStale: true,
            stampInitialized: false
        });
        expect(await Bun.file(stampPath).text()).toBe(stampBefore);
    });

    test("excludes gitignored files by default", async () => {
        const cwd = await makeFixture();
        const stableTime = Date.now() - 30_000;
        const ignoredSourcePath = path.join(cwd, "src-ui/src/lib/ignore.generated.ts");

        await Bun.write(path.join(cwd, ".gitignore"), "src-ui/src/lib/ignore.generated.ts\n");
        await setMtime(path.join(cwd, "src-ui/src/app.html"), stableTime);
        await setMtime(path.join(cwd, "src-ui/src/lib/keep.ts"), stableTime);
        await setMtime(ignoredSourcePath, stableTime);
        await setMtime(path.join(cwd, "src-ui/build/index.html"), stableTime);
        await setMtime(path.join(cwd, "src-ui/build/assets/app.js"), stableTime);

        await bootstrapTask({
            cwd,
            generates: ["src-ui/build/**"],
            sources: ["src-ui/src/**"],
            stampFile: ".build-stamps/task.task"
        });

        await Bun.write(ignoredSourcePath, "changed but ignored\n");

        const afterIgnoredChange = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build/**"],
            sources: ["src-ui/src/**"],
            stampFile: ".build-stamps/task.task"
        });
        expect(afterIgnoredChange).toEqual({
            isStale: false,
            stampInitialized: false
        });
    });

    test("excludes nested .gitignore matches by default", async () => {
        const cwd = await makeFixture();
        const stableTime = Date.now() - 30_000;
        const ignoredSourcePath = path.join(cwd, "src-ui/src/lib/ignore.generated.ts");

        await Bun.write(path.join(cwd, "src-ui/src/lib/.gitignore"), "*.generated.ts\n");
        await setMtime(path.join(cwd, "src-ui/src/app.html"), stableTime);
        await setMtime(path.join(cwd, "src-ui/src/lib/.gitignore"), stableTime);
        await setMtime(path.join(cwd, "src-ui/src/lib/keep.ts"), stableTime);
        await setMtime(ignoredSourcePath, stableTime);
        await setMtime(path.join(cwd, "src-ui/build/index.html"), stableTime);
        await setMtime(path.join(cwd, "src-ui/build/assets/app.js"), stableTime);

        await bootstrapTask({
            cwd,
            generates: ["src-ui/build/**"],
            sources: ["src-ui/src/**"],
            stampFile: ".build-stamps/task.task"
        });

        await Bun.write(ignoredSourcePath, "changed but nested-ignored\n");

        const afterIgnoredChange = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build/**"],
            sources: ["src-ui/src/**"],
            stampFile: ".build-stamps/task.task"
        });
        expect(afterIgnoredChange).toEqual({
            isStale: false,
            stampInitialized: false
        });
    });

    test("supports nested .gitignore negation rules", async () => {
        const cwd = await makeFixture();
        const stableTime = Date.now() - 30_000;
        const keepSourcePath = path.join(cwd, "src-ui/src/lib/keep.ts");

        await Bun.write(path.join(cwd, "src-ui/src/lib/.gitignore"), "*.ts\n!keep.ts\n");
        await setMtime(path.join(cwd, "src-ui/src/app.html"), stableTime);
        await setMtime(path.join(cwd, "src-ui/src/lib/.gitignore"), stableTime);
        await setMtime(keepSourcePath, stableTime);
        await setMtime(path.join(cwd, "src-ui/src/lib/ignore.generated.ts"), stableTime);
        await setMtime(path.join(cwd, "src-ui/build/index.html"), stableTime);
        await setMtime(path.join(cwd, "src-ui/build/assets/app.js"), stableTime);

        await bootstrapTask({
            cwd,
            generates: ["src-ui/build/**"],
            sources: ["src-ui/src/**"],
            stampFile: ".build-stamps/task.task"
        });

        await Bun.write(keepSourcePath, "export const keep = false;\n");

        const changedAfterKeepEdit = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build/**"],
            sources: ["src-ui/src/**"],
            stampFile: ".build-stamps/task.task"
        });
        expect(changedAfterKeepEdit).toEqual({
            isStale: true,
            stampInitialized: false
        });
    });

    test("includes gitignored files with noGitignore flag", async () => {
        const cwd = await makeFixture();
        const stableTime = Date.now() - 30_000;
        const ignoredSourcePath = path.join(cwd, "src-ui/src/lib/ignore.generated.ts");

        await Bun.write(path.join(cwd, ".gitignore"), "src-ui/src/lib/ignore.generated.ts\n");
        await setMtime(path.join(cwd, "src-ui/src/app.html"), stableTime);
        await setMtime(path.join(cwd, "src-ui/src/lib/keep.ts"), stableTime);
        await setMtime(ignoredSourcePath, stableTime);
        await setMtime(path.join(cwd, "src-ui/build/index.html"), stableTime);
        await setMtime(path.join(cwd, "src-ui/build/assets/app.js"), stableTime);

        await bootstrapTask({
            cwd,
            generates: ["src-ui/build/**"],
            noGitignore: true,
            sources: ["src-ui/src/**"],
            stampFile: ".build-stamps/task.task"
        });

        await Bun.write(ignoredSourcePath, "changed and tracked by noGitignore\n");

        const afterIgnoredChange = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build/**"],
            noGitignore: true,
            sources: ["src-ui/src/**"],
            stampFile: ".build-stamps/task.task"
        });
        expect(afterIgnoredChange).toEqual({
            isStale: true,
            stampInitialized: false
        });
    });

    test("still tracks generated outputs even when they are gitignored", async () => {
        const cwd = await makeFixture();

        await Bun.write(path.join(cwd, ".gitignore"), "src-ui/build/\n");

        const initial = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build/**"],
            sources: ["src-ui/src/**"],
            stampFile: ".build-stamps/task.task"
        });
        expect(initial).toEqual({
            isStale: true,
            stampInitialized: true
        });

        await updateTaskStamp({
            cwd,
            stampFile: ".build-stamps/task.task"
        });

        const afterUpdate = await getTaskStaleness({
            cwd,
            generates: ["src-ui/build/**"],
            sources: ["src-ui/src/**"],
            stampFile: ".build-stamps/task.task"
        });
        expect(afterUpdate).toEqual({
            isStale: false,
            stampInitialized: false
        });
    });
});

describe("is-stale CLI exit codes", () => {
    test("exits 0 when the task is stale", async () => {
        const cwd = await makeFixture();

        const result = runCli(
            [
                ".build-stamps/task.task",
                "--sources",
                "src-ui/src/**",
                "--generates",
                "src-ui/build/**"
            ],
            cwd
        );

        expect(result.exitCode).toBe(0);
        expect(result.stderr).toBe("");
    });

    test("exits 1 when the task is fresh after --update", async () => {
        const cwd = await makeFixture();

        const staleResult = runCli(
            [
                ".build-stamps/task.task",
                "--sources",
                "src-ui/src/**",
                "--generates",
                "src-ui/build/**"
            ],
            cwd
        );
        expect(staleResult.exitCode).toBe(0);

        const updateResult = runCli([".build-stamps/task.task", "--update"], cwd);
        expect(updateResult.exitCode).toBe(0);

        const freshResult = runCli(
            [
                ".build-stamps/task.task",
                "--sources",
                "src-ui/src/**",
                "--generates",
                "src-ui/build/**"
            ],
            cwd
        );
        expect(freshResult.exitCode).toBe(1);
        expect(freshResult.stderr).toBe("");
    });

    test("exits 2 for unmatched positive --sources patterns with an explicit message", async () => {
        const cwd = await makeFixture();

        const result = runCli(
            [
                ".build-stamps/task.task",
                "--sources",
                "missing/**",
                "--generates",
                "src-ui/build/**"
            ],
            cwd
        );

        expect(result.exitCode).toBe(2);
        expect(result.stderr).toContain("The following --sources patterns did not match");
        expect(result.stderr).toContain("missing/**");
    });

    test("exits 2 and prints git failures explicitly", async () => {
        const cwd = await makeFixture();

        const result = runCli(
            [
                ".build-stamps/task.task",
                "--sources",
                "src-ui/src/**",
                "--generates",
                "src-ui/build/**"
            ],
            cwd,
            { PATH: "" }
        );

        expect(result.exitCode).toBe(2);
        expect(result.stderr).toContain("git");
        expect(result.stderr).toContain("Command: git");
    });
});
