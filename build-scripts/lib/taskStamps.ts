import * as fs from "node:fs/promises";
import * as path from "node:path";
import * as Bun from "bun";
import { findExecutable, sanitizedGitEnv } from "./command";
import { resolveExistingPath, resolveRepositoryRoot } from "./repository";
import { sha256File } from "./util";

type EntryKind = "directory" | "file";

export interface TaskStampArgs {
    cwd?: string;
    generates?: string[];
    /**
     * Skip gitignore filtering. Structural directories that never contain task
     * sources or generated outputs are still pruned from the walk.
     */
    noGitignore?: boolean;
    sources: string[];
    stampFile: string;
}

interface Entry {
    absolutePath: string;
    kind: EntryKind;
    relativePath: string;
}

interface TaskStampFile {
    generates: string[];
    generatedDigest?: string;
    noGitignore?: boolean;
    sourceDigest?: string;
    sources: string[];
    updatedAt: string;
}

export interface TaskStalenessResult {
    isStale: boolean;
    stampInitialized: boolean;
}

interface GitCommandResult {
    exitCode: number;
    stderr: string;
    stdout: string;
}

export class GitCliError extends Error {
    readonly args: string[];
    readonly cwd: string;
    readonly exitCode: number;
    readonly stderr: string;
    readonly stdout: string;

    constructor(args: string[], cwd: string, result: GitCommandResult) {
        super(`git ${args.join(" ")} failed with exit code ${result.exitCode}.`);
        this.name = "GitCliError";
        this.args = args;
        this.cwd = cwd;
        this.exitCode = result.exitCode;
        this.stderr = result.stderr;
        this.stdout = result.stdout;
    }
}

export class PatternMatchError extends Error {
    readonly optionName: "--generates" | "--sources";
    readonly patterns: string[];

    constructor(optionName: "--generates" | "--sources", patterns: string[], message?: string) {
        super(message ?? `${optionName} patterns did not match any files or directories.`);
        this.name = "PatternMatchError";
        this.optionName = optionName;
        this.patterns = patterns;
    }
}

function normalizePattern(value: string): string {
    const trimmed = value.startsWith("!") ? value.slice(1) : value;
    const normalized = path.normalize(trimmed).split(path.sep).join(path.posix.sep);
    return normalized.startsWith("./") ? normalized.slice(2) : normalized;
}

function isNegatedPattern(value: string): boolean {
    return value.startsWith("!");
}

function normalizeRelativePath(relativePath: string): string {
    return relativePath.split(path.sep).join(path.posix.sep);
}

function toRepoRelativePath(targetPath: string, repositoryRoot: string): string {
    const relativePath = normalizeRelativePath(path.relative(repositoryRoot, targetPath));
    if (relativePath === "" || relativePath === ".") {
        return ".";
    }
    if (relativePath.startsWith("../")) {
        throw new Error(`Target must stay inside the repository: ${targetPath}`);
    }
    return relativePath;
}

function runGit(
    args: string[],
    cwd: string,
    acceptedExitCodes = [0],
    stdinData?: string
): GitCommandResult {
    const env = sanitizedGitEnv();
    const gitExecutable = findExecutable("git", env);
    if (!gitExecutable) {
        const gitResult = {
            exitCode: 127,
            stderr: "git executable not found in PATH.\n",
            stdout: ""
        };

        if (!acceptedExitCodes.includes(gitResult.exitCode)) {
            throw new GitCliError(args, cwd, gitResult);
        }

        return gitResult;
    }

    const result = Bun.spawnSync([gitExecutable, ...args], {
        cwd,
        env,
        stderr: "pipe",
        stdin: stdinData === undefined ? undefined : Buffer.from(stdinData),
        stdout: "pipe"
    });

    const gitResult = {
        exitCode: result.exitCode ?? 1,
        stderr: result.stderr.toString(),
        stdout: result.stdout.toString()
    };

    if (!acceptedExitCodes.includes(gitResult.exitCode)) {
        throw new GitCliError(args, cwd, gitResult);
    }

    return gitResult;
}

// These directories are excluded from every task walk. No current task uses
// them as sources or generated outputs; revisit if that changes. Skipping them
// keeps Docker and local walks bounded.
const UNSEARCHED_DIRECTORY_NAMES = new Set([
    ".dprint",
    ".flatpak-builder",
    "node_modules",
    "server-dist",
    "target"
]);

async function walkDirectory(rootDirectory: string, relativeDirectory = ""): Promise<Entry[]> {
    const absoluteDirectory = path.join(rootDirectory, relativeDirectory);
    const entries = await fs.readdir(absoluteDirectory, { withFileTypes: true });
    const resolvedEntries = await Promise.all(
        entries.map(async (entry) => {
            if (entry.name === ".git") {
                return [];
            }

            if (entry.isDirectory() && UNSEARCHED_DIRECTORY_NAMES.has(entry.name)) {
                return [];
            }

            const relativePath = path.posix.join(relativeDirectory, entry.name);
            const absolutePath = path.join(rootDirectory, relativePath);

            if (entry.isDirectory()) {
                return [
                    {
                        absolutePath,
                        kind: "directory" as const,
                        relativePath
                    },
                    ...(await walkDirectory(rootDirectory, relativePath))
                ];
            }

            if (!entry.isFile()) {
                return [];
            }

            return [
                {
                    absolutePath,
                    kind: "file" as const,
                    relativePath
                }
            ];
        })
    );

    return resolvedEntries.flat();
}

async function getIgnoredRepoRelativePaths(
    entries: Entry[],
    repositoryRoot: string
): Promise<Set<string>> {
    if (entries.length === 0) {
        return new Set();
    }

    const repoRelativePaths = entries.map((entry) =>
        toRepoRelativePath(entry.absolutePath, repositoryRoot)
    );

    const result = runGit(
        ["check-ignore", "-z", "--stdin"],
        repositoryRoot,
        [0, 1],
        `${repoRelativePaths.join("\0")}\0`
    );

    return new Set(result.stdout.split("\0").filter(Boolean));
}

interface DockerIgnoreRule {
    directoryOnly: boolean;
    globs: Bun.Glob[];
    negated: boolean;
}

const warnedMissingDockerIgnore = new Set<string>();

async function readDockerIgnoreRules(repositoryRoot: string): Promise<DockerIgnoreRule[]> {
    let contents: string;
    try {
        contents = await fs.readFile(path.join(repositoryRoot, ".dockerignore"), "utf8");
    } catch (error) {
        if ((error as NodeJS.ErrnoException).code === "ENOENT") {
            if (!warnedMissingDockerIgnore.has(repositoryRoot)) {
                warnedMissingDockerIgnore.add(repositoryRoot);
                console.warn(
                    `No .dockerignore found at ${repositoryRoot}; task-stamp digests will not use ignore filtering.`
                );
            }
            return [];
        }
        throw error;
    }

    return contents
        .split("\n")
        .map((line) => line.trim())
        .filter((line) => line.length > 0 && !line.startsWith("#"))
        .map((line) => {
            const negated = line.startsWith("!");
            const pattern = normalizePattern(line).replace(/\/+$/, "");
            const directoryPattern = pattern.endsWith("/**") ? pattern.slice(0, -3) : undefined;
            return {
                directoryOnly: line.endsWith("/") || directoryPattern !== undefined,
                globs: [pattern, directoryPattern]
                    .filter((value): value is string => Boolean(value))
                    .map((value) => new Bun.Glob(value)),
                negated
            };
        });
}

function dockerIgnoreMatches(
    relativePath: string,
    kind: EntryKind,
    rules: DockerIgnoreRule[]
): boolean {
    const segments = relativePath.split("/");
    for (let index = 0; index < segments.length; index++) {
        const candidate = segments.slice(0, index + 1).join("/");
        const isLast = index === segments.length - 1;
        let isIgnored = false;
        for (const rule of rules) {
            if (rule.directoryOnly && isLast && kind !== "directory") {
                continue;
            }
            if (rule.globs.some((glob) => glob.match(candidate))) {
                isIgnored = !rule.negated;
            }
        }
        // Docker does not allow re-including a path under an excluded parent.
        if (isIgnored) {
            return true;
        }
    }
    return false;
}

async function getDockerIgnoredRepoRelativePaths(
    entries: Entry[],
    repositoryRoot: string
): Promise<Set<string>> {
    const rules = await readDockerIgnoreRules(repositoryRoot);
    if (rules.length === 0) {
        return new Set();
    }

    const ignored = new Set<string>();
    for (const entry of entries) {
        const relativePath = toRepoRelativePath(entry.absolutePath, repositoryRoot);
        if (dockerIgnoreMatches(relativePath, entry.kind, rules)) {
            ignored.add(relativePath);
        }
    }
    return ignored;
}

async function listWorkspaceEntries(cwd: string, noGitignore = false): Promise<Entry[]> {
    const entries = await walkDirectory(cwd);
    if (noGitignore) {
        return entries;
    }

    const { gitResolved, root } = resolveRepositoryRoot(cwd, () =>
        runGit(["rev-parse", "--show-toplevel"], cwd).stdout.trim()
    );
    const ignoredRepoRelativePaths = gitResolved
        ? await getIgnoredRepoRelativePaths(entries, root)
        : await getDockerIgnoredRepoRelativePaths(entries, root);
    return entries.filter((entry) => {
        const repoRelativePath = toRepoRelativePath(entry.absolutePath, root);
        return !ignoredRepoRelativePaths.has(repoRelativePath);
    });
}

function matchesLiteralPath(entry: Entry, pattern: string): boolean {
    return entry.relativePath === pattern || entry.relativePath.startsWith(`${pattern}/`);
}

function hasGlobMagic(pattern: string): boolean {
    return /[*?[{]/.test(pattern);
}

function matchesPattern(entry: Entry, pattern: string): boolean {
    if (!hasGlobMagic(pattern)) {
        return matchesLiteralPath(entry, pattern);
    }
    return new Bun.Glob(pattern).match(entry.relativePath);
}

function resolveEntries(entries: Entry[], patterns: string[], ignoredPaths: Set<string>) {
    const resolved = new Map<string, Entry>();
    const unmatchedPositivePatterns: string[] = [];

    for (const rawPattern of patterns) {
        const negated = isNegatedPattern(rawPattern);
        const pattern = normalizePattern(rawPattern);
        const matches = entries.filter(
            (entry) => !ignoredPaths.has(entry.relativePath) && matchesPattern(entry, pattern)
        );

        if (!negated && matches.length === 0) {
            unmatchedPositivePatterns.push(rawPattern);
        }

        for (const entry of matches) {
            if (negated) {
                resolved.delete(entry.relativePath);
            } else {
                resolved.set(entry.relativePath, entry);
            }
        }
    }

    return {
        entries: [...resolved.values()].sort((left, right) =>
            left.relativePath.localeCompare(right.relativePath)
        ),
        unmatchedPositivePatterns
    };
}

async function buildEntryDigest(entries: Entry[]): Promise<string> {
    const hasher = new Bun.CryptoHasher("sha256");
    for (const entry of entries) {
        hasher.update(entry.relativePath);
        hasher.update("\0");
        hasher.update(entry.kind);
        hasher.update("\0");

        if (entry.kind === "file") {
            hasher.update(await sha256File(entry.absolutePath));
        }

        hasher.update("\0");
    }

    return hasher.digest("hex");
}

async function readTaskStamp(stampPath: string): Promise<TaskStampFile | null> {
    try {
        const contents = await fs.readFile(stampPath, "utf8");
        return JSON.parse(contents) as TaskStampFile;
    } catch (error) {
        if ((error as NodeJS.ErrnoException).code === "ENOENT") {
            return null;
        }
        if (error instanceof SyntaxError) {
            return null;
        }
        throw error;
    }
}

async function writeTaskStampDefinition(stampPath: string, args: TaskStampArgs) {
    await fs.mkdir(path.dirname(stampPath), { recursive: true });
    await Bun.write(
        stampPath,
        `${JSON.stringify(
            {
                generates: args.generates ?? [],
                noGitignore: args.noGitignore ?? false,
                sources: args.sources,
                updatedAt: new Date().toISOString()
            },
            null,
            2
        )}\n`
    );
}

async function updateTaskStampFile(
    stampPath: string,
    args: TaskStampArgs,
    sourceDigest: string,
    generatedDigest?: string
) {
    await fs.mkdir(path.dirname(stampPath), { recursive: true });
    await Bun.write(
        stampPath,
        `${JSON.stringify(
            {
                generates: args.generates ?? [],
                generatedDigest,
                noGitignore: args.noGitignore ?? false,
                sourceDigest,
                sources: args.sources,
                updatedAt: new Date().toISOString()
            },
            null,
            2
        )}\n`
    );
}

function hasSameTaskDefinition(stampFile: TaskStampFile, args: TaskStampArgs): boolean {
    return (
        JSON.stringify(stampFile.sources) === JSON.stringify(args.sources) &&
        JSON.stringify(stampFile.generates ?? []) === JSON.stringify(args.generates ?? []) &&
        (stampFile.noGitignore ?? false) === (args.noGitignore ?? false)
    );
}

function toTaskStampArgsFromFile(
    cwd: string,
    stampFile: string,
    taskStampFile: TaskStampFile
): TaskStampArgs {
    return {
        cwd,
        generates: taskStampFile.generates,
        noGitignore: taskStampFile.noGitignore ?? false,
        sources: taskStampFile.sources,
        stampFile
    };
}

async function resolveTaskEntries(args: TaskStampArgs) {
    const cwd = resolveExistingPath(path.resolve(args.cwd ?? process.cwd()));
    const resolvedStampPath = path.resolve(cwd, args.stampFile);
    const ignoredPaths = new Set([
        normalizeRelativePath(path.relative(cwd, resolvedStampPath))
    ]);
    const sourceEntries = await listWorkspaceEntries(cwd, args.noGitignore ?? false);
    const generatedEntries =
        (args.noGitignore ?? false) ? sourceEntries : await listWorkspaceEntries(cwd, true);

    const sourceResolution = resolveEntries(sourceEntries, args.sources, ignoredPaths);
    if (sourceResolution.unmatchedPositivePatterns.length > 0) {
        throw new PatternMatchError(
            "--sources",
            sourceResolution.unmatchedPositivePatterns,
            `The following --sources patterns did not match any files or directories: ${sourceResolution.unmatchedPositivePatterns.join(", ")}`
        );
    }

    if (sourceResolution.entries.length === 0) {
        throw new PatternMatchError(
            "--sources",
            args.sources,
            "The provided --sources patterns resolved to an empty task input set."
        );
    }

    const generatedResolution =
        args.generates && args.generates.length > 0
            ? resolveEntries(generatedEntries, args.generates, ignoredPaths)
            : undefined;

    return {
        cwd,
        generatedResolution,
        resolvedStampPath,
        sourceResolution
    };
}

export async function updateTaskStamp(
    args: Pick<TaskStampArgs, "cwd" | "stampFile">
): Promise<void> {
    const cwd = path.resolve(args.cwd ?? process.cwd());
    const resolvedStampPath = path.resolve(cwd, args.stampFile);
    const taskStampFile = await readTaskStamp(resolvedStampPath);

    if (!taskStampFile) {
        throw new Error(
            "Cannot update the task stamp because the stamp file does not exist or is invalid."
        );
    }

    const taskArgs = toTaskStampArgsFromFile(cwd, args.stampFile, taskStampFile);
    const { generatedResolution, sourceResolution } = await resolveTaskEntries(taskArgs);
    const sourceDigest = await buildEntryDigest(sourceResolution.entries);

    if (!taskArgs.generates || taskArgs.generates.length === 0) {
        await updateTaskStampFile(resolvedStampPath, taskArgs, sourceDigest);
        return;
    }

    const hasMissingGenerates =
        !generatedResolution ||
        generatedResolution.unmatchedPositivePatterns.length > 0 ||
        generatedResolution.entries.length === 0;
    if (hasMissingGenerates) {
        const unmatchedPatterns = generatedResolution?.unmatchedPositivePatterns ?? [];
        throw new PatternMatchError(
            "--generates",
            unmatchedPatterns,
            unmatchedPatterns.length > 0
                ? `The following --generates patterns did not match any files or directories: ${unmatchedPatterns.join(", ")}`
                : "Cannot update the task stamp because the generated outputs are missing."
        );
    }

    const generatedDigest = await buildEntryDigest(generatedResolution.entries);
    await updateTaskStampFile(resolvedStampPath, taskArgs, sourceDigest, generatedDigest);
}

export async function getTaskStaleness(args: TaskStampArgs): Promise<TaskStalenessResult> {
    const { generatedResolution, resolvedStampPath, sourceResolution } =
        await resolveTaskEntries(args);
    const taskStampFile = await readTaskStamp(resolvedStampPath);

    if (!taskStampFile || !hasSameTaskDefinition(taskStampFile, args)) {
        await writeTaskStampDefinition(resolvedStampPath, args);
        return {
            isStale: true,
            stampInitialized: true
        };
    }

    const sourceDigest = await buildEntryDigest(sourceResolution.entries);

    if (!args.generates || args.generates.length === 0) {
        return {
            isStale: taskStampFile.sourceDigest !== sourceDigest,
            stampInitialized: false
        };
    }

    const hasMissingGenerates =
        !generatedResolution ||
        generatedResolution.unmatchedPositivePatterns.length > 0 ||
        generatedResolution.entries.length === 0;

    if (hasMissingGenerates) {
        return { isStale: true, stampInitialized: false };
    }

    const generatedDigest = await buildEntryDigest(generatedResolution.entries);
    return {
        isStale:
            taskStampFile.sourceDigest !== sourceDigest ||
            taskStampFile.generatedDigest !== generatedDigest,
        stampInitialized: false
    };
}
