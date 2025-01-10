<script lang="ts">
import * as Select from "$lib/components/ui/select";
import { Setting } from "$lib/components/ui/setting/index";
import { debouncePromise } from "$lib/debounce";
import { envFromKV, envToKV } from "$lib/environmentVariables";
import { m } from "$lib/i18n";
import { patchConfig } from "$lib/settings";
import { getConfig, pathExists } from "$lib/tauri";
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
	envVars: "",
});

onMount(async () => {
	const initialJSON = await getConfig();
	initialConfig = JSON.parse(initialJSON);
	currentConfig = jsonpatch.deepClone(initialConfig);
	input = {
		mode: (currentConfig.memos.mode as string) || "prod",
		binaryPath: (currentConfig.memos.binary_path as string) || "",
		workingDir: (currentConfig.memos.working_dir as string) || "",
		dataDir: (currentConfig.memos.data as string) || "",
		bindAddr: (currentConfig.memos.addr as string) || "",
		bindPort: (currentConfig.memos.port as number) || 0,
		envVars: envFromKV((currentConfig.memos.env as { [key: string]: string }) || {}),
	};
});

const memosMode = {
	prod: m.memosModeProduction(),
	dev: m.memosModeDevelopment(),
	demo: m.memosModeDemonstration(),
} as const;

let selectedMode: Selected<string> = $derived({
	label: memosMode[input.mode as keyof typeof memosMode],
	value: input.mode,
});

async function updateSetting(updateFn?: () => void): Promise<boolean> {
	return await debouncePromise(async () => {
		updateFn?.();
		return await patchConfig(initialConfig, currentConfig).then(
			() => {
				initialConfig = jsonpatch.deepClone(currentConfig);
			},
			() => {
				currentConfig = jsonpatch.deepClone(initialConfig);
			},
		);
	})();
}

async function updateDataDir(e: Event) {
	if (input.dataDir && !(await pathExists(input.dataDir))) {
		toast.error("Path does not exist.");
		input.dataDir = initialConfig.memos.data as string;
		return;
	}
	await updateSetting(() => {
		currentConfig.memos.data = input.dataDir;
	});
}

async function updateMode(s: Selected<string> | undefined) {
	input.mode = s?.value ?? "prod";
	await updateSetting(() => {
		currentConfig.memos.mode = input.mode;
	});
}

async function updateEnvVars(e: Event) {
	const kv = envToKV(input.envVars);
	const ok = await updateSetting(() => {
		currentConfig.memos.env = kv;
	});
	if (ok) input.envVars = envFromKV(kv);
}
</script>

<div class="space-y-4">
  <div>
    <h3 class="text-lg font-medium flex flex-row">
      {m.memosDescription()}<a
        href="https://www.usememos.com/docs/install/runtime-options"
        target="_blank"
      >
        <ExternalLink class="ml-1 mt-1 h-[1.2rem] w-[1.2rem]" />
      </a>
    </h3>

    <p class="text-sm text-muted-foreground">{m.memosOverview()}</p>
  </div>

  <Setting name={m.memosMode()} desc={m.memosModeDescription()}>
    <Select.Root
      selected={selectedMode}
      onSelectedChange={(s) => updateMode(s)}
    >
      <Select.Trigger class="ml-1 min-w-max md:w-64">
        <Select.Value placeholder={m.memosMode()} />
      </Select.Trigger>
      <Select.Content>
        <Select.Item value="prod" class="text-primary">
          {memosMode.prod}<LightningBolt class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </Select.Item>
        <Select.Item value="dev">
          {memosMode.dev}
          <Code class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </Select.Item>
        <Select.Item value="demo">
          {memosMode.demo}<LockClosed class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </Select.Item>
      </Select.Content>
    </Select.Root>
  </Setting>

  <Setting
    name={m.memosDataDirectory()}
    desc={m.memosDataDirectoryDescription()}
  >
    <input
      id="dataDirectory"
      type="text"
      bind:value={input.dataDir}
      onfocusout={updateDataDir}
      onkeypress={(e) => e.key === "Enter" && updateDataDir(e)}
      class="p-2 rounded-md border bg-background min-w-max md:w-96"
    />
  </Setting>

  <Setting name={m.memosBinaryPath()} desc={m.memosBinaryPathDescription()}>
    <input
      id="binaryPath"
      type="text"
      bind:value={input.binaryPath}
      onfocusout={() =>
        updateSetting(
          () => (currentConfig.memos.binary_path = input.binaryPath),
        )}
      onkeypress={(e) =>
        e.key === "Enter" &&
        updateSetting(
          () => (currentConfig.memos.binary_path = input.binaryPath),
        )}
      class="p-2 rounded-md border bg-background min-w-max md:w-96"
    />
  </Setting>

  <Setting
    name={m.memosWorkingDirectory()}
    desc={m.memosWorkingDirectoryDescription()}
  >
    <input
      id="workingDirectory"
      type="text"
      bind:value={input.workingDir}
      onfocusout={() =>
        updateSetting(
          () => (currentConfig.memos.working_dir = input.workingDir),
        )}
      onkeypress={(e) =>
        e.key === "Enter" &&
        updateSetting(
          () => (currentConfig.memos.working_dir = input.workingDir),
        )}
      class="p-2 rounded-md border bg-background min-w-max md:w-96"
    />
  </Setting>

  <Setting name={m.memosBindAddress()} desc={m.memosBindAddressDescription()}>
    <input
      id="bindAddress"
      type="text"
      bind:value={input.bindAddr}
      onfocusout={() =>
        updateSetting(() => (currentConfig.memos.addr = input.bindAddr))}
      onkeypress={(e) =>
        e.key === "Enter" &&
        updateSetting(() => (currentConfig.memos.addr = input.bindAddr))}
      class="p-2 rounded-md border bg-background min-w-max md:w-96"
    />
  </Setting>

  <Setting name={m.memosBindPort()} desc={m.memosBindPortDescription()}>
    <input
      id="bindPort"
      type="number"
      min="0"
      max="65535"
      bind:value={input.bindPort}
      onfocusout={() =>
        updateSetting(() => (currentConfig.memos.port = input.bindPort))}
      onkeypress={(e) =>
        e.key === "Enter" &&
        updateSetting(() => (currentConfig.memos.port = input.bindPort))}
      class="p-2 rounded-md border bg-background min-w-max w-40"
    />
  </Setting>

  <Setting
    name={m.memosEnvironmentVariables()}
    desc={m.memosEnvironmentVariablesDescription()}
  >
    <textarea
      id="env"
      rows="3"
      class="p-2 rounded-md border bg-background min-w-max md:w-96"
      bind:value={input.envVars}
      onfocusout={updateEnvVars}
      onkeypress={(e) => e.key === "Enter" && updateEnvVars(e)}
    >
    </textarea>
  </Setting>
</div>
