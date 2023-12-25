/*
 * This script runs before `Tauri bundle` step.
 * deno run -A ./build-scripts/upxPackHook.ts
 */

import { findRepositoryRoot, runSync } from "./common.ts";
import type { UpxOptions } from "./upxPackHook.d.ts";

const filesToPack = [
    "./src-tauri/target/release/memospot" + (Deno.build.os === "windows" ? ".exe" : ""),
    "./server-dist/memos-x86_64-unknown-linux-gnu",
];

const upxOptions: UpxOptions = {
    bin: "upx" + (Deno.build.os === "windows" ? ".exe" : ""),
    flags: ["--best"],
    fileList: filesToPack,
    ignoreErrors: true,
};

/*
 * Compress specified files with UPX.
 *
 * @param options The options for the UPX packer.
 * @throws {Error} An error if the command fails and `options.ignoreErrors` is false.
 */
export function upxPackHook(options: UpxOptions) {
    const supportedPlatforms = options.supportedPlatforms ?? ["windows", "linux"];
    const log: string[] = [];

    const HandleError = (message: string) => {
        const error = new Error(message);
        if (options.ignoreErrors) {
            log.push("[Ignored] Error: " + error.message);
            return { output: log.join("\n"), error: null };
        } else {
            return { output: log.join("\n"), error: error };
        }
    };

    if (!supportedPlatforms.includes(Deno.build.os)) {
        return HandleError(`\`UPX pack\` is not supported on ${Deno.build.os}.`);
    }

    const root = findRepositoryRoot();
    Deno.chdir(root);
    log.push(`Repository root: ${root}`);
    log.push("Running `UPX pack` ...");

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
            const error = `\`UPX pack\` failed for file \`${file}\`.\n${e.message}`;
            if (options.ignoreErrors) {
                HandleError(error);
                failed++;
                continue;
            }

            return HandleError(error);
        }
    }

    if (failed) {
        const total = Object.keys(filesToPack).length;
        log.push(
            `UPX failed on ${String(failed).padStart(2, "0")}/${String(total).padStart(
                2,
                "0"
            )} files.`
        );
    }

    return {
        output: log.join("\n"),
        error: null,
    };
}

if (import.meta.main) {
    const { output, error } = upxPackHook(upxOptions);
    console.log(output);

    if (error) {
        console.error(error);
        Deno.exit(upxOptions.ignoreErrors ? 0 : 1);
    }

    Deno.exit(0);
}
