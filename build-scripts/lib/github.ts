import * as Bun from "bun";
import type { GitHubRelease } from "../types/github";
import { RESET } from "./terminal";
/**
 * Get the default headers for GitHub API requests.
 *
 * Will include Authorization header if GITHUB_TOKEN is set in environment variables.
 * @returns
 */
export function getDefaultGithubHeaders(): Record<string, string> {
    const defaultHeaders: Record<string, string> = {
        "User-Agent": "Bun",
        "X-GitHub-Api-Version": "2022-11-28"
    };
    if (process.env.GITHUB_TOKEN) {
        return {
            ...defaultHeaders,
            Authorization: `Bearer ${process.env.GITHUB_TOKEN}`
        };
    }
    return defaultHeaders;
}

/*
 * Check and apply rate limiting behavior for a fetch response.
 *
 * If the response status is 403 and the remaining rate limit is 0, waits for the reset time and retries.
 *
 * @param response The fetch response.
 * @param retryFunc The function to retry if rate limit is exceeded.
 * @returns The response.
 */
async function rateLimit(response: Response, retryFunc: () => Promise<any>): Promise<Response> {
    if (response.status === 403 && response.headers.get("X-RateLimit-Remaining") === "0") {
        const resetHeader = response.headers.get("X-RateLimit-Reset");
        if (!resetHeader) {
            throw new Error("Rate limited but X-RateLimit-Reset header is missing.");
        }

        const resetTimeSeconds = Number(resetHeader);
        if (Number.isNaN(resetTimeSeconds)) {
            throw new Error("Invalid X-RateLimit-Reset header value.");
        }

        const waitTime = Math.max(0, resetTimeSeconds * 1000 - Date.now());
        const waitSecs = Math.ceil(waitTime / 1000);
        console.log(`$YELLOWRate limit exceeded, waiting for ${waitSecs}s${RESET}`);
        await new Promise((resolve) => setTimeout(resolve, waitTime));
        return retryFunc();
    }
    return response;
}

/**
 * Fetch the latest release tag from the GitHub repository.
 * @returns The latest release tag or null if not found.
 */
export async function getLatestReleaseTag(repository: string): Promise<string | null> {
    const latestUrl = `https://api.github.com/repos/${repository}/releases/latest`;
    console.debug(`Fetching latest release tag from ${latestUrl}…`);

    const response = await fetchWithRateLimit(latestUrl);
    if (!response || !response.ok) {
        throw new Error(`Failed to fetch latest release: ${response?.statusText}`);
    }
    const release = (await response.json()) as GitHubRelease;
    return release.tag_name || null;
}

/**
 * Fetch a response from a URL with rate limit handling.
 * @param srcURL The URL to fetch.
 * @returns The fetch response.
 */
export async function fetchWithRateLimit(srcURL: string): Promise<Response> {
    console.log(`Fetching ${srcURL}…`);

    return fetch(srcURL, {
        method: "GET",
        redirect: "follow",
        headers: getDefaultGithubHeaders()
    })
        .then(
            async (r) => {
                const response = await rateLimit(r, () => fetchWithRateLimit(srcURL));

                if (!response.ok) {
                    throw new Error(`Failed to download file: ${response.statusText}`);
                }
                return response;
            },
            () => {
                throw new Error(`Unable to fetch ${srcURL}.`);
            }
        )
        .catch((e) => {
            throw new Error(`Unable to fetch ${srcURL}: ${e}`);
        });
}

/**
 * Download a file from a URL and save it to a local file.
 * @param srcURL The URL of the file to download.
 * @param dstFile The local file path to save the downloaded file.
 */
export async function downloadFileWithRateLimit(srcURL: string, dstFile: string) {
    const file = Bun.file(dstFile);
    const writer = file.writer();

    console.log(`Downloading ${srcURL}…`);

    await fetch(srcURL, { redirect: "follow", headers: getDefaultGithubHeaders() }).then(
        async (r) => {
            const response = await rateLimit(r, () =>
                downloadFileWithRateLimit(srcURL, dstFile)
            );

            if (!response.ok) {
                throw new Error(`Failed to download file: ${response.statusText}`);
            }
            const reader = response.body?.getReader();
            while (reader) {
                const { done, value } = await reader.read();
                if (done) {
                    break;
                }
                writer.write(value);
            }
        },
        () => {
            throw new Error(`Unable to download ${srcURL}.`);
        }
    );
    writer.end();
}
