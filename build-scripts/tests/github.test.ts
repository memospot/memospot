import { afterEach, describe, expect, test } from "bun:test";
import { RELEASE_REPOSITORY } from "../bin/downloadMemos";
import { getDefaultGithubHeaders, getLatestReleaseTag } from "../lib/github";

describe("getDefaultGithubHeaders", () => {
    const originalToken = process.env.GITHUB_TOKEN;

    afterEach(() => {
        if (originalToken) {
            process.env.GITHUB_TOKEN = originalToken;
        } else {
            delete process.env.GITHUB_TOKEN;
        }
    });

    test("omits Authorization header when GITHUB_TOKEN is not set", () => {
        delete process.env.GITHUB_TOKEN;

        const headers = getDefaultGithubHeaders();

        expect(headers).toMatchObject({
            "User-Agent": "Bun",
            "X-GitHub-Api-Version": "2022-11-28"
        });
        expect(headers.Authorization).toBeUndefined();
    });

    test("adds Authorization header when GITHUB_TOKEN is set", () => {
        process.env.GITHUB_TOKEN = "ghp_test";

        const headers = getDefaultGithubHeaders();

        expect(headers).toMatchObject({
            "User-Agent": "Bun",
            "X-GitHub-Api-Version": "2022-11-28",
            Authorization: "Bearer ghp_test"
        });
    });
});

test("test getLatestReleaseTag", async () => {
    const tag = await getLatestReleaseTag(RELEASE_REPOSITORY);

    console.warn(`Latest release tag: ${tag}`);

    expect(tag).toBeDefined();
    expect(tag).toMatch(/^v\d+\.\d+\.\d+$/);
});
