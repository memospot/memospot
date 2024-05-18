/**
 * This script runs before `Tauri build` step.
 */

import * as crypto from "node:crypto";
import fs from "node:fs";
import * as async from "async";
import * as Bun from "bun";
import decompress from "decompress";
import { minimatch } from "minimatch";
import { findRepositoryRoot } from "./common";
import type { GitHubAsset, GitHubRelease } from "./types/downloadMemosBuildsHook";

/**
 * Convert a GOOS-GOARCH build file name to a Rust target triple.
 *
 * Sample target triples:
 *
 * - `x86_64-pc-windows-msvc`
 * - `x86_64-unknown-linux-gnu`
 * - `x86_64-apple-darwin`
 * - `aarch64-apple-darwin`
 *
 * @param file The file name.
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
 * Calculate the sha256 hex digest of a given file.
 */
export async function sha256File(filePath: string): Promise<string> {
    if (process.versions.bun) {
        const file = Bun.file(filePath);
        const hasher = new Bun.CryptoHasher("sha256");
        const buffer = await file.arrayBuffer();
        hasher.update(buffer);
        return hasher.digest("hex");
    }

    return new Promise((resolve) => {
        const hash = crypto.createHash("sha256");
        fs.createReadStream(filePath)
            .on("data", (data) => hash.update(data))
            .on("error", (err) => {
                throw err;
            })
            .on("end", () => resolve(hash.digest("hex")));
    });
}

/**
 * Parse a sum file.
 *
 * @param source - The path to the sum file. Might be a URL or local file.
 * @returns Map of files and their sums if valid JSON; otherwise empty object (`{}`).
 */
export async function parseSha256Sums(source: string): Promise<Record<string, string>> {
    let sha256Sums: string;
    // check if source is a URL or file path and handle accordingly (e.g., download the content)
    if (!source.startsWith("http") && fs.existsSync(source)) {
        if (process.versions.bun) {
            const file = Bun.file(source);
            sha256Sums = await file.text();
        } else {
            sha256Sums = fs.readFileSync(source, "utf8");
        }
    } else {
        const response = await fetch(source, { redirect: "follow", method: "GET" });
        sha256Sums = await response.text();
    }

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

export async function downloadFile(srcURL: string, dstFile: string) {
    if (process.versions.bun) {
        const response = await fetch(srcURL, { redirect: "follow" });
        await Bun.write(dstFile, response);
        return;
    }

    const Readable = (await import("node:stream")).Readable;
    const finished = (await import("node:stream/promises")).finished;

    const response = await fetch(srcURL, { redirect: "follow" });
    const stream = fs.createWriteStream(dstFile, { flags: "wx" });
    if (!response.body) {
        throw new Error("No response body");
    }
    /// biome-ignore lint/suspicious/noExplicitAny: experimental function
    await finished(Readable.fromWeb(response.body as any).pipe(stream));
}

async function downloadServerBinaries() {
    const repo = "memospot/memos-builds";
    const repoUrl = `https://api.github.com/repos/${repo}/releases/latest`;

    // Match only platforms that Memospot also supports
    const downloadFilesGlob = [
        "memos-*-darwin-arm64.tar.gz",
        "memos-*-darwin-x86_64.tar.gz",
        "memos-*-linux-x86_64.tar.gz",
        "memos-*-windows-x86_64.zip"
    ];

    // fetch data from github api
    const response = await fetch(repoUrl);
    const ghRelease = (await response.json()) as GitHubRelease;
    const releaseAssets = ghRelease.assets as GitHubAsset[];

    if (!releaseAssets || releaseAssets.length === 0) {
        throw new Error("Failed to fetch assets");
    }

    console.log(`Latest ${repo} tag: ${ghRelease.tag_name}`);

    const sha256sums = releaseAssets.find((asset) => {
        return asset.name.endsWith("SHA256SUMS.txt");
    })?.browser_download_url;
    if (!sha256sums) {
        throw new Error("Failed to find SHA256SUMS.txt");
    }

    const selectedFiles = releaseAssets.filter((asset) => {
        return downloadFilesGlob.some((mask) => minimatch(asset.name, mask));
    });

    if (selectedFiles.length === 0) {
        throw new Error("Failed to match files");
    }
    console.log(`Matched ${selectedFiles.length} files`);

    // download files in parallel
    await async
        .eachLimit(selectedFiles, 5, async (ghAsset: GitHubAsset) => {
            const fileName = ghAsset.name;
            const dstPath = `./server-dist/${fileName}`;

            if (fs.existsSync(dstPath)) {
                fs.rmSync(dstPath, { force: true, recursive: true });
            }

            console.log(`Downloading ${fileName} from ${ghAsset.browser_download_url}...`);
            await downloadFile(ghAsset.browser_download_url, dstPath);
        })
        .catch((error) => {
            throw error;
        });

    // check hashes via memos_SHA256SUMS.txt
    const fileHashes = await parseSha256Sums(sha256sums);

    await async
        .eachLimit(selectedFiles, 2, async (file: GitHubAsset) => {
            const fileName = file.name;
            console.log(`Checking hash for ${fileName}...`);

            const filePath = `./server-dist/${fileName}`;
            const fileHash = await sha256File(filePath);

            console.log(`Hash: ${fileHash}`);
            if (fileHash !== fileHashes[fileName]) {
                throw new Error(`Hash mismatch for ${fileName}`);
            }
        })
        .catch((error) => {
            throw error;
        });

    // extract files in parallel
    await async.eachLimit(selectedFiles, 2, async (file: GitHubAsset) => {
        const uuid = crypto.randomUUID();
        const extractDir = `./server-dist/${uuid}`;
        if (!fs.existsSync(extractDir)) {
            fs.mkdirSync(extractDir);
        }

        const fileName = file.name;
        const filePath = `./server-dist/${fileName}`;
        if (fileName.endsWith(".zip") || fileName.endsWith(".tar.gz")) {
            console.log(`Extracting ${fileName}...`);
            await decompress(filePath, extractDir)
                .then((files) => {
                    console.log(`Extracted ${files.length} files`);
                })
                .catch((error) => {
                    fs.rmSync(extractDir, { recursive: true });
                    throw error;
                });
        }

        const exe = fileName.includes("windows") ? ".exe" : "";

        const triplet = makeTripletFromFileName(fileName);
        fs.renameSync(`${extractDir}/memos${exe}`, `./server-dist/memos-${triplet}${exe}`);
        // chmod +x downloaded file
        if (process.platform !== "win32") {
            fs.chmodSync(`./server-dist/memos-${triplet}${exe}`, 0o755);
        }

        // check if there's a sidecar front-end folder (Memos v0.18.2 - v0.21.0)
        const sidecarDist = `${extractDir}/dist`;
        if (fs.existsSync(sidecarDist) && fs.statSync(sidecarDist).isDirectory()) {
            const frontendDist = "./server-dist/dist";
            // move front-end dist folder, only once, as it's the same for all platforms
            if (!fs.existsSync(frontendDist)) {
                fs.renameSync(sidecarDist, frontendDist);
            }
        } else {
            // create an empty directory so the Tauri build doesn't fail after Memos v0.22+ is out
            fs.mkdirSync(sidecarDist);
        }

        fs.rmSync(extractDir, { recursive: true });
        fs.rmSync(filePath);
    });
}

async function main() {
    const startTime = performance.now();

    const repoRoot = findRepositoryRoot();
    console.log(`Repository root is \`${repoRoot}\``);
    process.chdir(repoRoot);
    console.log("Running pre-build hook `Download Memos Builds` ...");

    const serverDistDir = "./server-dist";
    const serverDistDirExists =
        fs.existsSync(serverDistDir) && fs.statSync(serverDistDir).isDirectory();
    if (!serverDistDirExists) {
        fs.mkdirSync(serverDistDir, { recursive: true, mode: 0o755 });
    }

    // remove a previous dist folder, if it exists
    const distDir = "./server-dist/dist";
    if (serverDistDirExists && fs.existsSync(distDir) && fs.statSync(distDir).isDirectory()) {
        fs.rmSync(distDir, { force: true, recursive: true });
    }

    await downloadServerBinaries();

    const endTime = performance.now();
    const timeElapsed = endTime - startTime;
    console.log("Time elapsed: ", timeElapsed, "ms");
}

if (import.meta.main) {
    await main().catch((e) => {
        throw e;
    });
}
