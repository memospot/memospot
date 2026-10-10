/**
 * Manages state for a single settings section.
 *
 * Holds the baseline (last saved) configuration snapshot alongside the
 * editable input, derives `hasPendingChanges`, builds a JSON Patch on save,
 * and persists via `patchConfig`.
 *
 * Callers own toasts and rollback decisions: `save()` throws on failure.
 * Callers provide only a `FieldMapping`; wrap the returned object with
 * Svelte 5 `$state(...)` so mutations to `input` / `baselineInput` remain
 * reactive.
 */

import * as jsonpatch from "fast-json-patch";
import { patchConfig } from "./settings";
import { getAppConfig, getDefaultAppConfig } from "./tauri";
import type { Config } from "./types/gen/Config";
import type { ConfigUpdateResult } from "./types/gen/ConfigUpdateResult";

/**
 * Defines how a section maps between the persisted `Config` and its
 * editable form input.
 *
 * The page supplies only this mapping; the section handles orchestration.
 * `mapToInput` / `mapFromInput` are the sole places that know the section's
 * subtree and any value normalization (e.g. `envFromKV` / `envToKV`).
 */
export type FieldMapping<TInput extends Record<string, unknown>> = {
    /** Initial input used before `init()` completes. */
    initial: TInput;
    /** Derives editable input from the persisted `Config`. */
    mapToInput: (config: Config) => TInput;
    /** Writes editable input back into a `Config` clone for patching/persistence. */
    mapFromInput: (input: TInput, config: Config) => void;
    /** Optional gate that runs before `save()`. Return a field-keyed error message to reject the field. */
    validate?: (input: TInput) => Partial<Record<keyof TInput, string>>;
    /** Fields whose lifecycle is owned by a live command (e.g. `locale` via `set_locale`). Excluded from the patch and from the pending check. */
    live?: (keyof TInput)[];
};

/**
 * Interface for a single editable settings section.
 *
 * Create with `createSettingsSection()` and wrap with `$state(...)` at the
 * call site. Bind form controls to `input` and report to the settings header
 * via `useSectionActions({ loadDefaults, reloadCurrent, save,
 * hasPendingChanges }, onActionsChange)` — see `#lib/settingsUi.svelte`.
 * `baselineInput` is the last saved input (read-only) and is used to revert
 * individual fields (e.g. failed path validation) and to advance live fields
 * without going through `save()` — see `commitLive`.
 */
export type SettingsSection<TInput extends Record<string, unknown>> = {
    /** Editable form data. The page binds directly to this object. */
    input: TInput;
    /** Last saved input. Read-only; used for revert and pending detection. */
    baselineInput: Readonly<TInput>;
    /** True if `input` differs from `baselineInput` (excluding `live` fields). False before `init()`. */
    readonly hasPendingChanges: boolean;
    /** Loads the saved `Config`, establishes the baseline snapshot, and initializes `input`. */
    init(): Promise<void>;
    /** Stages the default `Config` into `input` only. Call `save()` to persist. */
    loadDefaults(): Promise<void>;
    /** Discards pending edits and restores `input` from `baselineInput`. */
    reset(): void;
    /** Commits a field that was applied via a live command (e.g. `locale` via `set_locale`) without going through `save()`. Advances the baseline so the field is neither pending nor included in the next patch. */
    commitLive<K extends keyof TInput>(key: K, value: TInput[K]): void;
    /**
     * Validates `input`, diffs against the baseline snapshot, and persists via `patchConfig`.
     *
     * Returns the update result on success, or `false` when there is nothing
     * to send. Throws the failing field messages on validation failure, or the
     * back-end error message on persistence failure; callers own toasts and
     * rollback. A `mapFromInput` throw (a mapping bug, not user input)
     * propagates as-is.
     */
    save(): Promise<ConfigUpdateResult | false>;
};

type Internal<TInput extends Record<string, unknown>> = SettingsSection<TInput> & {
    _initialized: boolean;
    _baselineConfigSnapshot: Config;
};

function filtered<TInput extends Record<string, unknown>>(
    value: TInput,
    liveSet: Set<string>
): Record<string, unknown> {
    const clone = jsonpatch.deepClone(value) as Record<string, unknown>;
    for (const key of liveSet) {
        delete clone[key];
    }
    return clone;
}

/**
 * Creates a new settings section.
 *
 * Returns a plain object intended to be wrapped with `$state(...)` in a
 * Svelte 5 component. Pending state is derived runes-natively: every
 * mutation refreshes `_pending` from the diff, so effects reading
 * `hasPendingChanges` re-run without a manual version counter.
 *
 * Live fields are excluded from pending checks and patches; call
 * `commitLive()` after a live command succeeds to advance their baseline.
 */
export function createSettingsSection<TInput extends Record<string, unknown>>(options: {
    mapping: FieldMapping<TInput>;
}) {
    const liveSet = new Set((options.mapping.live ?? []).map((k) => String(k)));

    const isPending = (self: Internal<TInput>) => {
        if (!self._initialized) return false;
        const diff = jsonpatch.compare(
            filtered(self.baselineInput as TInput, liveSet) as object,
            filtered(self.input, liveSet) as object
        );
        return diff.length > 0;
    };

    const section = {
        _initialized: false,
        _baselineConfigSnapshot: {} as Config,
        input: jsonpatch.deepClone(options.mapping.initial) as TInput,
        baselineInput: jsonpatch.deepClone(options.mapping.initial) as TInput,

        get hasPendingChanges() {
            // Read through `input` / `baselineInput` so the `$state` proxy
            // tracks the access: direct page mutations (e.g.
            // `section.input.theme = "dark"`) re-run dependent effects
            // without a manual version counter.
            return isPending(this as Internal<TInput>);
        },
        async init(this: Internal<TInput>): Promise<void> {
            const cfg = await getAppConfig();
            this._baselineConfigSnapshot = jsonpatch.deepClone(cfg) as Config;
            const mapped = options.mapping.mapToInput(cfg);
            this.input = jsonpatch.deepClone(mapped) as TInput;
            this.baselineInput = jsonpatch.deepClone(mapped) as TInput;
            this._initialized = true;
        },
        async loadDefaults(this: Internal<TInput>): Promise<void> {
            const cfg = await getDefaultAppConfig();
            const mapped = options.mapping.mapToInput(cfg);
            const cloned = jsonpatch.deepClone(mapped) as TInput;
            if (!this._initialized) {
                this._baselineConfigSnapshot = jsonpatch.deepClone(cfg) as Config;
                this.input = cloned;
                this.baselineInput = jsonpatch.deepClone(mapped) as TInput;
                this._initialized = true;
                return;
            }
            this.input = cloned;
        },
        reset(this: Internal<TInput>): void {
            this.input = jsonpatch.deepClone(this.baselineInput) as TInput;
        },
        commitLive<K extends keyof TInput>(
            this: Internal<TInput>,
            key: K,
            value: TInput[K]
        ): void {
            const nextInput = { ...this.input } as Record<string, unknown>;
            const nextBaseline = { ...this.baselineInput } as Record<string, unknown>;
            nextInput[key as string] = value as unknown;
            nextBaseline[key as string] = jsonpatch.deepClone(value) as unknown;
            this.input = nextInput as TInput;
            this.baselineInput = nextBaseline as TInput;
            if (this._initialized) {
                const tmp = jsonpatch.deepClone(this._baselineConfigSnapshot) as Config;
                const tmpInput = jsonpatch.deepClone(this.baselineInput) as TInput;
                try {
                    options.mapping.mapFromInput(tmpInput, tmp);
                    this._baselineConfigSnapshot = jsonpatch.deepClone(tmp) as Config;
                } catch {
                    // Keep the old snapshot. `mapFromInput` (and the underlying
                    // mapping/validation) threw for this live value.
                }
            }
        },
        save: undefined as unknown as () => Promise<ConfigUpdateResult | false>
    } as Internal<TInput>;

    const doSaveInner = async (self: Internal<TInput>) => {
        if (!self._initialized) return false;

        if (options.mapping.validate) {
            const errors = options.mapping.validate(self.input);
            const keys = Object.keys(errors) as (keyof TInput)[];
            if (keys.length > 0) {
                const messages = keys
                    .map((key) => errors[key])
                    .filter((message) => message) as string[];
                throw messages;
            }
        }

        const working = jsonpatch.deepClone(self._baselineConfigSnapshot) as Config;
        options.mapping.mapFromInput(self.input, working);

        let configForPatch: Config = working;
        if (liveSet.size > 0) {
            const diffAll = jsonpatch.compare(
                self._baselineConfigSnapshot as object,
                working as object
            );
            const filteredWorking = jsonpatch.deepClone(working) as Record<string, unknown>;
            let hasLiveOps = false;
            for (const op of diffAll) {
                const last = op.path.split("/").pop() ?? "";
                if (liveSet.has(jsonpatch.unescapePathComponent(last))) {
                    hasLiveOps = true;
                    const committedVal = jsonpatch.getValueByPointer(
                        self._baselineConfigSnapshot,
                        op.path
                    );
                    const currentVal = jsonpatch.getValueByPointer(filteredWorking, op.path);
                    if (committedVal === undefined) {
                        if (currentVal !== undefined) {
                            jsonpatch.applyOperation(filteredWorking, {
                                op: "remove",
                                path: op.path
                            });
                        }
                    } else if (currentVal === undefined) {
                        jsonpatch.applyOperation(filteredWorking, {
                            op: "add",
                            path: op.path,
                            value: committedVal
                        });
                    } else {
                        jsonpatch.applyOperation(filteredWorking, {
                            op: "replace",
                            path: op.path,
                            value: committedVal
                        });
                    }
                }
            }
            if (hasLiveOps) {
                configForPatch = filteredWorking as unknown as Config;
            }
        }

        const result = await patchConfig(self._baselineConfigSnapshot, configForPatch);
        if (result === false) return false;
        self._baselineConfigSnapshot = jsonpatch.deepClone(configForPatch) as Config;
        self.baselineInput = jsonpatch.deepClone(self.input) as TInput;
        return result;
    };

    (section as SettingsSection<TInput>).save = async function (
        this: Internal<TInput> | undefined
    ): Promise<ConfigUpdateResult | false> {
        const self = (this as Internal<TInput> | undefined) ?? (section as Internal<TInput>);
        return doSaveInner(self);
    };

    return section;
}
