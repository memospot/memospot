import * as Bun from "bun";

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

    try {
        console.debug("Running command: ", fullCommand, " via Bun");
        const cmd = Bun.spawnSync([command, ...args], {
            stdout: "pipe",
            stderr: "pipe",
            cwd: cwd
        });
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
    } catch (error) {
        throw new Error(`ERROR: Failed to execute \`${fullCommand}\`: ${error}`);
    }
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
