import { describe, expect, test } from "bun:test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import * as Bun from "bun";
import {
    mergePlatformSarif,
    type PlatformSarif,
    type SarifLog,
    type SarifResult,
    type SarifRule
} from "./sarifMerge";

const TARGETS = [
    "x86_64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc"
];
const CLI_PATH = path.resolve(import.meta.dir, "merge-clippy-sarif.ts");

function runCli(artifactDir: string, output: string, missingFile: string, expected = TARGETS) {
    return Bun.spawnSync(
        [
            process.execPath,
            CLI_PATH,
            "--artifact-dir",
            artifactDir,
            "--expected",
            expected.join(","),
            "--output",
            output,
            "--missing-file",
            missingFile
        ],
        { stderr: "pipe", stdout: "pipe" }
    );
}

async function writeArtifact(
    dir: string,
    target: string,
    sarifLog: SarifLog | string
): Promise<void> {
    const content = typeof sarifLog === "string" ? sarifLog : JSON.stringify(sarifLog);
    await fs.writeFile(path.join(dir, `clippy-results-${target}.sarif`), content, "utf8");
}

function result(overrides: Partial<SarifResult> = {}): SarifResult {
    return {
        ruleId: "dead_code",
        ruleIndex: 0,
        message: { text: "fields `epoch` and `inflight` are never read" },
        locations: [
            {
                physicalLocation: {
                    artifactLocation: {
                        uri: "crates/memospot/src/runtime_config.rs"
                    },
                    region: {
                        startLine: 296,
                        startColumn: 12,
                        endLine: 296,
                        endColumn: 17
                    }
                }
            }
        ],
        level: "warning",
        ...overrides
    };
}

function sarif(results: SarifResult[], rules: SarifRule[] = [{ id: "dead_code" }]): SarifLog {
    return {
        version: "2.1.0",
        $schema: "https://json.schemastore.org/sarif-2.1.0.json",
        runs: [
            {
                tool: {
                    driver: {
                        name: "clippy",
                        informationUri: "https://rust-lang.github.io/rust-clippy/",
                        rules
                    }
                },
                results
            }
        ]
    };
}

function platform(
    target: string,
    results: SarifResult[],
    rules: SarifRule[] = [{ id: "dead_code" }]
): PlatformSarif {
    return { target, sarif: sarif(results, rules) };
}

describe("mergePlatformSarif", () => {
    test("deduplicates a finding reported by every platform and leaves its message unchanged", () => {
        // GIVEN every expected platform reports the same finding
        const inputs = TARGETS.map((target) => platform(target, [result()]));

        // WHEN merging the SARIF logs
        const merged = mergePlatformSarif(inputs, TARGETS);

        // THEN the finding appears once with the original message
        expect(merged.missingTargets).toEqual([]);
        expect(merged.sarif.runs).toHaveLength(1);
        expect(merged.sarif.runs[0].results).toEqual([result()]);
    });

    test("marks a subset finding with only-on when every platform was analyzed", () => {
        // GIVEN only macOS arm64 and Windows report the finding
        const inputs = TARGETS.map((target) =>
            platform(
                target,
                target === "aarch64-apple-darwin" || target === "x86_64-pc-windows-msvc"
                    ? [result()]
                    : []
            )
        );

        // WHEN merging the SARIF logs
        const merged = mergePlatformSarif(inputs, TARGETS);

        // THEN the finding names the platforms that reported it
        expect(merged.missingTargets).toEqual([]);
        expect(merged.sarif.runs[0].results?.[0].message).toEqual({
            text: "fields `epoch` and `inflight` are never read (only on: macOS arm64, Windows x64)"
        });
    });

    test("uses detected-on wording and records notifications when artifacts are missing", () => {
        // GIVEN two artifacts are missing
        const inputs = [
            platform("aarch64-apple-darwin", [result()]),
            platform("x86_64-pc-windows-msvc", [result()])
        ];

        // WHEN merging the available SARIF logs
        const merged = mergePlatformSarif(inputs, TARGETS);

        // THEN the finding is not described as exclusive, and the gaps are recorded
        expect(merged.missingTargets).toEqual([
            "x86_64-unknown-linux-gnu",
            "x86_64-apple-darwin"
        ]);
        expect(merged.sarif.runs[0].results?.[0].message).toEqual({
            text: "fields `epoch` and `inflight` are never read (detected on: macOS arm64, Windows x64)"
        });
        expect(merged.sarif.runs[0].invocations).toEqual([
            {
                executionSuccessful: true,
                toolExecutionNotifications: [
                    {
                        level: "error",
                        message: {
                            text: "Missing or unreadable clippy SARIF for Linux x64 (x86_64-unknown-linux-gnu); results from that platform are not included."
                        }
                    },
                    {
                        level: "error",
                        message: {
                            text: "Missing or unreadable clippy SARIF for macOS x64 (x86_64-apple-darwin); results from that platform are not included."
                        }
                    }
                ]
            }
        ]);
    });

    test("merges rules and remaps rule indexes", () => {
        // GIVEN platforms with different rule tables
        const inputs = [
            platform(
                "x86_64-unknown-linux-gnu",
                [
                    result({
                        message: { text: "unused variable: `value`" },
                        ruleId: "unused_variables",
                        ruleIndex: 1
                    })
                ],
                [{ id: "dead_code" }, { id: "unused_variables" }]
            ),
            platform("aarch64-apple-darwin", [result()], [{ id: "dead_code" }])
        ];

        // WHEN merging the SARIF logs
        const merged = mergePlatformSarif(inputs, [
            "x86_64-unknown-linux-gnu",
            "aarch64-apple-darwin"
        ]);

        // THEN both rules survive and each result points at its merged index
        expect(merged.sarif.runs[0].tool.driver.rules).toEqual([
            { id: "dead_code" },
            { id: "unused_variables" }
        ]);
        expect(merged.sarif.runs[0].results).toEqual([
            result({
                message: {
                    text: "fields `epoch` and `inflight` are never read (only on: macOS arm64)"
                },
                ruleIndex: 0
            }),
            result({
                message: { text: "unused variable: `value` (only on: Linux x64)" },
                ruleId: "unused_variables",
                ruleIndex: 1
            })
        ]);
    });

    test("normalizes Windows URI separators when deduplicating", () => {
        // GIVEN the Windows artifact uses backslashes and is loaded first
        const inputs = [
            platform("x86_64-pc-windows-msvc", [
                result({
                    locations: [
                        {
                            physicalLocation: {
                                artifactLocation: {
                                    uri: "crates\\memospot\\src\\runtime_config.rs"
                                },
                                region: {
                                    startLine: 296,
                                    startColumn: 12,
                                    endLine: 296,
                                    endColumn: 17
                                }
                            }
                        }
                    ]
                })
            ]),
            platform("x86_64-unknown-linux-gnu", [result()])
        ];

        // WHEN merging the SARIF logs
        const merged = mergePlatformSarif(inputs, [
            "x86_64-pc-windows-msvc",
            "x86_64-unknown-linux-gnu"
        ]);

        // THEN the finding is deduplicated and the output uses forward slashes
        expect(merged.sarif.runs[0].results).toHaveLength(1);
        expect(
            merged.sarif.runs[0].results?.[0].locations?.[0].physicalLocation?.artifactLocation
                ?.uri
        ).toBe("crates/memospot/src/runtime_config.rs");
    });

    test("keeps findings at different locations separate", () => {
        // GIVEN two findings at different lines
        const inputs = TARGETS.map((target) =>
            platform(target, [
                result(),
                result({
                    message: { text: "unused variable: `value`" },
                    ruleId: "unused_variables",
                    ruleIndex: 0,
                    locations: [
                        {
                            physicalLocation: {
                                artifactLocation: {
                                    uri: "crates/memospot/src/runtime_config.rs"
                                },
                                region: {
                                    startLine: 300,
                                    startColumn: 9,
                                    endLine: 300,
                                    endColumn: 14
                                }
                            }
                        }
                    ]
                })
            ])
        );

        // WHEN merging the SARIF logs
        const merged = mergePlatformSarif(inputs, TARGETS);

        // THEN both findings survive
        expect(merged.sarif.runs[0].results).toHaveLength(2);
        expect(merged.sarif.runs[0].results?.map((item) => item.ruleId)).toEqual([
            "dead_code",
            "unused_variables"
        ]);
    });

    test("resolves a result without ruleId from ruleIndex", () => {
        // GIVEN every platform reports a result that references only ruleIndex
        const inputResult = result();
        delete inputResult.ruleId;
        const inputs = TARGETS.map((target) => platform(target, [inputResult]));

        // WHEN merging the SARIF logs
        const merged = mergePlatformSarif(inputs, TARGETS);

        // THEN the result gains the resolved ruleId and keeps the merged index
        expect(merged.sarif.runs[0].results).toEqual([result()]);
    });

    test("omits ruleId when neither ruleId nor ruleIndex resolves", () => {
        // GIVEN every platform reports a result with no rule reference
        const inputResult = result();
        delete inputResult.ruleId;
        delete inputResult.ruleIndex;
        const inputs = TARGETS.map((target) => platform(target, [inputResult]));

        // WHEN merging the SARIF logs
        const merged = mergePlatformSarif(inputs, TARGETS);

        // THEN the result keeps no ruleId and no ruleIndex
        const expected = result();
        delete expected.ruleId;
        delete expected.ruleIndex;
        expect(merged.sarif.runs[0].results).toEqual([expected]);
    });

    test("keeps divergent messages at the same location separate", () => {
        // GIVEN two platforms use different wording for the same rule and span
        const inputs = [
            platform("x86_64-unknown-linux-gnu", [
                result({ message: { text: "Alpha wording" } })
            ]),
            platform("aarch64-apple-darwin", [result({ message: { text: "Beta wording" } })])
        ];

        // WHEN merging the SARIF logs
        const merged = mergePlatformSarif(inputs, [
            "x86_64-unknown-linux-gnu",
            "aarch64-apple-darwin"
        ]);

        // THEN both wordings survive with their own platform suffix
        expect(merged.sarif.runs[0].results).toHaveLength(2);
        expect(merged.sarif.runs[0].results?.map((item) => item.message?.text)).toEqual([
            "Alpha wording (only on: Linux x64)",
            "Beta wording (only on: macOS arm64)"
        ]);
    });
});

describe("merge-clippy-sarif CLI", () => {
    test("writes the merged SARIF and the missing-target list", async () => {
        // GIVEN a single Linux artifact and three missing platforms
        const dir = await fs.mkdtemp(path.join(os.tmpdir(), "memospot-sarif-"));
        await writeArtifact(dir, "x86_64-unknown-linux-gnu", sarif([result()]));
        const output = path.join(dir, "merged.sarif");
        const missingFile = path.join(dir, "missing.txt");

        try {
            // WHEN running the CLI
            const cli = runCli(dir, output, missingFile);

            // THEN it succeeds with partial results and names the missing targets
            expect(cli.exitCode).toBe(0);
            const merged = JSON.parse(await fs.readFile(output, "utf8")) as SarifLog;
            expect(merged.runs[0].results?.[0].message?.text).toBe(
                "fields `epoch` and `inflight` are never read (detected on: Linux x64)"
            );
            expect(await fs.readFile(missingFile, "utf8")).toBe(
                `${TARGETS.slice(1).join("\n")}\n`
            );
        } finally {
            await fs.rm(dir, { force: true, recursive: true });
        }
    });

    test("refuses to write an empty analysis when every artifact is missing", async () => {
        // GIVEN no artifacts
        const dir = await fs.mkdtemp(path.join(os.tmpdir(), "memospot-sarif-"));
        const output = path.join(dir, "merged.sarif");
        const missingFile = path.join(dir, "missing.txt");

        try {
            // WHEN running the CLI
            const cli = runCli(dir, output, missingFile);

            // THEN it fails without writing a SARIF and reports every target
            expect(cli.exitCode).toBe(1);
            expect(await fs.readFile(missingFile, "utf8")).toBe(`${TARGETS.join("\n")}\n`);
            await expect(fs.access(output)).rejects.toThrow();
        } finally {
            await fs.rm(dir, { force: true, recursive: true });
        }
    });

    test("writes one merged result when every artifact is present", async () => {
        // GIVEN all expected artifacts contain the same finding
        const dir = await fs.mkdtemp(path.join(os.tmpdir(), "memospot-sarif-"));
        for (const target of TARGETS) {
            await writeArtifact(dir, target, sarif([result()]));
        }
        const output = path.join(dir, "merged.sarif");
        const missingFile = path.join(dir, "missing.txt");

        try {
            // WHEN running the CLI
            const cli = runCli(dir, output, missingFile);

            // THEN it succeeds with one deduplicated result and no missing targets
            expect(cli.exitCode).toBe(0);
            const merged = JSON.parse(await fs.readFile(output, "utf8")) as SarifLog;
            expect(merged.runs[0].results).toEqual([result()]);
            expect(merged.runs[0].invocations).toBeUndefined();
            expect(await fs.readFile(missingFile, "utf8")).toBe("");
        } finally {
            await fs.rm(dir, { force: true, recursive: true });
        }
    });

    test("reports unreadable artifacts as missing and merges the rest", async () => {
        // GIVEN one valid artifact, one invalid JSON, one SARIF without runs, and one missing
        const dir = await fs.mkdtemp(path.join(os.tmpdir(), "memospot-sarif-"));
        await writeArtifact(dir, "x86_64-unknown-linux-gnu", sarif([result()]));
        await writeArtifact(dir, "aarch64-apple-darwin", "{not json");
        await writeArtifact(
            dir,
            "x86_64-pc-windows-msvc",
            JSON.stringify({ version: "2.1.0" })
        );
        const output = path.join(dir, "merged.sarif");
        const missingFile = path.join(dir, "missing.txt");

        try {
            // WHEN running the CLI
            const cli = runCli(dir, output, missingFile);

            // THEN it succeeds with the valid results and reports the unreadable targets
            expect(cli.exitCode).toBe(0);
            const merged = JSON.parse(await fs.readFile(output, "utf8")) as SarifLog;
            expect(merged.runs[0].results?.[0].message?.text).toBe(
                "fields `epoch` and `inflight` are never read (detected on: Linux x64)"
            );
            expect((await fs.readFile(missingFile, "utf8")).trim().split("\n")).toEqual([
                "aarch64-apple-darwin",
                "x86_64-apple-darwin",
                "x86_64-pc-windows-msvc"
            ]);
            expect(cli.stderr.toString()).toContain("Unable to read");
        } finally {
            await fs.rm(dir, { force: true, recursive: true });
        }
    });

    test("fails when an artifact target is not listed in expected", async () => {
        // GIVEN an unexpected artifact file
        const dir = await fs.mkdtemp(path.join(os.tmpdir(), "memospot-sarif-"));
        await writeArtifact(dir, "x86_64-unknown-linux-gnu", sarif([result()]));
        await writeArtifact(dir, "extra-target", sarif([result()]));
        const output = path.join(dir, "merged.sarif");
        const missingFile = path.join(dir, "missing.txt");

        try {
            // WHEN running the CLI
            const cli = runCli(dir, output, missingFile);

            // THEN it fails with a configuration error instead of dropping the target
            expect(cli.exitCode).toBe(2);
            expect(cli.stderr.toString()).toContain(
                "Unexpected clippy SARIF artifacts for: extra-target"
            );
            await expect(fs.access(output)).rejects.toThrow();
            await expect(fs.access(missingFile)).rejects.toThrow();
        } finally {
            await fs.rm(dir, { force: true, recursive: true });
        }
    });
});
