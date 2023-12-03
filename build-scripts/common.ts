/**
 * Run a command synchronously and return the output.
 *
 * @param command The command to run.
 * @param args The arguments to pass to the command.
 * @returns The output of the command.
 * @throws {Error} An error if the command fails.
 */
export function runSync(command: string, args: string[]) {
    const cmd = new Deno.Command(command, {
        args: args,
        stdout: "piped",
        stderr: "piped",
    });

    const { success, code, stdout, stderr } = cmd.outputSync();
    const output = new TextDecoder().decode(stdout);
    const error = new TextDecoder().decode(stderr);

    if (!success || code !== 0 || error !== "") {
        const fullCommand = [command, ...args].join(" ");
        throw new Error(`\`${fullCommand}]\` failed with status code ${code}.\n${error}`);
    }

    return {
        success: success,
        code: code,
        stdout: output,
        stderr: error,
    };
}

/**
 * Find the root of this repository.
 *
 * @returns The path to the repository root.
 * @throws {Error} An error if the current working directory is not a git repository.
 */
export function findRepositoryRoot() {
    try {
        const cmd = runSync("git", ["rev-parse", "--show-toplevel"]);
        const { stdout } = cmd;
        return stdout.trim();
    } catch (e) {
        throw e;
    }
}
