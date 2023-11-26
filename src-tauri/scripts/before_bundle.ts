/*
 * This script runs before the tauri bundle command.
 * pnpm ts-node .\src-tauri\scripts\before_bundle.ts
 */

import { run, findRepoRoot } from "./common.ts";
import { CommandLine } from "./types";
import { platform } from "node:process";

const hooks: CommandLine = {
    "upx pack": {
        cmd: "upx",
        args: [
            "--best",
            "./src-tauri/target/release/memospot" +
                (platform === "win32" ? ".exe" : ""),
        ],
        ignoreErrors: true,
    },
};

async function main() {
    const repoRoot = await findRepoRoot();
    console.log(`Repo root: ${repoRoot}`);
    process.chdir(repoRoot);
    console.log("Running `before bundle` hooks...");

    let failed = 0;
    for (const hook of Object.keys(hooks)) {
        const current = hooks[hook];
        console.log(`Running: ${hook}`);

        const ret = await run(current.cmd, current.args);
        if (ret.code !== 0) {
            failed++;
            console.log(`Error: ${hook} failed.`);
            if (ret.stdout) {
                console.log(ret.stdout);
            }
            if (ret.stderr) {
                console.log(ret.stderr);
            }
            if (!current.ignoreErrors) {
                throw new Error(`Failed to run ${hook}`);
            }
        }
    }

    if (failed) {
        console.log(
            `Some hooks (${failed}/${
                Object.keys(hooks).length
            }) failed, but they are not required to complete on every build.`,
        );
        process.exit(0);
    }
}

main().catch((e) => {
    console.error(e);
    process.exit(1);
});
