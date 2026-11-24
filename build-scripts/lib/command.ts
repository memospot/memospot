import * as path from "node:path";
import * as Bun from "bun";

function isGitCommand(command: string): boolean {
    const name = path
        .basename(command)
        .toLowerCase()
        .replace(/\.(exe|cmd|bat)$/, "");
    return name === "git";
}

/**
 * Copy the process environment without inherited git overrides.
 */
export function sanitizedGitEnv(): NodeJS.ProcessEnv {
    const env = { ...process.env };
    delete env.GIT_ALTERNATE_OBJECT_DIRECTORIES;
    delete env.GIT_CEILING_DIRECTORIES;
    delete env.GIT_COMMON_DIR;
    delete env.GIT_DIR;
    delete env.GIT_INDEX_FILE;
    delete env.GIT_OBJECT_DIRECTORY;
    delete env.GIT_WORK_TREE;
    return env;
}

/**
 * A subprocess that exited unsuccessfully or could not be resolved.
 *
 * Carries the exit code so callers can distinguish recoverable git states,
 * such as a linked worktree whose gitdir pointer is unavailable inside a
 * build container, from missing executables or other failures.
 */
export class CommandError extends Error {
    readonly command: string;
    readonly exitCode: number;
    readonly stdout: string;
    readonly stderr: string;

    constructor(command: string, exitCode: number, stdout: string, stderr: string) {
        super(`Command exited with code ${exitCode}.\n${stderr}`);
        this.name = "CommandError";
        this.command = command;
        this.exitCode = exitCode;
        this.stdout = stdout;
        this.stderr = stderr;
    }
}

/**
 * Resolve an executable using the environment's PATH, honoring the Windows
 * `Path` casing and falling back to Bun's default lookup when neither exists.
 */
export function findExecutable(command: string, env: NodeJS.ProcessEnv): string | null {
    const PATH = env.PATH ?? env.Path;
    return PATH === undefined ? Bun.which(command) : Bun.which(command, { PATH });
}

/**
 * Run a command synchronously and return the output.
 *
 * @param command The command to run.
 * @param args The arguments to pass to the command.
 * A zero exit status is success even if the command writes to stderr.
 * Git commands are detected by executable basename and run without inherited
 * `GIT_*` overrides, including when the caller passes an absolute path.
 *
 * @param cwd The working directory to run the command in.
 * @returns The output of the command.
 * @throws {Error} When the subprocess cannot be spawned.
 * @throws {CommandError} When the executable cannot be found (127) or the command exits unsuccessfully.
 */
export function runSync(command: string, args: string[], cwd?: string) {
    const fullCommand = [command, ...args].join(" ");
    const env = isGitCommand(command) ? sanitizedGitEnv() : process.env;
    const executable = findExecutable(command, env);
    if (!executable) {
        throw new CommandError(
            fullCommand,
            127,
            "",
            `Failed to execute \`${fullCommand}\`: command not found\n`
        );
    }

    let cmd: Bun.SyncSubprocess<"pipe", "pipe">;
    try {
        console.debug("Running command: ", fullCommand, " via Bun");
        cmd = Bun.spawnSync([executable, ...args], {
            stdout: "pipe",
            stderr: "pipe",
            cwd: cwd,
            env
        });
    } catch (error) {
        throw new Error(`ERROR: Failed to execute \`${fullCommand}\`: ${error}`);
    }
    const output = new TextDecoder().decode(cmd.stdout);
    const error = new TextDecoder().decode(cmd.stderr);
    const exitCode = cmd.exitCode ?? 1;
    if (!cmd.success || exitCode !== 0) {
        throw new CommandError(fullCommand, exitCode, output, error);
    }
    return {
        success: cmd.success,
        code: exitCode,
        stdout: output,
        stderr: error
    };
}
