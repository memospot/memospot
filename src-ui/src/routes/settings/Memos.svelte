<script lang="ts">
import type { Selected } from "bits-ui";
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
import { envFromKV, envToKV } from "$lib/environmentVariables";
import { m } from "$lib/i18n";
import { createSettingsSection } from "$lib/settingsSection";
import {
    buildSectionActions,
    keywordsFromLocale,
    type SectionActionsProps
} from "$lib/settingsUi";
import { pathExists } from "$lib/tauri";
import type { Config } from "$lib/types/gen/Config";

let { onActionsChange }: SectionActionsProps = $props();

type MemosInput = {
    mode: string;
    binaryPath: string;
    workingDir: string;
    dataDir: string;
    bindAddr: string;
    bindPort: number;
    envVarsEnabled: boolean;
    envVars: string;
};

const section = $state(
    createSettingsSection<MemosInput>({
        mapping: {
            initial: {
                mode: "prod",
                binaryPath: "",
                workingDir: "",
                dataDir: "",
                bindAddr: "",
                bindPort: 0,
                envVarsEnabled: false,
                envVars: ""
            },
            mapToInput: (cfg: Config) => ({
                mode: cfg.memos.mode ?? "prod",
                binaryPath: cfg.memos.binary_path ?? "",
                workingDir: cfg.memos.working_dir ?? "",
                dataDir: cfg.memos.data ?? "",
                bindAddr: cfg.memos.addr ?? "",
                bindPort: cfg.memos.port ?? 0,
                envVarsEnabled: cfg.memos.env.enabled ?? false,
                envVars: envFromKV((cfg.memos.env.vars ?? {}) as Record<string, string>)
            }),
            mapFromInput: (input: MemosInput, cfg: Config) => {
                cfg.memos.mode = input.mode;
                cfg.memos.binary_path = input.binaryPath;
                cfg.memos.working_dir = input.workingDir;
                cfg.memos.data = input.dataDir;
                cfg.memos.addr = input.bindAddr;
                cfg.memos.port = input.bindPort;
                cfg.memos.env.enabled = input.envVarsEnabled;
                cfg.memos.env.vars = envToKV(input.envVars);
            }
        }
    })
);

const memosModeNames = {
    prod: m.settingsMemosModeProduction(),
    dev: m.settingsMemosModeDevelopment(),
    demo: m.settingsMemosModeDemonstration()
} as const;

let selectedMode: Selected<string> = $derived({
    label: memosModeNames[section.input.mode as keyof typeof memosModeNames],
    value: section.input.mode
});

const reduceAnimation = $derived(
    JSON.parse(localStorage.getItem("reduce-animation") ?? "false")
);

onMount(async () => {
    await section.init();
});

async function setMemosMode(s: Selected<string> | undefined) {
    section.input.mode = s?.value ?? "prod";
}

async function validateMemosDataDir(e: Event | KeyboardEvent) {
    if (e.type === "keypress" && (e as KeyboardEvent).key !== "Enter") return;

    await validatePath(e).catch(() => {
        section.input.dataDir = section.baselineInput.dataDir;
    });
}

async function validateMemosBinaryPath(e: Event | KeyboardEvent) {
    if (e.type === "keypress" && (e as KeyboardEvent).key !== "Enter") return;

    await validatePath(e).catch(() => {
        section.input.binaryPath = section.baselineInput.binaryPath;
    });
}

async function validateMemosWorkingDir(e: Event | KeyboardEvent) {
    if (e.type === "keypress" && (e as KeyboardEvent).key !== "Enter") return;

    await validatePath(e).catch(() => {
        section.input.workingDir = section.baselineInput.workingDir;
    });
}

async function validatePath(e: Event) {
    const inputEl = e.target as HTMLInputElement;
    if (!inputEl.value || (await pathExists(inputEl.value))) {
        return Promise.resolve();
    }
    toast.error(m.settingsErrPathDoesNotExist());
    return Promise.reject();
}

async function updateEnvVars(_: Event) {
    const kv = envToKV(section.input.envVars);
    section.input.envVars = envFromKV(kv);
}

$effect(() => {
    onActionsChange?.(
        buildSectionActions(
            () => section.loadDefaults(),
            () => section.reset(),
            () => section.save(),
            section.hasPendingChanges
        )
    );
});
</script>

<div class="space-y-3">
  <div class="mb-4">
    <h3 class="font-semibold uppercase tracking-[0.09rem] text-sm text-foreground flex flex-row mb-1">
      {m.settingsMemosDescription()}<a
        href="https://usememos.com/docs/configuration#common-options"
        target="_blank"
      >
        <ExternalLink class="ml-1 h-[1.2rem] w-[1.2rem]" />
      </a>
    </h3>

    <p class="text-sm text-muted-foreground">{m.settingsOverview()}</p>
  </div>

  <Setting
    name={m.settingsMemosMode()}
    desc={m.settingsMemosModeDescription()}
    searchId="memos-mode"
    searchKeywords={keywordsFromLocale(m.settingsMemosModeSearchKeywords)}
  >
    <Select portal={null} selected={selectedMode} onSelectedChange={setMemosMode}>
      <SelectTrigger class="ml-1 min-w-max md:w-64">
        <SelectValue placeholder={m.settingsMemosMode()} />
      </SelectTrigger>
      <SelectContent {reduceAnimation}>
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
    searchKeywords={keywordsFromLocale(m.settingsMemosDataDirectorySearchKeywords)}
  >
    <input
      id="dataDirectory"
      type="text"
      bind:value={section.input.dataDir}
      onfocusout={validateMemosDataDir}
      onkeypress={validateMemosDataDir}
      class="font-mono p-2 rounded-md border bg-input min-w-max md:w-96"
    />
  </Setting>

  <Setting
    name={m.settingsMemosBinaryPath()}
    desc={m.settingsMemosBinaryPathDescription()}
    searchId="memos-binary-path"
    searchKeywords={keywordsFromLocale(m.settingsMemosBinarySearchKeywords)}
  >
    <input
      id="binaryPath"
      type="text"
      bind:value={section.input.binaryPath}
      onfocusout={validateMemosBinaryPath}
      onkeypress={validateMemosBinaryPath}
      class="font-mono p-2 rounded-md border bg-input min-w-max md:w-96"
    />
  </Setting>

  <Setting
    name={m.settingsMemosWorkingDirectory()}
    desc={m.settingsMemosWorkingDirectoryDescription()}
    searchId="memos-working-directory"
    searchKeywords={keywordsFromLocale(m.settingsMemosWorkingDirectorySearchKeywords)}
  >
    <input
      id="workingDirectory"
      type="text"
      bind:value={section.input.workingDir}
      onfocusout={validateMemosWorkingDir}
      onkeypress={validateMemosWorkingDir}
      class="font-mono p-2 rounded-md border bg-input min-w-max md:w-96"
    />
  </Setting>

  <Setting
    name={m.settingsMemosBindAddress()}
    desc={m.settingsMemosBindAddressDescription()}
    searchId="memos-bind-address"
    searchKeywords={keywordsFromLocale(m.settingsMemosBindAddressSearchKeywords)}
  >
    <input
      id="bindAddress"
      type="text"
      bind:value={section.input.bindAddr}
      class="font-mono p-2 rounded-md border bg-input min-w-max md:w-96"
    />
  </Setting>

  <Setting
    name={m.settingsMemosBindPort()}
    desc={m.settingsMemosBindPortDescription()}
    searchId="memos-bind-port"
    searchKeywords={keywordsFromLocale(m.settingsMemosBindPortSearchKeywords)}
  >
    <input
      id="bindPort"
      type="number"
      min="0"
      max="65535"
      bind:value={section.input.bindPort}
      class="font-mono p-2 rounded-md border bg-input min-w-max w-40"
    />
  </Setting>

  <SettingToggle
    name={m.settingsMemosEnvironmentVariables()}
    desc={m.settingsMemosEnvironmentVariablesDescription()}
    searchId="memos-env-vars"
    searchKeywords={keywordsFromLocale(m.settingsMemosEnvironmentVariablesSearchKeywords)}
    bind:state={section.input.envVarsEnabled}
  >
    <textarea
      id="env"
      rows="5"
      class="p-2 rounded-md border bg-input min-w-max w-full font-mono leading-tight"
      bind:value={section.input.envVars}
      onfocusout={updateEnvVars}
      onkeypress={async (e) => e.key === "Enter" && (await updateEnvVars(e))}
      disabled={!section.input.envVarsEnabled}
    >
    </textarea>
  </SettingToggle>
</div>
