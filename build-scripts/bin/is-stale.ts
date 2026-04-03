#!/usr/bin/env bun
/**
 * `is-stale` tracks whether a task needs to run by hashing its declared inputs and outputs.
 *
 * Usage:
 *  1. `bun run ./build-scripts/bin/is-stale.ts <stamp-file> -s <source> [-s <source> ...] [-g <generate> ...] [--no-gitignore] [--cwd <dir>]`
 *  2. `bun run ./build-scripts/bin/is-stale.ts <stamp-file> --update [--cwd <dir>]`
 *
 * Exit codes:
 *  - `0`: stale
 *  - `1`: fresh
 *  - `2`: error
 */

import * as path from "node:path";
import { parseArgs } from "node:util";
import * as Bun from "bun";
import { GitCliError, getTaskStaleness, updateTaskStamp } from "../lib/taskStamps";
import { GREEN, RED, RESET } from "../lib/terminal";

const DEFAULT_CWD = process.cwd();
const EXIT_CODE_STALE = 0;
const EXIT_CODE_FRESH = 1;
const EXIT_CODE_ERROR = 2;

type CliArgs = {
    cwd: string;
    generates: string[];
    help: boolean;
    noGitignore: boolean;
    sources: string[];
    stampFile: string;
    update: boolean;
};

class UsageError extends Error {
    constructor(message: string) {
        super(message);
        this.name = "UsageError";
    }
}

const USAGE = `Usage:
  bun run ./build-scripts/bin/is-stale.ts <stamp-file> -s <source> [-s <source> ...] [-g <generate> ...] [--no-gitignore] [--cwd <dir>]
  bun run ./build-scripts/bin/is-stale.ts <stamp-file> --update [--cwd <dir>]

Notes:
  Store stamp files as .task files, typically under .build-stamps/.

Exit codes:
  0 stale in check mode, or update completed successfully
  1 not stale in check mode
  2 invalid usage or runtime error`;

function parseCliArgs(argv: string[]): CliArgs {
    const { positionals, values } = parseArgs({
        allowPositionals: true,
        args: argv,
        options: {
            cwd: {
                default: DEFAULT_CWD,
                short: "d",
                type: "string"
            },
            generates: {
                multiple: true,
                short: "g",
                type: "string"
            },
            help: {
                default: false,
                short: "h",
                type: "boolean"
            },
            "no-gitignore": {
                default: false,
                type: "boolean"
            },
            sources: {
                multiple: true,
                short: "s",
                type: "string"
            },
            update: {
                default: false,
                short: "u",
                type: "boolean"
            }
        },
        strict: true
    });

    const help = (values.help as boolean | undefined) ?? false;
    const update = (values.update as boolean | undefined) ?? false;
    const stampFile = positionals[0] ?? "";

    if (positionals.length > 1) {
        throw new UsageError(
            `Expected exactly one positional <stamp-file>, received ${positionals.length}.`
        );
    }

    if (!help && stampFile.length === 0) {
        throw new UsageError("Missing required <stamp-file> positional argument.");
    }

    const sources = (values.sources ?? []) as string[];
    if (!help && !update && sources.length === 0) {
        throw new UsageError("Missing required --sources / -s values.");
    }

    if (update && sources.length > 0) {
        throw new UsageError(
            "Do not pass --sources / -s with --update; the existing .task stamp definition is reused."
        );
    }

    if (update && ((values.generates ?? []) as string[]).length > 0) {
        throw new UsageError(
            "Do not pass --generates / -g with --update; the existing .task stamp definition is reused."
        );
    }

    if (update && ((values["no-gitignore"] as boolean | undefined) ?? false)) {
        throw new UsageError(
            "Do not pass --no-gitignore with --update; the existing .task stamp definition is reused."
        );
    }

    return {
        cwd: path.resolve((values.cwd as string | undefined) ?? DEFAULT_CWD),
        generates: (values.generates ?? []) as string[],
        help,
        noGitignore: (values["no-gitignore"] as boolean | undefined) ?? false,
        sources,
        stampFile,
        update
    };
}

function printUsage(output: "stderr" | "stdout" = "stdout") {
    if (output === "stderr") {
        console.error(USAGE);
        return;
    }

    console.log(USAGE);
}

function printFriendlyError(error: unknown) {
    if (error instanceof GitCliError) {
        console.error(`${RED}Error:${RESET} ${error.message}`);
        console.error(`Command: git ${error.args.join(" ")}`);
        console.error(`Working directory: ${error.cwd}`);
        if (error.stderr.trim().length > 0) {
            console.error(`git stderr:\n${error.stderr.trimEnd()}`);
        }
        if (error.stdout.trim().length > 0) {
            console.error(`git stdout:\n${error.stdout.trimEnd()}`);
        }
        return;
    }

    const message = error instanceof Error ? error.message : String(error);
    console.error(`${RED}Error:${RESET} ${message}`);
}

async function main(): Promise<void> {
    const args = parseCliArgs(Bun.argv.slice(2));

    if (args.help) {
        printUsage();
        process.exit(EXIT_CODE_STALE);
    }

    if (args.update) {
        await updateTaskStamp({
            cwd: args.cwd,
            stampFile: args.stampFile
        });
        console.log(`${GREEN}Task stamp updated.${RESET}`);
        process.exit(EXIT_CODE_STALE);
    }

    const result = await getTaskStaleness({
        cwd: args.cwd,
        generates: args.generates,
        noGitignore: args.noGitignore,
        sources: args.sources,
        stampFile: args.stampFile
    });

    process.exit(result.isStale ? EXIT_CODE_STALE : EXIT_CODE_FRESH);
}

main().catch((error) => {
    printFriendlyError(error);
    if (error instanceof UsageError) {
        printUsage("stderr");
    }
    process.exit(EXIT_CODE_ERROR);
});
