/**
 * Merges per-platform clippy SARIF logs into a single run.
 *
 * Findings are grouped by rule, level, primary location, and message. Platforms
 * that reported the same finding are unioned, so cross-platform findings appear
 * once. The message gets a platform suffix only when the finding is not present
 * on every expected platform.
 */

export type SarifMessage = {
    text?: string;
    markdown?: string;
};

export type SarifRule = {
    id: string;
    fullDescription?: SarifMessage;
    shortDescription?: SarifMessage;
    helpUri?: string;
    help?: SarifMessage;
    defaultConfiguration?: { level?: string };
    properties?: Record<string, unknown>;
};

export type SarifRegion = {
    startLine?: number;
    startColumn?: number;
    endLine?: number;
    endColumn?: number;
    byteOffset?: number;
    byteLength?: number;
};

export type SarifPhysicalLocation = {
    artifactLocation?: {
        uri?: string;
        uriBaseId?: string;
        index?: number;
    };
    region?: SarifRegion;
};

export type SarifLocation = {
    physicalLocation?: SarifPhysicalLocation;
    message?: SarifMessage;
};

export type SarifResult = {
    ruleId?: string;
    ruleIndex?: number;
    message?: SarifMessage;
    locations?: SarifLocation[];
    level?: string;
    relatedLocations?: SarifLocation[];
    properties?: Record<string, unknown>;
    [key: string]: unknown;
};

export type SarifInvocation = {
    executionSuccessful?: boolean;
    toolExecutionNotifications?: Array<{
        level?: string;
        message?: SarifMessage;
    }>;
    [key: string]: unknown;
};

export type SarifRun = {
    tool: {
        driver: {
            name: string;
            informationUri?: string;
            rules?: SarifRule[];
            [key: string]: unknown;
        };
        [key: string]: unknown;
    };
    results?: SarifResult[];
    invocations?: SarifInvocation[];
    [key: string]: unknown;
};

export type SarifLog = {
    version: string;
    $schema?: string;
    runs: SarifRun[];
    [key: string]: unknown;
};

export type PlatformSarif = {
    target: string;
    sarif: SarifLog;
};

export type MergePlatformSarifResult = {
    sarif: SarifLog;
    missingTargets: string[];
    resultCount: number;
};

const PLATFORM_LABELS: Record<string, string> = {
    "x86_64-unknown-linux-gnu": "Linux x64",
    "aarch64-apple-darwin": "macOS arm64",
    "x86_64-apple-darwin": "macOS x64",
    "x86_64-pc-windows-msvc": "Windows x64"
};

export function platformLabel(target: string): string {
    return PLATFORM_LABELS[target] ?? target;
}

function clone<T>(value: T): T {
    return structuredClone(value);
}

function normalizeUri(uri: string): string {
    return uri.replaceAll("\\", "/");
}

function normalizeLocationUris(result: SarifResult): void {
    for (const location of [...(result.locations ?? []), ...(result.relatedLocations ?? [])]) {
        const artifactLocation = location.physicalLocation?.artifactLocation;
        if (artifactLocation?.uri) {
            artifactLocation.uri = normalizeUri(artifactLocation.uri);
        }
    }
}

function messageText(result: SarifResult): string {
    return result.message?.text ?? result.message?.markdown ?? "";
}

function primaryLocationKey(result: SarifResult): string {
    const physical = result.locations?.[0]?.physicalLocation;
    const uri = normalizeUri(physical?.artifactLocation?.uri ?? "");
    const region = physical?.region;
    // Keep message and level in the key so target-specific variants are not
    // silently collapsed into the first platform's result.
    return [
        result.ruleId ?? "",
        result.level ?? "",
        uri,
        region?.startLine ?? 0,
        region?.startColumn ?? 0,
        region?.endLine ?? 0,
        region?.endColumn ?? 0,
        messageText(result)
    ].join("\u0000");
}

function compareStrings(first: string, second: string): number {
    if (first < second) {
        return -1;
    }
    if (first > second) {
        return 1;
    }
    return 0;
}

function sortedPlatforms(targets: Set<string>, expectedTargets: string[]): string[] {
    return expectedTargets.filter((target) => targets.has(target));
}

export function mergePlatformSarif(
    inputs: PlatformSarif[],
    expectedTargets: string[]
): MergePlatformSarifResult {
    if (expectedTargets.length === 0) {
        throw new Error("At least one expected target is required.");
    }
    if (new Set(expectedTargets).size !== expectedTargets.length) {
        throw new Error("Expected targets must be unique.");
    }

    const expected = new Set(expectedTargets);
    const inputsByTarget = new Map<string, PlatformSarif>();
    for (const input of inputs) {
        if (!expected.has(input.target)) {
            throw new Error(`Unexpected SARIF target: ${input.target}`);
        }
        if (inputsByTarget.has(input.target)) {
            throw new Error(`Duplicate SARIF target: ${input.target}`);
        }
        inputsByTarget.set(input.target, input);
    }

    const missingTargets = expectedTargets.filter((target) => !inputsByTarget.has(target));
    const orderedInputs = expectedTargets
        .map((target) => inputsByTarget.get(target))
        .filter((input): input is PlatformSarif => input !== undefined);

    const baseRun = orderedInputs[0]?.sarif.runs[0];
    if (!baseRun) {
        throw new Error("At least one input SARIF run is required.");
    }

    const rulesById = new Map<string, SarifRule>();
    const ruleOrder: string[] = [];
    for (const input of orderedInputs) {
        for (const run of input.sarif.runs) {
            for (const rule of run.tool.driver.rules ?? []) {
                if (!rulesById.has(rule.id)) {
                    rulesById.set(rule.id, clone(rule));
                    ruleOrder.push(rule.id);
                }
            }
        }
    }

    const resultByKey = new Map<string, SarifResult>();
    const platformsByKey = new Map<string, Set<string>>();
    for (const input of orderedInputs) {
        for (const run of input.sarif.runs) {
            const runRules = run.tool.driver.rules ?? [];
            for (const sourceResult of run.results ?? []) {
                const ruleId =
                    sourceResult.ruleId ??
                    (sourceResult.ruleIndex !== undefined
                        ? runRules[sourceResult.ruleIndex]?.id
                        : undefined) ??
                    "";
                const result = clone(sourceResult);
                if (ruleId.length > 0) {
                    result.ruleId = ruleId;
                } else {
                    delete result.ruleId;
                }
                normalizeLocationUris(result);

                if (ruleId.length > 0 && !rulesById.has(ruleId)) {
                    rulesById.set(ruleId, { id: ruleId });
                    ruleOrder.push(ruleId);
                }

                const key = primaryLocationKey(result);
                const existing = resultByKey.get(key);
                if (existing) {
                    platformsByKey.get(key)?.add(input.target);
                    continue;
                }
                resultByKey.set(key, result);
                platformsByKey.set(key, new Set([input.target]));
            }
        }
    }

    const rules = ruleOrder
        .map((id) => rulesById.get(id))
        .filter((rule): rule is SarifRule => rule !== undefined);
    const ruleIndexById = new Map(rules.map((rule, index) => [rule.id, index]));
    const mergedResults = [...resultByKey.entries()]
        .map(([key, result]) => {
            const platforms = sortedPlatforms(
                platformsByKey.get(key) ?? new Set<string>(),
                expectedTargets
            );
            const suffix =
                platforms.length < expectedTargets.length
                    ? ` (${missingTargets.length > 0 ? "detected on" : "only on"}: ${platforms
                          .map(platformLabel)
                          .join(", ")})`
                    : "";
            const merged = clone(result);
            merged.message = {
                ...(merged.message ?? {}),
                text: `${messageText(result)}${suffix}`
            };
            if (merged.message.markdown !== undefined) {
                merged.message.markdown = `${merged.message.markdown}${suffix}`;
            }
            const ruleIndex = ruleIndexById.get(merged.ruleId ?? "");
            if (ruleIndex === undefined) {
                delete merged.ruleIndex;
            } else {
                merged.ruleIndex = ruleIndex;
            }
            return merged;
        })
        .sort((first, second) => {
            const firstLocation = first.locations?.[0]?.physicalLocation;
            const secondLocation = second.locations?.[0]?.physicalLocation;
            const firstUri = normalizeUri(firstLocation?.artifactLocation?.uri ?? "");
            const secondUri = normalizeUri(secondLocation?.artifactLocation?.uri ?? "");
            return (
                compareStrings(firstUri, secondUri) ||
                (firstLocation?.region?.startLine ?? 0) -
                    (secondLocation?.region?.startLine ?? 0) ||
                (firstLocation?.region?.startColumn ?? 0) -
                    (secondLocation?.region?.startColumn ?? 0) ||
                compareStrings(first.ruleId ?? "", second.ruleId ?? "") ||
                compareStrings(messageText(first), messageText(second))
            );
        });

    // clippy-sarif uses URI-based locations and emits no run-level base IDs, so
    // the merged run intentionally keeps only the fields it actually produces.
    const baseDriver = clone(baseRun.tool.driver);
    baseDriver.rules = rules;
    const run: SarifRun = {
        tool: {
            ...clone(baseRun.tool),
            driver: baseDriver
        },
        results: mergedResults
    };

    if (missingTargets.length > 0) {
        run.invocations = [
            {
                executionSuccessful: true,
                toolExecutionNotifications: missingTargets.map((target) => ({
                    level: "error",
                    message: {
                        text: `Missing or unreadable clippy SARIF for ${platformLabel(
                            target
                        )} (${target}); results from that platform are not included.`
                    }
                }))
            }
        ];
    }

    const sarif: SarifLog = {
        version: "2.1.0",
        $schema:
            orderedInputs[0].sarif.$schema ?? "https://json.schemastore.org/sarif-2.1.0.json",
        runs: [run]
    };

    return {
        sarif,
        missingTargets,
        resultCount: mergedResults.length
    };
}
