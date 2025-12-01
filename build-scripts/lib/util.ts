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

/**
 * Calculate the sha256 hex digest of a given file.
 *
 * @param filePath The path to the file.
 * @returns The sha256 hex digest of the file.
 */
export async function sha256File(filePath: string): Promise<string> {
    const file = Bun.file(filePath);
    const hasher = new Bun.CryptoHasher("sha256");
    const buffer = await file.arrayBuffer();
    hasher.update(buffer);
    return hasher.digest("hex");
}

/**
 * Convert a GOOS-GOARCH build file name to a Rust target triple.
 *
 * Sample target triples:
 *
 * - `x86_64-pc-windows-msvc`
 * - `x86_64-unknown-linux-gnu`
 * - `x86_64-apple-darwin`
 * - `aarch64-apple-darwin`
 * @param filename The file name.
 * @returns The target triple.
 */
export function makeTripletFromFileName(filename: string): string {
    const osList = ["darwin", "linux", "windows"];
    const platformMap: Record<string, string> = {
        windows: "pc",
        linux: "unknown",
        darwin: "apple"
    };
    const archMap: Record<string, string> = {
        x86_64: "x86_64",
        x64: "x86_64",
        x86: "i686",
        "386": "i686",
        arm64: "aarch64",
        aarch64: "aarch64",
        riscv64: "riscv64gc"
    };
    const variantMap: Record<string, string> = {
        windows: "msvc",
        linux: "gnu"
    };

    const os = osList.find((os) => filename.includes(os)) || "unknown";
    const platform =
        Object.entries(platformMap)
            .find(([key]) => filename.includes(key))
            ?.pop() || "unknown";
    const arch =
        Object.entries(archMap)
            .find(([key]) => filename.includes(key))
            ?.pop() || "unknown";
    const variant =
        Object.entries(variantMap)
            .find(([key]) => filename.includes(key))
            ?.pop() || "";

    const triplet = [arch, platform, os, variant].join("-");
    return triplet.endsWith("-") ? triplet.slice(0, -1) : triplet;
}

/**
 * Parse a `SHA256SUMS.txt` made by GoReleaser.
 *
 * @param sha256Sums - The sum file content.
 * @returns Map of files and their sums if valid JSON; otherwise empty object (`{}`).
 */
export async function parseSha256Sums(sha256Sums: string): Promise<Record<string, string>> {
    const lines = sha256Sums.split("\n");
    const fileHashes: Record<string, string> = {};
    for (const line of lines) {
        if (line.length === 0) {
            continue;
        }
        const elements = line.split("  ");
        const hash = elements[0].trim();
        const fileName = elements[1].trim();
        fileHashes[fileName] = hash;
    }
    return fileHashes;
}

/**
 * Convert a value to a boolean, handling various string representations.
 */
export function toBool(value: string | boolean | null | undefined): boolean {
    if (typeof value === "boolean") {
        return value;
    }
    if (value == null) {
        return false;
    }
    return ["true", "1", "yes", "y"].includes(String(value).toLowerCase());
}

/**
 * Remove duplicates and sort an array.
 */
export function uniqueSorted(array: string[]): string[] {
    return [...new Set(array)].sort();
}
