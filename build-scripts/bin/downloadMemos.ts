/**
 * This script runs before `Tauri build` step.
 *
 * Downloads Memos binaries from GitHub and unpacks them into the `./server-dist`
 * directory using the naming convention required by Tauri for bundling.
 *
 * - Will download the latest release if `--tag` is not
 *   specified or `MEMOS_VERSION` env var is not set.
 * - Skips downloading if current files are up-to-date.
 * - Uses GITHUB_TOKEN for higher rate limit.
 * - Retries with backoff on rate limit errors.
 *
 * Usage:
 *  bun run ./build-scripts/bin/downloadMemos.ts [--all]
 */

import fs from "node:fs";
import { parseArgs } from "node:util";
import * as async from "async";
import * as Bun from "bun";
import decompress from "decompress";
import { minimatch } from "minimatch";
import {
    downloadFileWithRateLimit,
    fetchWithRateLimit,
    getLatestReleaseTag
} from "../lib/github";
import {
    BG_WHITE,
    BLACK,
    BLUE,
    CYAN,
    GREEN,
    MAGENTA,
    RED,
    RESET,
    YELLOW
} from "../lib/terminal";
import {
    findRepositoryRoot,
    makeTripletFromFileName,
    parseSha256Sums,
    sha256File,
    toBool,
    uniqueSorted
} from "../lib/util";
import type { GitHubAsset, GitHubRelease } from "../types/github";

export const RELEASE_REPOSITORY = "memospot/memos-builds" as const;

/**
 * Globs to match Go binaries from Rust target triples.
 */
export const RUST2GO = {
    "aarch64-apple-darwin": "memos-*-darwin-arm64.tar.gz",
    "x86_64-apple-darwin": "memos-*-darwin-x86_64.tar.gz",
    "aarch64-unknown-linux-gnu": "memos-*-linux-arm64.tar.gz",
    "x86_64-unknown-linux-gnu": "memos-*-linux-x86_64.tar.gz",
    "aarch64-pc-windows-msvc": "memos-*-windows-arm64.zip",
    "x86_64-pc-windows-msvc": "memos-*-windows-x86_64.zip"
} as const;
type SupportedBuild = (typeof RUST2GO)[keyof typeof RUST2GO];
export const SUPPORTED_BUILDS = Object.values(RUST2GO);
export const SUPPORTED_PLATFORMS = Object.keys(RUST2GO);

export const CROSS_COMPILE_PLATFORMS = {
    "aarch64-apple-darwin": ["x86_64-apple-darwin"],
    "x86_64-apple-darwin": ["aarch64-apple-darwin"],
    // "x86_64-pc-windows-msvc": ["aarch64-pc-windows-msvc"],
    "x86_64-unknown-linux-gnu": [
        "x86_64-pc-windows-msvc"
        // ,"aarch64-unknown-linux-gnu"
    ]
    // ,"aarch64-unknown-linux-gnu": ["x86_64-unknown-linux-gnu"]
} as const;

/**
 * Parse command-line arguments and environment variables.
 */
interface ParsedArgs {
    getAll: boolean;
    crossCompile: boolean;
    rustTarget: string | undefined;
}

function parseDownloadArgs(): ParsedArgs {
    const { values } = parseArgs({
        args: Bun.argv,
        options: {
            all: { type: "boolean", default: toBool(import.meta.env.DOWNLOAD_ALL) },
            CI: { type: "boolean", default: toBool(import.meta.env.CI) },
            "cross-compile": {
                type: "boolean",
                default: toBool(import.meta.env.CROSS_COMPILE)
            },
            "rust-target": { type: "string", default: import.meta.env.RUST_TARGET }
        },
        allowPositionals: true,
        strict: true
    });

    return {
        getAll: values.all ?? false,
        crossCompile: (values.CI ?? false) || (values["cross-compile"] ?? false),
        rustTarget: values["rust-target"] as string | undefined
    };
}

/**
 * Get assets for all supported builds.
 */
function getAllAssets(): string[] {
    const assets = uniqueSorted(SUPPORTED_BUILDS);
    console.log(`${CYAN}Matched all.${RESET}`);
    console.log(`${CYAN}Assets: ${assets}${RESET}`);
    return assets;
}

/**
 * Parse and validate rust target string, returning valid and invalid targets.
 *
 * @param rustTargets - The rust target string to parse. It may contain multiple targets separated by commas.
 * @returns An object containing valid and invalid targets.
 */
function parseRustTargets(rustTargets: string): {
    valid: Set<keyof typeof RUST2GO>;
    invalid: Set<string>;
} {
    const targets = rustTargets
        .split(",")
        .map((t) => t.trim())
        .filter((t) => t.length > 0);

    return targets.reduce<{
        valid: Set<keyof typeof RUST2GO>;
        invalid: Set<string>;
    }>(
        (acc, target) => {
            if (target in RUST2GO) {
                acc.valid.add(target as keyof typeof RUST2GO);
            } else {
                acc.invalid.add(target);
            }
            return acc;
        },
        {
            valid: new Set<keyof typeof RUST2GO>(),
            invalid: new Set<string>()
        }
    );
}

/**
 * Get assets for specified rust targets.
 */
function getRustTargetAssets(targets: string): string[] {
    const { valid, invalid } = parseRustTargets(targets);

    if (invalid.size > 0) {
        console.warn(
            `${YELLOW}Ignoring unsupported RUST_TARGET entries: ${[...invalid].join(
                ", "
            )}${RESET}`
        );
    }

    if (valid.size === 0) {
        throw new Error(`Unsupported RUST_TARGET: ${targets}`);
    }

    const assets = uniqueSorted([...valid].map((t) => RUST2GO[t]));
    console.log(`${CYAN}Matching assets for ${[...valid].join(", ")}${RESET}`);
    console.log(`${CYAN}Assets: ${assets}${RESET}`);
    return assets;
}

/**
 * Get the current platform triplet.
 */
function getCurrentTriplet(): string {
    const current_platform = process.platform.replace("win32", "windows");
    const current_arch = process.arch.replaceAll("x64", "x86_64");
    return makeTripletFromFileName(`${current_platform}-${current_arch}`);
}

/**
 * Get assets for cross-compilation targets.
 */
function getCrossCompileAssets(currentTriplet: string): string[] | null {
    console.log(`${MAGENTA}Cross-compile requested for host ${currentTriplet}…${RESET}`);
    const crossTargets =
        currentTriplet in CROSS_COMPILE_PLATFORMS
            ? CROSS_COMPILE_PLATFORMS[currentTriplet as keyof typeof CROSS_COMPILE_PLATFORMS]
            : undefined;

    if (!crossTargets) {
        console.warn(
            `${YELLOW}Host ${currentTriplet} has no predefined cross targets; falling back to host build.${RESET}`
        );
    }

    const targetTriplets = [
        ...new Set([...(crossTargets ?? []), currentTriplet])
    ] as (keyof typeof RUST2GO)[];

    const assets = uniqueSorted(
        targetTriplets
            .map((triplet) => RUST2GO[triplet])
            .filter((glob): glob is SupportedBuild => Boolean(glob))
    );

    if (assets.length > 0) {
        console.log(`${CYAN}Assets: ${assets}${RESET}`);
        return assets;
    }

    console.warn(
        `${YELLOW}Cross-compile requested but host ${currentTriplet} resolved to no supported assets.${RESET}`
    );
    return null;
}

/**
 * Get assets matching the current platform.
 */
function getPlatformAssets(currentTriplet: string): string[] {
    const current_platform = process.platform.replace("win32", "windows");
    const current_arch = process.arch.replaceAll("x64", "x86_64");

    const platformMatches = SUPPORTED_BUILDS.filter(
        (p) => p.includes(current_platform) && p.includes(current_arch)
    );

    if (platformMatches.length > 0) {
        console.log(
            `${CYAN}Host: ${currentTriplet}\t Target(s): [${platformMatches}] ${RESET}`
        );
        return uniqueSorted(platformMatches);
    }

    return uniqueSorted(SUPPORTED_BUILDS.slice());
}

/**
 * Get the list of files to download based on command line arguments and environment variables.
 *
 * @returns An array of file globs to download.
 */
export function getDownloadFilesGlob(): string[] {
    const args = parseDownloadArgs();
    console.log(`${MAGENTA}Matching assets…${RESET}`);

    if (args.getAll) {
        return getAllAssets();
    }

    if (args.rustTarget) {
        return getRustTargetAssets(args.rustTarget);
    }

    const currentTriplet = getCurrentTriplet();

    if (args.crossCompile) {
        const crossAssets = getCrossCompileAssets(currentTriplet);
        if (crossAssets) {
            return crossAssets;
        }
    }

    return getPlatformAssets(currentTriplet);
}

/**
 *  Get the requested tag from the command line argument `--tag` or from the environment variable `MEMOS_VERSION`.
 *
 * @returns The requested tag or null if not specified.
 */
export function getRequestedTag(): string | null {
    const { values } = parseArgs({
        args: Bun.argv,
        options: {
            tag: {
                type: "string",
                default: import.meta.env.MEMOS_VERSION
            }
        },
        allowPositionals: true
    });

    const tag = values.tag;
    if (tag?.match(/^v?\d+\.\d+\.\d+(-\S+)?$/)) {
        const version = tag.startsWith("v") ? tag.slice(1) : tag;
        return `v${version}`;
    }

    return null;
}

/**
 * Fetch and parse a `SHA256SUMS.txt` file uploaded to a GitHub release by GoReleaser.
 *
 * @param source - The URL or local path to the sum file.
 * @returns Map of files and their sums if valid JSON; otherwise empty object (`{}`).
 */
export async function fetchAndParseSha256Sums(source: string): Promise<Record<string, string>> {
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
        const response = await fetchWithRateLimit(source);
        if (!response) {
            throw new Error(`Unable to fetch ${source}.`);
        }
        sha256Sums = await response.text();
    }

    return parseSha256Sums(sha256Sums);
}

async function downloadMemos(downloadFilesGlob: string[], tag: string) {
    const repoUrl = `https://api.github.com/repos/${RELEASE_REPOSITORY}/releases/tags/${tag}`;

    // Fetch data from GitHub API.
    const response = await fetchWithRateLimit(repoUrl);
    if (!response?.ok) {
        throw new Error(`Failed to fetch GitHub release ${tag}: ${response?.statusText}`);
    }
    const ghRelease = (await response.json()) as GitHubRelease;
    const releaseAssets = ghRelease.assets as GitHubAsset[];

    if (!releaseAssets || releaseAssets.length === 0) {
        throw new Error("Failed to fetch assets");
    }

    console.log(
        `\x1b[34mMatching GitHub assets from ${RELEASE_REPOSITORY}:${ghRelease.tag_name}…\x1b[0m`
    );

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

    console.log(`\x1b[32mMatched ${selectedFiles.length} files\x1b[0m`);

    // Download files in parallel.
    console.log("\x1b[34mDownloading GitHub assets…\x1b[0m");
    await async
        .eachLimit(selectedFiles, 5, async (ghAsset: GitHubAsset) => {
            const fileName = ghAsset.name;
            const dstPath = `./server-dist/${fileName}`;

            if (fs.existsSync(dstPath)) {
                fs.rmSync(dstPath, { force: true, recursive: true });
            }
            process.stdout.write(`${ghAsset.browser_download_url}\n`);
            await downloadFileWithRateLimit(ghAsset.browser_download_url, dstPath)
                .then(() => {
                    console.log(`${GREEN}[OK]${RESET} Downloaded ${CYAN}${fileName}${RESET}`);
                })
                .catch((error) => {
                    console.log(`${RED}[ERROR] ${fileName}${RESET}: ${error}`);
                    throw error;
                });
        })
        .catch((error: any) => {
            throw error;
        });

    // Check hashes via memos_SHA256SUMS.txt.
    const fileHashes = await fetchAndParseSha256Sums(sha256sums);
    console.log("\x1b[34mChecking downloaded file hashes…\x1b[0m");
    await async
        .eachLimit(selectedFiles, 2, async (file: GitHubAsset) => {
            const fileName = file.name;

            const filePath = `./server-dist/${fileName}`;
            const fileHash = await sha256File(filePath);

            if (fileHash !== fileHashes[fileName]) {
                console.log(`${RED}[ERROR]${RESET} ${fileName} ${CYAN}${fileHash}${RESET}`);
                throw new Error(
                    `Hash mismatch for ${fileName}. Expected: ${fileHashes[fileName]}, got: ${fileHash}`
                );
            }
            console.log(`${GREEN}[OK]${RESET} ${fileName} ${CYAN}${fileHash}${RESET}`);
        })
        .catch((error) => {
            throw error;
        });

    // Extract files in parallel.
    console.log(`${BLUE}Extracting downloaded files…${RESET}`);
    await async.eachLimit(selectedFiles, 2, async (file: GitHubAsset) => {
        const uuid = crypto.randomUUID();
        const extractDir = `./server-dist/${uuid}`;
        if (!fs.existsSync(extractDir)) {
            fs.mkdirSync(extractDir);
        }

        const fileName = file.name;
        const filePath = `./server-dist/${fileName}`;
        if (fileName.endsWith(".zip") || fileName.endsWith(".tar.gz")) {
            await decompress(filePath, extractDir)
                .then((files: any[]) => {
                    console.log(
                        `$\x1b[32m[OK]\x1b[0m \x1b[36m${fileName}\x1b[0m Extracted ${files.length} files.`
                    );
                })
                .catch((error: any) => {
                    console.log(`\x1b[31m[ERROR]\x1b[0m \x1b[36m${fileName}\x1b[0m ${error}`);
                    fs.rmSync(extractDir, { recursive: true });
                    throw error;
                });
        }

        const exe = fileName.includes("windows") ? ".exe" : "";

        const triplet = makeTripletFromFileName(fileName);
        fs.renameSync(`${extractDir}/memos${exe}`, `./server-dist/memos-${triplet}${exe}`);
        // chmod +x downloaded file.
        if (process.platform !== "win32") {
            fs.chmodSync(`./server-dist/memos-${triplet}${exe}`, 0o755);
        }

        // Check if there's a sidecar front-end folder (Memos v0.18.2 - v0.21.0).
        const sidecarDist = `${extractDir}/dist`;
        const frontendDist = "./server-dist/dist";
        if (fs.existsSync(sidecarDist)) {
            // Move front-end dist folder only once, as it's the same for all platforms.
            if (!fs.existsSync(frontendDist)) {
                fs.renameSync(sidecarDist, frontendDist);
            }
        }

        fs.rmSync(extractDir, { recursive: true });
        fs.rmSync(filePath);
    });

    // Save tag to `./server-dist/MEMOS_VERSION.txt`.
    const versionFilePath = "./server-dist/MEMOS_VERSION.txt";
    if (fs.existsSync(versionFilePath)) {
        fs.rmSync(versionFilePath);
    }
    fs.writeFileSync(versionFilePath, tag, { encoding: "utf8", mode: 0o644 });
}

async function main() {
    const startTime = performance.now();

    const repoRoot = findRepositoryRoot();
    console.log(`Repository root is \`${repoRoot}\``);
    process.chdir(repoRoot);

    let requestedTag = getRequestedTag();
    if (!requestedTag) {
        console.log(
            `${YELLOW}No tag specified, will use the latest release from ${RELEASE_REPOSITORY}${RESET}`
        );
        requestedTag = await getLatestReleaseTag(RELEASE_REPOSITORY);
        if (!requestedTag) {
            throw new Error(
                `Failed to fetch latest release tag from ${RELEASE_REPOSITORY}. Please specify a tag with --tag or set MEMOS_VERSION environment variable.`
            );
        }
        console.log(`${GREEN}Using latest release tag: ${requestedTag}${RESET}`);
    }
    console.log(
        `${BLACK}${BG_WHITE}|> Running script "Download Memos Builds for tag ${requestedTag}…" <|${RESET}`
    );

    if (
        import.meta.env.CI === "true" &&
        import.meta.env.MEMOS_VERSION !== requestedTag &&
        import.meta.env.GITHUB_ENV &&
        fs.existsSync(import.meta.env.GITHUB_ENV)
    ) {
        console.log(
            `${YELLOW}Running in CI environment, using GITHUB_ENV to set MEMOS_VERSION.${RESET}`
        );
        fs.appendFileSync(import.meta.env.GITHUB_ENV, `MEMOS_VERSION=${requestedTag}\n`);
    }

    const serverDistDir = "./server-dist";
    const serverDistDirExists =
        fs.existsSync(serverDistDir) && fs.statSync(serverDistDir).isDirectory();
    if (!serverDistDirExists) {
        fs.mkdirSync(serverDistDir, { recursive: true, mode: 0o755 });
    }

    const binaries = getDownloadFilesGlob().map((glob) => {
        const exe = glob.includes("windows") ? ".exe" : "";
        return `memos-${makeTripletFromFileName(glob)}${exe}`;
    });

    // Check if the present binaries matches the requested tag.
    const versionFilePath = `${serverDistDir}/MEMOS_VERSION.txt`;
    const version = fs.existsSync(versionFilePath)
        ? fs.readFileSync(versionFilePath, "utf8").trim()
        : "";

    const validBinaries = binaries.every((bin) => {
        console.log(`Checking if ${MAGENTA}${bin}${RESET} already exists…`);
        return fs.existsSync(`${serverDistDir}/${bin}`);
    });

    if (validBinaries) {
        if (version === requestedTag) {
            console.log(`Found all required binaries for ${GREEN}${version}${RESET}.`);
            console.log("Skipping download.");
            return;
        }
        console.log(
            `${YELLOW}Existing binaries for ${MAGENTA}${version}${YELLOW} do not match the requested tag ${MAGENTA}${requestedTag}${YELLOW}.${RESET}`
        );
    }

    // Remove a previous dist folder (Memos v0.18.2 - v0.21.0), if it exists.
    const distDir = "./server-dist/dist";
    if (serverDistDirExists && fs.existsSync(distDir) && fs.statSync(distDir).isDirectory()) {
        fs.rmSync(distDir, { force: true, recursive: true });
    }

    if (process.env.GITHUB_TOKEN) {
        console.log(`${GREEN}Found GitHub token. Will use it for authentication.${RESET}`);
    }

    const downloadFilesGlob = getDownloadFilesGlob();
    await downloadMemos(downloadFilesGlob, requestedTag);

    const endTime = performance.now();
    const timeElapsed = endTime - startTime;
    console.log("Time elapsed: ", timeElapsed, "ms");
}

if (import.meta.main) {
    // This script can download (with --all) about 52 MB of data for the main supported platforms
    // (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc and x86_64-unknown-linux-gnu).
    // It will take around 7 minutes on a slow 1 Mbps connection.
    //
    // The timeout prevents the script from running forever if something goes wrong on the CI.
    const timeoutMinutes = 10;
    const wrapped = async.timeout(main, timeoutMinutes * 60 * 1000, "Script timed out.");
    wrapped((err?: Error | null, _data?: any) => {
        if (err) {
            throw err;
        }
    });
}
