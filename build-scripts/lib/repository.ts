import * as fs from "node:fs";
import * as path from "node:path";
import { runSync } from "./command";

const REPOSITORY_ROOT_MARKERS = [
    ".git",
    ".gitattributes",
    "Cargo.toml",
    "package.json"
] as const;

const GIT_METADATA_UNUSABLE_EXIT_CODE = 128;

export interface RepositoryRoot {
    gitResolved: boolean;
    root: string;
}

const realpathSync = fs.realpathSync.native ?? fs.realpathSync;

/**
 * Resolve `target` to a canonical path, preserving the lexical path when it no
 * longer exists.
 *
 * The native resolver expands Windows 8.3 aliases, so a short temp path and a
 * long git-reported root compare as the same repository.
 */
export function resolveExistingPath(target: string): string {
    try {
        return realpathSync(target);
    } catch (error) {
        if ((error as NodeJS.ErrnoException).code === "ENOENT") {
            return path.resolve(target);
        }
        throw error;
    }
}

/**
 * Find the repository root by walking up from `cwd` until the repository
 * markers are present.
 *
 * The `.git` and `.gitattributes` markers are required so this fallback applies
 * only to checkouts whose VCS metadata exists but cannot be resolved, such as a
 * linked worktree copied into a Docker build context. The fallback extends the
 * Rust workspace resolver's `.git` and `.gitattributes` requirements with
 * `Cargo.toml` and `package.json`, so it is stricter than the Rust check.
 * VCS-less source trees remain unsupported.
 */
function findRepositoryRootByMarkers(cwd: string): string | undefined {
    for (let current = path.resolve(cwd); ; current = path.dirname(current)) {
        if (
            REPOSITORY_ROOT_MARKERS.every((marker) => fs.existsSync(path.join(current, marker)))
        ) {
            return resolveExistingPath(current);
        }
        if (path.dirname(current) === current) {
            return undefined;
        }
    }
}

const warnedGitFallbacks = new Set<string>();

function warnGitMetadataUnavailable(cwd: string, root: string): void {
    const key = `${resolveExistingPath(cwd)}\0${root}`;
    if (warnedGitFallbacks.has(key)) {
        return;
    }
    warnedGitFallbacks.add(key);
    console.warn(`Git metadata is unavailable at ${cwd}; using repository markers at ${root}.`);
}

function isGitMetadataUnusable(error: unknown): boolean {
    return (
        error instanceof Error &&
        "exitCode" in error &&
        (error as { exitCode?: unknown }).exitCode === GIT_METADATA_UNUSABLE_EXIT_CODE
    );
}

/**
 * Resolve the repository root from an injected git lookup.
 *
 * Falls back to repository markers only when git reports unusable metadata
 * (exit code 128). A missing git executable or any other failure is propagated.
 *
 * @param cwd The directory to start from.
 * @param getGitRoot Returns the git-reported root, or throws an error carrying
 * an `exitCode` when git cannot resolve the repository.
 * @returns The canonical root and whether git resolved it.
 */
export function resolveRepositoryRoot(cwd: string, getGitRoot: () => string): RepositoryRoot {
    try {
        return { gitResolved: true, root: resolveExistingPath(getGitRoot()) };
    } catch (error) {
        if (!isGitMetadataUnusable(error)) {
            throw error;
        }
        const markerRoot = findRepositoryRootByMarkers(cwd);
        if (!markerRoot) {
            throw error;
        }
        warnGitMetadataUnavailable(cwd, markerRoot);
        return { gitResolved: false, root: markerRoot };
    }
}

/**
 * Find the root of this repository.
 *
 * Prefers git, and falls back to repository markers only when git reports that
 * the repository metadata is unusable (exit code 128). A missing git executable
 * or any other failure is propagated.
 *
 * @param cwd The directory to start from. Defaults to the current directory.
 * @returns The canonical path to the repository root.
 * @throws {CommandError} When git exits with a code other than 128, or with 128
 * when no marker root exists.
 * @throws {Error} When git cannot be spawned or a resolved path cannot be canonicalized.
 */
export function findRepositoryRoot(cwd?: string): string {
    const startDirectory = cwd ?? process.cwd();
    return resolveRepositoryRoot(startDirectory, () =>
        runSync("git", ["rev-parse", "--show-toplevel"], startDirectory).stdout.trim()
    ).root;
}
