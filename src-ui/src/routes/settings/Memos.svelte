<script lang="ts">
import type { Selected } from "bits-ui";
import * as jsonpatch from "fast-json-patch";
import { onMount } from "svelte";
import Code from "svelte-radix/Code.svelte";
import ExternalLink from "svelte-radix/ExternalLink.svelte";
import LightningBolt from "svelte-radix/LightningBolt.svelte";
import LockClosed from "svelte-radix/LockClosed.svelte";
import { toast } from "svelte-sonner";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue
} from "$lib/components/ui/select";
import { Setting, SettingToggle } from "$lib/components/ui/setting/index";
import { debouncePromise } from "$lib/debounce";
import { envFromKV, envToKV } from "$lib/environmentVariables";
import { m } from "$lib/i18n";
import { patchConfig } from "$lib/settings";
import {
    aliasesFromLocale,
    buildSectionActions,
    type SectionActionsProps
} from "$lib/settingsUi";
import { getAppConfig, getDefaultAppConfig, pathExists } from "$lib/tauri";
import type { Config } from "$lib/types/gen/Config";

let { onActionsChange }: SectionActionsProps = $props();

let initialConfig = $state({}) as Config;
let currentConfig = $state({}) as Config;
let isInitialized = $state(false);
let input = $state({
    mode: "",
    binaryPath: "",
    workingDir: "",
    dataDir: "",
    bindAddr: "",
    bindPort: 0,
    envVarsEnabled: false,
    envVars: ""
});

const memosModeNames = {
    prod: m.settingsMemosModeProduction(),
    dev: m.settingsMemosModeDevelopment(),
    demo: m.settingsMemosModeDemonstration()
} as const;

let selectedMode: Selected<string> = $derived({
    label: memosModeNames[input.mode as keyof typeof memosModeNames],
    value: input.mode
});

onMount(async () => {
    const initialJSON = await getAppConfig();
    initialConfig = JSON.parse(initialJSON);
    currentConfig = jsonpatch.deepClone(initialConfig);
    await setPageToInitialConfig();
    isInitialized = true;
});

async function setPageToInitialConfig() {
    input = {
        mode: initialConfig.memos.mode ?? "prod",
        binaryPath: initialConfig.memos.binary_path ?? "",
        workingDir: initialConfig.memos.working_dir ?? "",
        dataDir: initialConfig.memos.data ?? "",
        bindAddr: initialConfig.memos.addr ?? "",
        bindPort: initialConfig.memos.port ?? 0,
        envVarsEnabled: initialConfig.memos.env.enabled ?? false,
        envVars: envFromKV((initialConfig.memos.env.vars ?? {}) as Record<string, string>)
    };

    currentConfig.memos = jsonpatch.deepClone(initialConfig.memos);
}

async function setPageToDefaultConfig() {
    const defaultJSON = JSON.parse(await getDefaultAppConfig()) as Config;
    input = {
        mode: defaultJSON.memos.mode ?? "prod",
        binaryPath: defaultJSON.memos.binary_path ?? "",
        workingDir: defaultJSON.memos.working_dir ?? "",
        dataDir: defaultJSON.memos.data ?? "",
        bindAddr: defaultJSON.memos.addr ?? "",
        bindPort: defaultJSON.memos.port ?? 0,
        envVarsEnabled: defaultJSON.memos.env.enabled ?? false,
        envVars: envFromKV((defaultJSON.memos.env.vars ?? {}) as Record<string, string>)
    };

    currentConfig.memos = jsonpatch.deepClone(defaultJSON.memos);
}

function syncCurrentConfigFromInput() {
    currentConfig.memos.mode = input.mode;
    currentConfig.memos.binary_path = input.binaryPath;
    currentConfig.memos.working_dir = input.workingDir;
    currentConfig.memos.data = input.dataDir;
    currentConfig.memos.addr = input.bindAddr;
    currentConfig.memos.port = input.bindPort;
    currentConfig.memos.env.enabled = input.envVarsEnabled;
    currentConfig.memos.env.vars = envToKV(input.envVars);
}

async function setMemosMode(s: Selected<string> | undefined) {
    input.mode = s?.value ?? "prod";
    currentConfig.memos.mode = input.mode;
}

async function validateMemosDataDir(e: Event | KeyboardEvent) {
    if (e.type === "keypress" && (e as KeyboardEvent).key !== "Enter") return;

    await validatePath(e).then(
        (_ok) => {
            currentConfig.memos.data = input.dataDir;
        },
        (_err) => {
            input.dataDir = initialConfig.memos.data as string;
        }
    );
}

async function validateMemosBinaryPath(e: Event | KeyboardEvent) {
    if (e.type === "keypress" && (e as KeyboardEvent).key !== "Enter") return;

    await validatePath(e).then(
        (_ok) => {
            currentConfig.memos.binary_path = input.binaryPath;
        },
        (_err) => {
            input.binaryPath = initialConfig.memos.binary_path as string;
        }
    );
}

async function validateMemosWorkingDir(e: Event | KeyboardEvent) {
    if (e.type === "keypress" && (e as KeyboardEvent).key !== "Enter") return;

    await validatePath(e).then(
        (_ok) => {
            currentConfig.memos.working_dir = input.workingDir;
        },
        (_err) => {
            input.workingDir = initialConfig.memos.working_dir as string;
        }
    );
}

async function validatePath(e: Event) {
    const input = e.target as HTMLInputElement;
    if (!input.value || (await pathExists(input.value))) {
        // Must allow empty paths.
        return Promise.resolve();
    }
    toast.error(m.settingsErrPathDoesNotExist());
    return Promise.reject();
}

async function updateEnvVars(_: Event) {
    const kv = envToKV(input.envVars);
    currentConfig.memos.env.vars = kv;
    input.envVars = envFromKV(kv);
}

async function updateSetting(updateFn?: () => void): Promise<boolean> {
    return debouncePromise(async () => {
        updateFn?.();
        return await patchConfig(initialConfig, currentConfig).then(
            (_ok) => {
                initialConfig = jsonpatch.deepClone(currentConfig);
            },
            (_err) => {
                currentConfig = jsonpatch.deepClone(initialConfig);
            }
        );
    })();
}

$effect(() => {
    if (!currentConfig.memos) return;
    syncCurrentConfigFromInput();
});

const hasPendingChanges = $derived(
    (isInitialized &&
        (input.mode !== (initialConfig.memos?.mode ?? "prod") ||
        input.binaryPath !== (initialConfig.memos?.binary_path ?? "") ||
        input.workingDir !== (initialConfig.memos?.working_dir ?? "") ||
        input.dataDir !== (initialConfig.memos?.data ?? "") ||
        input.bindAddr !== (initialConfig.memos?.addr ?? "") ||
        input.bindPort !== (initialConfig.memos?.port ?? 0) ||
        input.envVarsEnabled !== (initialConfig.memos?.env?.enabled ?? false) ||
        input.envVars !==
            envFromKV((initialConfig.memos?.env?.vars ?? {}) as Record<string, string>)))
        || false
);

$effect(() => {
    onActionsChange?.(
        buildSectionActions(setPageToDefaultConfig, setPageToInitialConfig, updateSetting, hasPendingChanges)
    );
});
</script>

<div class="space-y-3">
  <div>
    <h3 class="text-lg font-medium flex flex-row">
      {m.settingsMemosDescription()}<a
        href="https://www.usememos.com/docs/install/runtime-options"
        target="_blank"
      >
        <ExternalLink class="ml-1 mt-1 h-[1.2rem] w-[1.2rem]" />
      </a>
    </h3>

    <p class="text-sm text-muted-foreground">{m.settingsOverview()}</p>
  </div>

  <Setting
    name={m.settingsMemosMode()}
    desc={m.settingsMemosModeDescription()}
    searchId="memos-mode"
    searchAliases={aliasesFromLocale(m.settingsMemosModeSearchAliases())}
  >
    <Select selected={selectedMode} onSelectedChange={setMemosMode}>
      <SelectTrigger class="ml-1 min-w-max md:w-64">
        <SelectValue placeholder={m.settingsMemosMode()} />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="prod">
          {memosModeNames.prod} <LightningBolt class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </SelectItem>
        <SelectItem value="dev">
          {memosModeNames.dev} <Code class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </SelectItem>
        <SelectItem value="demo">
          {memosModeNames.demo} <LockClosed class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </SelectItem>
      </SelectContent>
    </Select>
  </Setting>

  <Setting
    name={m.settingsMemosDataDirectory()}
    desc={m.settingsMemosDataDirectoryDescription()}
    searchId="memos-data-directory"
    searchAliases={aliasesFromLocale(m.settingsMemosDataDirectorySearchAliases())}
  >
    <input
      id="dataDirectory"
      type="text"
      bind:value={input.dataDir}
      onfocusout={validateMemosDataDir}
      onkeypress={validateMemosDataDir}
      class="p-2 rounded-md border bg-background min-w-max md:w-96"
    />
  </Setting>

  <Setting
    name={m.settingsMemosBinaryPath()}
    desc={m.settingsMemosBinaryPathDescription()}
    searchId="memos-binary-path"
    searchAliases={aliasesFromLocale(m.settingsMemosBinarySearchAliases())}
  >
    <input
      id="binaryPath"
      type="text"
      bind:value={input.binaryPath}
      onfocusout={validateMemosBinaryPath}
      onkeypress={validateMemosBinaryPath}
      class="p-2 rounded-md border bg-background min-w-max md:w-96"
    />
  </Setting>

  <Setting
    name={m.settingsMemosWorkingDirectory()}
    desc={m.settingsMemosWorkingDirectoryDescription()}
    searchId="memos-working-directory"
    searchAliases={aliasesFromLocale(m.settingsMemosWorkingDirectorySearchAliases())}
  >
    <input
      id="workingDirectory"
      type="text"
      bind:value={input.workingDir}
      onfocusout={validateMemosWorkingDir}
      onkeypress={validateMemosWorkingDir}
      class="p-2 rounded-md border bg-background min-w-max md:w-96"
    />
  </Setting>

  <Setting
    name={m.settingsMemosBindAddress()}
    desc={m.settingsMemosBindAddressDescription()}
    searchId="memos-bind-address"
    searchAliases={aliasesFromLocale(m.settingsMemosBindAddressSearchAliases())}
  >
    <input
      id="bindAddress"
      type="text"
      bind:value={input.bindAddr}
      onfocusout={() => {
        currentConfig.memos.addr = input.bindAddr;
      }}
      class="p-2 rounded-md border bg-background min-w-max md:w-96"
    />
  </Setting>

  <Setting
    name={m.settingsMemosBindPort()}
    desc={m.settingsMemosBindPortDescription()}
    searchId="memos-bind-port"
    searchAliases={aliasesFromLocale(m.settingsMemosBindPortSearchAliases())}
  >
    <input
      id="bindPort"
      type="number"
      min="0"
      max="65535"
      bind:value={input.bindPort}
      onfocusout={() => {
        currentConfig.memos.port = input.bindPort;
      }}
      class="p-2 rounded-md border bg-background min-w-max w-40"
    />
  </Setting>

  <SettingToggle
    name={m.settingsMemosEnvironmentVariables()}
    desc={m.settingsMemosEnvironmentVariablesDescription()}
    searchId="memos-env-vars"
    searchAliases={aliasesFromLocale(m.settingsMemosEnvironmentVariablesSearchAliases())}
    bind:state={input.envVarsEnabled}
    onclick={() => {
      currentConfig.memos.env.enabled = input.envVarsEnabled;
    }}
  >
    <textarea
      id="env"
      rows="5"
      class="p-2 rounded-md border bg-background min-w-max w-full leading-tight"
      bind:value={input.envVars}
      onfocusout={updateEnvVars}
      onkeypress={async (e) => e.key === "Enter" && (await updateEnvVars(e))}
      disabled={!input.envVarsEnabled}
    >
    </textarea>
  </SettingToggle>

</div>
