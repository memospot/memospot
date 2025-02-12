<script lang="ts">
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
import { getAppConfig, getDefaultAppConfig, pathExists } from "$lib/tauri";
import type { Config } from "$lib/types/gen/Config";
import type { Selected } from "bits-ui";
import * as jsonpatch from "fast-json-patch";
import { onMount } from "svelte";
import Code from "svelte-radix/Code.svelte";
import ExternalLink from "svelte-radix/ExternalLink.svelte";
import LightningBolt from "svelte-radix/LightningBolt.svelte";
import LockClosed from "svelte-radix/LockClosed.svelte";
import { toast } from "svelte-sonner";

let initialConfig = $state({}) as Config;
let currentConfig = $state({}) as Config;
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
});

async function setPageToInitialConfig() {
    input = {
        mode: (initialConfig.memos.mode as string) || "prod",
        binaryPath: (initialConfig.memos.binary_path as string) || "",
        workingDir: (initialConfig.memos.working_dir as string) || "",
        dataDir: (initialConfig.memos.data as string) || "",
        bindAddr: (initialConfig.memos.addr as string) || "",
        bindPort: (initialConfig.memos.port as number) || 0,
        envVarsEnabled: (initialConfig.memos.env.enabled as boolean) || false,
        envVars: envFromKV((initialConfig.memos.env.vars as Record<string, string>) || {})
    };

    currentConfig.memos = jsonpatch.deepClone(initialConfig.memos);
}

async function setPageToDefaultConfig() {
    const defaultJSON = JSON.parse(await getDefaultAppConfig()) as Config;
    input = {
        mode: (defaultJSON.memos.mode as string) || "prod",
        binaryPath: (defaultJSON.memos.binary_path as string) || "",
        workingDir: (defaultJSON.memos.working_dir as string) || "",
        dataDir: (defaultJSON.memos.data as string) || "",
        bindAddr: (defaultJSON.memos.addr as string) || "",
        bindPort: (defaultJSON.memos.port as number) || 0,
        envVarsEnabled: (defaultJSON.memos.env.enabled as boolean) || false,
        envVars: envFromKV((defaultJSON.memos.env.vars as Record<string, string>) || {})
    };

    currentConfig.memos = jsonpatch.deepClone(defaultJSON.memos);
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
</script>

<div class="space-y-4">
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

  <Setting name={m.settingsMemosMode()} desc={m.settingsMemosModeDescription()}>
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

  <div class="flex flex-row space-x-1 justify-end">
    <button
      tabindex="-1"
      class="border-box inline-flex items-center justify-center rounded-md bg-secondary text-base px-4 py-2 h-10 cursor-pointer hover:opacity-80 hover:translate-y-[-1px]"
      type="button"
      onclick={async () => await setPageToDefaultConfig()}
    >
      {m.settingsLoadDefaults()}
    </button>
    <button
      tabindex="-1"
      class="border-box inline-flex items-center justify-center rounded-md bg-secondary text-base px-4 py-2 h-10 cursor-pointer hover:opacity-80 hover:translate-y-[-1px]"
      type="button"
      onclick={async () => await setPageToInitialConfig()}
    >
      {m.settingsReloadCurrent()}
    </button>
    <button
      tabindex="0"
      class="border-box inline-flex items-center justify-center rounded-md bg-primary text-zinc-50 text-base px-4 py-2 h-10 cursor-pointer hover:opacity-80 hover:translate-y-[-1px] [text-shadow:_1px_1px_0_rgb(0_0_0_/_90%)]"
      type="button"
      onclick={async () => await updateSetting()}
    >
      {m.settingsSave()}
    </button>
  </div>
</div>
