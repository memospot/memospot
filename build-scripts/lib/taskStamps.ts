import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import * as Bun from "bun";
import { $ } from "bun";
import { sha256File } from "./util";

type EntryKind = "directory" | "file";

export interface TaskStampArgs {
    cwd?: string;
    generates?: string[];
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

async function runGit(
    args: string[],
    cwd: string,
    acceptedExitCodes = [0],
    stdinFilePath?: string
): Promise<GitCommandResult> {
    const command = stdinFilePath
        ? $`git ${args} < ${stdinFilePath}`.cwd(cwd)
        : $`git ${args}`.cwd(cwd);
    const result = await command.nothrow().quiet();
    const gitResult = {
        exitCode: result.exitCode,
        stderr: result.stderr.toString(),
        stdout: result.stdout.toString()
    };

    if (!acceptedExitCodes.includes(gitResult.exitCode)) {
        throw new GitCliError(args, cwd, gitResult);
    }

    return gitResult;
}

async function findRepositoryRoot(cwd: string): Promise<string> {
    const result = await runGit(["rev-parse", "--show-toplevel"], cwd);
    return result.stdout.trim();
}

async function walkDirectory(rootDirectory: string, relativeDirectory = ""): Promise<Entry[]> {
    const absoluteDirectory = path.join(rootDirectory, relativeDirectory);
    const entries = await fs.readdir(absoluteDirectory, { withFileTypes: true });
    const resolvedEntries = await Promise.all(
        entries.map(async (entry) => {
            if (entry.name === ".git") {
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

    const tempDirectory = await fs.mkdtemp(path.join(os.tmpdir(), "memospot-is-stale-"));
    const stdinFilePath = path.join(tempDirectory, "paths.txt");
    const repoRelativePaths = entries.map((entry) =>
        toRepoRelativePath(entry.absolutePath, repositoryRoot)
    );

    try {
        await fs.writeFile(stdinFilePath, `${repoRelativePaths.join("\0")}\0`);
        const result = await runGit(
            ["check-ignore", "-z", "--stdin"],
            repositoryRoot,
            [0, 1],
            stdinFilePath
        );

        return new Set(result.stdout.split("\0").filter(Boolean));
    } finally {
        await fs.rm(tempDirectory, { force: true, recursive: true });
    }
}

async function listWorkspaceEntries(cwd: string, noGitignore = false): Promise<Entry[]> {
    const entries = await walkDirectory(cwd);
    if (noGitignore) {
        return entries;
    }

    const repositoryRoot = await findRepositoryRoot(cwd);
    const ignoredRepoRelativePaths = await getIgnoredRepoRelativePaths(entries, repositoryRoot);
    return entries.filter((entry) => {
        const repoRelativePath = toRepoRelativePath(entry.absolutePath, repositoryRoot);
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
    const cwd = path.resolve(args.cwd ?? process.cwd());
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
