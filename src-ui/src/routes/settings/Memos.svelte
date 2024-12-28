<script lang="ts">
import * as Select from "$lib/components/ui/select";
import { Setting } from "$lib/components/ui/setting/index";
import { m } from "$lib/i18n";
import Code from "svelte-radix/Code.svelte";
import LightningBolt from "svelte-radix/LightningBolt.svelte";
import LockClosed from "svelte-radix/LockClosed.svelte";

let binaryPath: string = $state("");
let workingDirectory: string = $state("");
let dataDirectory: string = $state("");
let bindAddress: string = $state("");
let bindPort: string = $state("");
let envVars: string = $state("");

let mode: string = $state("prod");
const modes = {
	prod: m.memosModeProduction(),
	dev: m.memosModeDevelopment(),
	demo: m.memosModeDemonstration(),
} as const;
let selectedMode = $derived({
	label: modes[mode as keyof typeof modes],
	value: mode,
});
</script>

<div class="space-y-4">
  <div>
    <h3 class="text-lg font-medium">{m.memosDescription()}</h3>
    <p class="text-sm text-muted-foreground">{m.memosOverview()}</p>
  </div>

  <Setting name={m.memosMode()} desc={m.memosModeDescription()}>
    <Select.Root
      selected={selectedMode}
      onSelectedChange={(s) => s && (mode = s.value ?? "prod")}
    >
      <Select.Trigger class="ml-1 min-w-max md:w-64">
        <Select.Value placeholder="Theme" />
      </Select.Trigger>
      <Select.Content>
        <Select.Item value="prod" class="text-primary">
          {modes.prod}<LightningBolt class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </Select.Item>
        <Select.Item value="dev">
          {modes.dev}
          <Code class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </Select.Item>
        <Select.Item value="demo">
          {modes.demo}<LockClosed class="h-[1.2rem] w-[1.2rem] ml-auto" />
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
      bind:value={dataDirectory}
      class="p-2 rounded-md border bg-background min-w-max md:w-96"
    />
  </Setting>

  <Setting name={m.memosBinaryPath()} desc={m.memosBinaryPathDescription()}>
    <input
      id="binaryPath"
      type="text"
      bind:value={binaryPath}
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
      bind:value={workingDirectory}
      class="p-2 rounded-md border bg-background min-w-max md:w-96"
    />
  </Setting>

  <Setting name={m.memosBindAddress()} desc={m.memosBindAddressDescription()}>
    <input
      id="bindAddress"
      type="text"
      bind:value={bindAddress}
      class="p-2 rounded-md border bg-background min-w-max md:w-96"
    />
  </Setting>

  <Setting name={m.memosBindPort()} desc={m.memosBindPortDescription()}>
    <input
      id="bindPort"
      type="number"
      min="0"
      max="65535"
      bind:value={bindPort}
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
      bind:value={envVars}
    ></textarea>
  </Setting>
</div>
