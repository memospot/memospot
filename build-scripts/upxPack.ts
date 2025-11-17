/*
 * This script runs before `Tauri bundle` step.
 */
import { findRepositoryRoot, runSync } from "./common";

export type UpxOptions = {
    bin: string;
    flags: string[];
    fileList: string[];
    supportedPlatforms?: string[];
    ignoreErrors?: boolean;
};

/**
 * Compress specified files with UPX.
 *
 * @param {UpxOptions} options The options for the UPX packer.
 * @throws {Error} An error if the command fails and `options.ignoreErrors` is false.
 */
export function upxPack(options: UpxOptions) {
    // While supported, UPX is disabled on Windows to avoid AV false-positives.
    const supportedPlatforms = options.supportedPlatforms ?? ["linux"];
    const log: string[] = [];

    const HandleError = (message: string) => {
        const error = new Error(message);
        if (options.ignoreErrors) {
            log.push(`[Ignored] Error: ${error.message}`);
            return { output: log.join("\n"), error: null };
        }
        return { output: log.join("\n"), error: error };
    };

    if (!supportedPlatforms.includes(process.platform)) {
        return HandleError(
            `\`UPX pack\` is not supported or is disabled on ${process.platform}.`
        );
    }

    const root = findRepositoryRoot();
    process.chdir(root);
    log.push(`Repository root: ${root}`);
    log.push("Running `UPX pack` …");

    if (options.fileList.length === 0) {
        return HandleError("No files to pack.");
    }

    let failed = 0;
    for (const file of options.fileList) {
        log.push(`Packing: ${file}`);

        try {
            const { stdout, stderr } = runSync(options.bin, options.flags.concat(file));
            log.push(stdout);
            log.push(stderr);
        } catch (e) {
            const message = (e as { message: string }).message || "Unknown error.";
            const error = `\`UPX pack\` failed for file \`${file}\`.\n${message}`;
            if (options.ignoreErrors) {
                HandleError(error);
                failed++;
                continue;
            }

            return HandleError(error);
        }
    }

    if (failed) {
        const total = Object.keys(options.fileList).length;
        log.push(
            `UPX failed on ${String(failed).padStart(2, "0")}/${String(total).padStart(
                2,
                "0"
            )} files.`
        );
    }

    return {
        output: log.join("\n"),
        error: null
    };
}

if (import.meta.main) {
    const repoRoot = findRepositoryRoot();
    const exe = process.platform === "win32" ? ".exe" : "";
    const filesToPack = [
        `${repoRoot}/target/release/memospot${exe}`,
        `${repoRoot}/target/x86_64-unknown-linux-gnu/release/memospot${exe}`
    ];
    const upxOptions = {
        bin: `upx${process.platform === "win32" ? ".exe" : ""}`,
        flags: ["-9"],
        fileList: filesToPack,
        supportedPlatforms: [], // Disabled due to https://github.com/tauri-apps/tauri/issues/14186.
        ignoreErrors: true
    };

    const { output, error } = upxPack(upxOptions);
    console.log(output);

    if (error) {
        console.error(error);
        process.exit(upxOptions.ignoreErrors ? 0 : 1);
    }

    process.exit(0);
}
