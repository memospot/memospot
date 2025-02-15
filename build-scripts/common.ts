import * as Bun from "bun";

/**
 * Foreground color codes for console output.
 */
export const RESET = "\x1b[0m";
export const BOLD = "\x1b[1m";
export const DIM = "\x1b[2m";
export const BLACK = "\x1b[30m";
export const RED = "\x1b[31m";
export const GREEN = "\x1b[32m";
export const YELLOW = "\x1b[33m";
export const BLUE = "\x1b[34m";
export const MAGENTA = "\x1b[35m";
export const CYAN = "\x1b[36m";
export const WHITE = "\x1b[37m";
/**
 * Background color codes for console output.
 */
export const BG_BLACK = "\x1b[40m";
export const BG_RED = "\x1b[41m";
export const BG_GREEN = "\x1b[42m";
export const BG_YELLOW = "\x1b[43m";
export const BG_BLUE = "\x1b[44m";
export const BG_MAGENTA = "\x1b[45m";
export const BG_CYAN = "\x1b[46m";
export const BG_WHITE = "\x1b[47m";

/**
 * Run a command synchronously and return the output.
 *
 * @param command The command to run.
 * @param args The arguments to pass to the command.
 * @param cwd The working directory to run the command in.
 * @returns The output of the command.
 * @throws {Error} An error if the command fails.
 */
export function runSync(command: string, args: string[], cwd?: string) {
    const fullCommand = [command, ...args].join(" ");
    let cmd: Bun.SyncSubprocess<"pipe", "pipe">;
    try {
        console.debug("Running command: ", fullCommand, " via Bun");
        cmd = Bun.spawnSync([command, ...args], {
            stdout: "pipe",
            stderr: "pipe",
            cwd: cwd
        });
    } catch (error) {
        throw new Error(`ERROR: Failed to execute \`${fullCommand}\`: ${error}`);
    }
    const output = new TextDecoder().decode(cmd.stdout);
    const error = new TextDecoder().decode(cmd.stderr);
    if (!cmd.success || cmd.exitCode !== 0 || error !== "") {
        throw new Error(`Command exited with code ${cmd.exitCode}.\n${error}`);
    }
    return {
        success: cmd.success,
        code: cmd.exitCode,
        stdout: output,
        stderr: error
    };
}

/**
 * Find the root of this repository.
 *
 * @returns The path to the repository root.
 * @throws {Error} An error if the current working directory is not a git repository.
 */
export function findRepositoryRoot(cwd?: string) {
    const cmd = runSync("git", ["rev-parse", "--show-toplevel"], cwd);
    const { stdout } = cmd;
    return stdout.trim();
}
