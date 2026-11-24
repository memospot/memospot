import { describe, expect, mock, test } from "bun:test";
import * as crypto from "node:crypto";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { pathToFileURL } from "node:url";
import * as Bun from "bun";
import { CommandError, runSync } from "../lib/command";
import { findRepositoryRoot } from "../lib/repository";
import { makeTripletFromFileName } from "../lib/util";

describe("runSync()", () => {
    test("validate error on invalid command", () => {
        expect(() => {
            runSync("/bin/invalid-command", []);
        }).toThrowError(/failed to execute/i);
    });
});

function makeRepositoryFixture(): string {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "memospot-repository-"));
    fs.writeFileSync(path.join(root, ".git"), `gitdir: ${path.join(root, "missing-gitdir")}\n`);
    fs.writeFileSync(path.join(root, ".gitattributes"), "");
    fs.writeFileSync(path.join(root, "Cargo.toml"), "");
    fs.writeFileSync(path.join(root, "package.json"), "{}");
    return root;
}

describe("findRepositoryRoot()", async () => {
    const repoRoot = findRepositoryRoot();
    test("validate git repository root", () => {
        // GIVEN a clone, worktree, or Docker checkout under any directory name
        // THEN the root is identified by repository markers
        expect(fs.existsSync(path.join(repoRoot, ".git"))).toBeTrue();
        expect(fs.existsSync(path.join(repoRoot, "Cargo.toml"))).toBeTrue();
        expect(fs.existsSync(path.join(repoRoot, "build-scripts/lib/util.ts"))).toBeTrue();
        const pkg = JSON.parse(fs.readFileSync(path.join(repoRoot, "package.json"), "utf8"));
        expect(pkg.name).toBe("memospot");
    });

    test("validate absolute path", async () => {
        expect(path.isAbsolute(repoRoot)).toBeTrue();
    });

    test("rejects a broken git file without repository markers", () => {
        // GIVEN a directory with a broken .git pointer and no marker files
        const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "memospot-non-repo-"));
        fs.writeFileSync(
            path.join(cwd, ".git"),
            `gitdir: ${path.join(cwd, "missing-gitdir")}\n`
        );
        try {
            // WHEN resolving the repository root
            let caught: unknown;
            try {
                findRepositoryRoot(cwd);
            } catch (error) {
                caught = error;
            }

            // THEN the original 128 failure is rethrown
            expect(caught).toBeInstanceOf(CommandError);
            expect((caught as CommandError).exitCode).toBe(128);
        } finally {
            fs.rmSync(cwd, { recursive: true, force: true });
        }
    });

    test("rejects repository markers without git metadata", () => {
        // GIVEN a directory with markers but no .git entry
        const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "memospot-vcs-less-"));
        fs.writeFileSync(path.join(cwd, ".gitattributes"), "");
        fs.writeFileSync(path.join(cwd, "Cargo.toml"), "");
        fs.writeFileSync(path.join(cwd, "package.json"), "{}");
        try {
            // WHEN resolving the repository root
            let caught: unknown;
            try {
                findRepositoryRoot(cwd);
            } catch (error) {
                caught = error;
            }
            // THEN the original git failure is rethrown
            expect(caught).toBeInstanceOf(CommandError);
            expect((caught as CommandError).exitCode).toBe(128);
        } finally {
            fs.rmSync(cwd, { recursive: true, force: true });
        }
    });

    test("keeps a missing git executable fatal", () => {
        // GIVEN a repository with a broken gitdir pointer and no git in PATH
        const root = makeRepositoryFixture();
        const repositoryPath = path.resolve(import.meta.dir, "..", "lib", "repository.ts");
        const script = [
            `import { findRepositoryRoot } from ${JSON.stringify(
                pathToFileURL(repositoryPath).href
            )};`,
            `try { findRepositoryRoot(${JSON.stringify(root)}); process.exit(0); }`,
            "catch (error) {",
            "  console.error(error instanceof Error ? error.message : String(error));",
            "  process.exit(2);",
            "}"
        ].join("\n");
        const env = { ...process.env, PATH: "" };
        try {
            // WHEN git cannot be spawned
            const result = Bun.spawnSync([process.execPath, "--eval", script], {
                env,
                stderr: "pipe",
                stdout: "pipe"
            });
            // THEN the spawn failure is not swallowed by the marker fallback
            expect(result.exitCode).toBe(2);
            expect(result.stderr.toString()).toContain("Failed to execute");
        } finally {
            fs.rmSync(root, { recursive: true, force: true });
        }
    });

    test("resolves a marker root when the worktree gitdir pointer is unavailable", () => {
        // GIVEN a checkout whose .git file points outside the available tree
        const root = makeRepositoryFixture();
        const deep = path.join(root, "src", "deep");
        fs.mkdirSync(deep, { recursive: true });
        try {
            // WHEN resolving from a nested directory
            const resolved = findRepositoryRoot(deep);
            // THEN the marker root is returned, not the broken gitdir target
            expect(resolved).toBe(fs.realpathSync(root));
        } finally {
            fs.rmSync(root, { recursive: true, force: true });
        }
    });

    test("canonicalizes a symlinked marker root", () => {
        // GIVEN a repository reached through a symlink
        const root = makeRepositoryFixture();
        const link = `${root}-link`;
        try {
            fs.symlinkSync(root, link, process.platform === "win32" ? "junction" : "dir");
            // WHEN resolving from the symlinked path
            const resolved = findRepositoryRoot(link);
            // THEN the canonical marker root is returned
            expect(resolved).toBe(fs.realpathSync(root));
        } finally {
            fs.rmSync(link, { recursive: true, force: true });
            fs.rmSync(root, { recursive: true, force: true });
        }
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
