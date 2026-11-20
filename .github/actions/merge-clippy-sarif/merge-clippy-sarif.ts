#!/usr/bin/env bun
/**
 * Merges per-platform clippy SARIF artifacts into one code scanning upload.
 *
 * Usage:
 *   bun run ./.github/actions/merge-clippy-sarif/merge-clippy-sarif.ts \
 *     --artifact-dir clippy-sarif \
 *     --expected "x86_64-unknown-linux-gnu,aarch64-apple-darwin,x86_64-apple-darwin,x86_64-pc-windows-msvc" \
 *     --output clippy-results.sarif \
 *     --missing-file missing-platforms.txt
 *
 * Missing or unreadable artifacts are written to the missing file so the caller
 * can upload the available results first and fail afterwards.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import { parseArgs } from "node:util";
import { mergePlatformSarif, type PlatformSarif, type SarifLog } from "./sarifMerge";

const USAGE = `Usage:
  bun run ./.github/actions/merge-clippy-sarif/merge-clippy-sarif.ts --artifact-dir <dir> --expected <targets> --output <file> [--missing-file <file>]

Options:
  --artifact-dir  Directory containing clippy-results-<target>.sarif
  --expected      Comma-separated list of expected Rust targets
  --output        Merged SARIF file to write
  --missing-file  File listing missing or unreadable targets (default: missing-platforms.txt)`;

const SARIF_PREFIX = "clippy-results-";
const SARIF_SUFFIX = ".sarif";

class UsageError extends Error {
    constructor(message: string) {
        super(message);
        this.name = "UsageError";
    }
}

async function fileExists(filePath: string): Promise<boolean> {
    try {
        await fs.access(filePath);
        return true;
    } catch {
        return false;
    }
}

async function readSarif(filePath: string): Promise<SarifLog> {
    const content = await fs.readFile(filePath, "utf8");
    const parsed = JSON.parse(content) as SarifLog;
    if (!Array.isArray(parsed.runs)) {
        throw new Error("SARIF log has no runs array.");
    }
    return parsed;
}

async function main(): Promise<void> {
    const { values } = parseArgs({
        args: process.argv.slice(2),
        options: {
            "artifact-dir": { type: "string" },
            expected: { type: "string" },
            help: { short: "h", type: "boolean" },
            "missing-file": { type: "string" },
            output: { type: "string" }
        },
        strict: true
    });

    if (values.help) {
        console.log(USAGE);
        return;
    }

    const artifactDir = values["artifact-dir"];
    const expectedRaw = values.expected;
    const output = values.output;
    const missingFile = values["missing-file"] ?? "missing-platforms.txt";

    if (!artifactDir || !expectedRaw || !output) {
        throw new UsageError(
            "Missing required --artifact-dir, --expected, or --output option."
        );
    }

    const expectedTargets = expectedRaw
        .split(",")
        .map((target) => target.trim())
        .filter((target) => target.length > 0);
    if (expectedTargets.length === 0) {
        throw new UsageError("Expected at least one target.");
    }
    if (new Set(expectedTargets).size !== expectedTargets.length) {
        throw new UsageError("Expected targets must be unique.");
    }

    if (await fileExists(artifactDir)) {
        const entries = await fs.readdir(artifactDir, { withFileTypes: true });
        const unexpectedTargets = entries
            .filter(
                (entry) =>
                    entry.isFile() &&
                    entry.name.startsWith(SARIF_PREFIX) &&
                    entry.name.endsWith(SARIF_SUFFIX)
            )
            .map((entry) =>
                entry.name.slice(SARIF_PREFIX.length, entry.name.length - SARIF_SUFFIX.length)
            )
            .filter((target) => !expectedTargets.includes(target));
        if (unexpectedTargets.length > 0) {
            throw new UsageError(
                `Unexpected clippy SARIF artifacts for: ${unexpectedTargets.join(
                    ", "
                )}. Update --expected to include them.`
            );
        }
    }

    const inputs: PlatformSarif[] = [];
    const missingTargets: string[] = [];
    for (const target of expectedTargets) {
        const sarifPath = path.join(artifactDir, `${SARIF_PREFIX}${target}${SARIF_SUFFIX}`);
        if (!(await fileExists(sarifPath))) {
            missingTargets.push(target);
            continue;
        }

        try {
            inputs.push({ sarif: await readSarif(sarifPath), target });
        } catch (error) {
            const message = error instanceof Error ? error.message : String(error);
            console.error(`Unable to read ${sarifPath}: ${message}`);
            missingTargets.push(target);
        }
    }

    await fs.writeFile(
        missingFile,
        missingTargets.map((target) => `${target}\n`).join(""),
        "utf8"
    );

    if (inputs.length === 0) {
        console.error(
            "No clippy SARIF artifacts were available; refusing to upload an empty analysis."
        );
        process.exit(1);
    }

    const merged = mergePlatformSarif(inputs, expectedTargets);
    await fs.writeFile(output, `${JSON.stringify(merged.sarif, null, 2)}\n`, "utf8");

    const available = expectedTargets.length - missingTargets.length;
    console.log(
        `Merged ${merged.resultCount} result(s) from ${available}/${expectedTargets.length} platform(s).`
    );
    if (missingTargets.length > 0) {
        console.log(`Missing: ${missingTargets.join(", ")}`);
    }
}

main().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    if (error instanceof UsageError) {
        console.error(USAGE);
        process.exit(2);
    }
    process.exit(1);
});
