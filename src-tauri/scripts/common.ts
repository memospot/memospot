import { SpawnSyncReturns, spawnSync } from "node:child_process";
import { resolve } from "node:path";

export async function run(cmd: string, args: string[]) {
    let status = 1;
    let stdout = "";
    let stderr = "";
    try {
        const child = spawnSync(cmd, args, {
            stdio: "pipe",
        });

        status = child.status ?? 1;
        stdout = child.stdout?.toString() ?? "";
        stderr = child.stderr?.toString() ?? child.error ?? "";
    } catch (error: any) {
        const e = error as SpawnSyncReturns<Buffer>;
        status = e.status ?? 1;
        stdout = e.stdout?.toString() ?? "";
        stderr = e.stderr?.toString() ?? "";
    }
    return { code: status, stdout: stdout, stderr: stderr };
}

export async function findRepoRoot() {
    const ret = await run("git", ["rev-parse", "--show-toplevel"]);

    if (ret.code !== 0) {
        console.log(ret.stdout);
        throw new Error("Failed to find repo root");
    }

    const repoRoot = ret.stdout.trim();
    return repoRoot;
}
