/**
 * Manages state for a single settings section.
 *
 * Holds the baseline (last saved) configuration snapshot alongside the
 * editable input, derives `hasPendingChanges`, builds a JSON Patch on save,
 * validates and persists via `patchConfig`, and rolls back on failure.
 *
 * Callers provide only a `FieldMapping`; wrap the returned object with
 * Svelte 5 `$state(...)` so mutations to `input` / `baselineInput` remain
 * reactive.
 */

import * as jsonpatch from "fast-json-patch";
import { toast } from "svelte-sonner";
import { patchConfig } from "./settings";
import { getAppConfig, getDefaultAppConfig } from "./tauri";
import type { Config } from "./types/gen/Config";

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
    /** Optional gate that runs before `save()`. Return a field-keyed error message to reject the field and surface a toast. */
    validate?: (input: TInput) => Partial<Record<keyof TInput, string>>;
    /** Fields whose lifecycle is owned by a live command (e.g. `locale` via `set_locale`). Excluded from the patch and from the pending check. */
    live?: (keyof TInput)[];
};

/**
 * Interface for a single editable settings section.
 *
 * Create with `createSettingsSection()` and wrap with `$state(...)` at the
 * call site. Bind form controls to `input` and wire the settings header via
 * `buildSectionActions(() => section.loadDefaults(), () => section.reset(), () => section.save(), section.hasPendingChanges)`.
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
    hasPendingChanges: boolean;
    /** Loads the saved `Config`, establishes the baseline snapshot, and initializes `input`. */
    init(): Promise<void>;
    /** Stages the default `Config` into `input` only. Call `save()` to persist. */
    loadDefaults(): Promise<void>;
    /** Discards pending edits and restores `input` from `baselineInput`. */
    reset(): void;
    /** Commits a field that was applied via a live command (e.g. `locale` via `set_locale`) without going through `save()`. Advances the baseline so the field is neither pending nor included in the next patch. */
    commitLive<K extends keyof TInput>(key: K, value: TInput[K]): void;
    /** Validates `input`, diffs against the baseline snapshot, and persists via `patchConfig`. Rolls back `input` on validation or persistence failure. */
    save(): Promise<boolean>;
};

/** Unescapes a JSON Pointer reference token per RFC 6901 (`~1` → `/`, `~0` → `~`). */
function unescapePointer(part: string) {
    return part.replaceAll("~1", "/").replaceAll("~0", "~");
}

/** Splits a JSON Pointer into its unescaped reference tokens. */
function splitPointer(pointer: string) {
    return pointer.split("/").slice(1).map(unescapePointer);
}

/** Reads the value at a JSON Pointer. Returns `undefined` if the path does not exist. */
function getValueByPointer(obj: unknown, pointer: string) {
    if (pointer === "" || pointer === "/") return obj;
    let cur: unknown = obj;
    for (const part of splitPointer(pointer)) {
        if (cur == null || typeof cur !== "object") return undefined;
        cur = (cur as Record<string, unknown>)[part];
    }
    return cur;
}

/** Writes a value at a JSON Pointer, creating intermediate objects as needed. */
function setValueByPointer(obj: Record<string, unknown>, pointer: string, value: unknown) {
    const parts = splitPointer(pointer);
    let cur: Record<string, unknown> = obj;
    for (let i = 0; i < parts.length - 1; i++) {
        const part = parts[i];
        if (cur[part] == null || typeof cur[part] !== "object") {
            cur[part] = {};
        }
        cur = cur[part] as Record<string, unknown>;
    }
    cur[parts[parts.length - 1]] = value as never;
}

/** Deletes the property at a JSON Pointer. No-ops if the path does not exist. */
function removeByPointer(obj: Record<string, unknown>, pointer: string) {
    const parts = splitPointer(pointer);
    let cur: Record<string, unknown> = obj;
    for (let i = 0; i < parts.length - 1; i++) {
        const part = parts[i];
        if (cur[part] == null || typeof cur[part] !== "object") return;
        cur = cur[part] as Record<string, unknown>;
    }
    delete cur[parts[parts.length - 1]];
}

type Internal<TInput extends Record<string, unknown>> = SettingsSection<TInput> & {
    _initialized: boolean;
    _baselineConfigSnapshot: Config;
    _version: number;
};

/**
 * Creates a new settings section.
 *
 * Returns a plain object intended to be wrapped with `$state(...)` in a
 * Svelte 5 component. Each mutation replaces `input` / `baselineInput` with
 * a new cloned object and bumps `_version` so `hasPendingChanges` — which
 * depends on `_version` — recomputes reliably under the `$state` proxy.
 *
 * Live fields are excluded from pending checks and patches; call
 * `commitLive()` after a live command succeeds to advance their baseline.
 */
export function createSettingsSection<TInput extends Record<string, unknown>>(options: {
    mapping: FieldMapping<TInput>;
}) {
    const liveSet = new Set((options.mapping.live ?? []).map((k) => String(k)));

    const section = {
        _initialized: false,
        _baselineConfigSnapshot: {} as Config,
        _version: 0,
        input: jsonpatch.deepClone(options.mapping.initial) as TInput,
        baselineInput: jsonpatch.deepClone(options.mapping.initial) as TInput,

        get hasPendingChanges() {
            const self = this as Internal<TInput>;
            // Depend on `_version` so Svelte effects that read
            // `hasPendingChanges` re-run when any mutation bumps the
            // version, even if inner spread tracking is missed by the proxy.
            void self._version;
            if (!self._initialized) return false;
            const filteredInput = { ...self.input } as Record<string, unknown>;
            const filteredBaseline = { ...self.baselineInput } as Record<string, unknown>;
            for (const k of liveSet) {
                delete filteredInput[k];
                delete filteredBaseline[k];
            }
            const diff = jsonpatch.compare(filteredBaseline as object, filteredInput as object);
            return diff.length > 0;
        },
        async init(this: Internal<TInput>): Promise<void> {
            const json = await getAppConfig();
            const cfg = JSON.parse(json) as Config;
            this._baselineConfigSnapshot = jsonpatch.deepClone(cfg) as Config;
            const mapped = options.mapping.mapToInput(cfg);
            this.input = jsonpatch.deepClone(mapped) as TInput;
            (this as unknown as { baselineInput: TInput }).baselineInput = jsonpatch.deepClone(
                mapped
            ) as TInput;
            this._initialized = true;
            this._version++;
        },
        async loadDefaults(this: Internal<TInput>): Promise<void> {
            const json = await getDefaultAppConfig();
            const cfg = JSON.parse(json) as Config;
            const mapped = options.mapping.mapToInput(cfg);
            const cloned = jsonpatch.deepClone(mapped) as TInput;
            if (!this._initialized) {
                this._baselineConfigSnapshot = jsonpatch.deepClone(cfg) as Config;
                this.input = cloned;
                (this as unknown as { baselineInput: TInput }).baselineInput =
                    jsonpatch.deepClone(mapped) as TInput;
                this._initialized = true;
                this._version++;
                return;
            }
            this.input = cloned;
            this._version++;
        },
        reset(this: Internal<TInput>): void {
            this.input = jsonpatch.deepClone(this.baselineInput) as TInput;
            this._version++;
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
            (this as unknown as { baselineInput: TInput }).baselineInput =
                nextBaseline as TInput;
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
            this._version++;
        },
        save: undefined as unknown as () => Promise<boolean>
    } as Internal<TInput>;

    const doSaveInner = async (self: Internal<TInput>) => {
        if (!self._initialized) return false;

        if (options.mapping.validate) {
            const errors = options.mapping.validate(self.input);
            const keys = Object.keys(errors) as (keyof TInput)[];
            if (keys.length > 0) {
                for (const key of keys) {
                    const message = errors[key];
                    if (message) toast.error(String(message));
                }
                const cloned = jsonpatch.deepClone(self.baselineInput) as TInput;
                const next = { ...self.input } as Record<string, unknown>;
                for (const key of keys) {
                    next[key as string] = (cloned as Record<string, unknown>)[key as string];
                }
                self.input = next as TInput;
                self._version++;
                return false;
            }
        }

        const working = jsonpatch.deepClone(self._baselineConfigSnapshot) as Config;
        try {
            options.mapping.mapFromInput(self.input, working);
        } catch {
            return false;
        }

        let configForPatch: Config = working;
        if (liveSet.size > 0) {
            const diffAll = jsonpatch.compare(
                self._baselineConfigSnapshot as object,
                working as object
            );
            const filtered = jsonpatch.deepClone(working) as Record<string, unknown>;
            let hasLiveOps = false;
            for (const op of diffAll) {
                const last = op.path.split("/").pop() ?? "";
                if (liveSet.has(last)) {
                    hasLiveOps = true;
                    const committedVal = getValueByPointer(
                        self._baselineConfigSnapshot,
                        op.path
                    );
                    if (committedVal === undefined) {
                        removeByPointer(filtered, op.path);
                    } else {
                        setValueByPointer(filtered, op.path, committedVal);
                    }
                }
            }
            if (hasLiveOps) {
                configForPatch = filtered as unknown as Config;
            }
        }

        const diffFiltered = jsonpatch.compare(
            self._baselineConfigSnapshot as object,
            configForPatch as object
        );
        if (diffFiltered.length === 0) return false;

        try {
            await patchConfig(self._baselineConfigSnapshot, configForPatch);
            self._baselineConfigSnapshot = jsonpatch.deepClone(configForPatch) as Config;
            (self as unknown as { baselineInput: TInput }).baselineInput = jsonpatch.deepClone(
                self.input
            ) as TInput;
            self._version++;
            return true;
        } catch {
            self.input = jsonpatch.deepClone(self.baselineInput) as TInput;
            self._version++;
            return false;
        }
    };

    (section as SettingsSection<TInput>).save = async function (
        this: Internal<TInput> | undefined
    ): Promise<boolean> {
        const self = (this as Internal<TInput> | undefined) ?? (section as Internal<TInput>);
        return doSaveInner(self);
    };

    return section;
}
