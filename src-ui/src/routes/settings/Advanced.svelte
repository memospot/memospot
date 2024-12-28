<script lang="ts">
import { Setting, SettingToggle } from "$lib/components/ui/setting/index";
import { Switch } from "$lib/components/ui/switch/index";
import { m } from "$lib/i18n";

// Button states.
let stateRemoteServer: boolean = $state(false);
let stateUpdater: boolean = $state(false);
let stateMigrations: boolean = $state(false);
let stateBackups: boolean = $state(false);
let stateLogging: boolean = $state(false);
let stateEnvVars: boolean = $state(false);

let url: string = $state("https://demo.usememos.com");
let userAgent: string = $state(
	"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
);
</script>

<div class="space-y-4">
  <div>
    <h3 class="text-lg font-medium">
      {m.advancedDescription()}
    </h3>
  </div>

  <SettingToggle
    name={m.advancedRemoteServer()}
    desc={m.advancedRemoteServerDescription()}
    bind:state={stateRemoteServer}
  >
    <Setting
      name={m.advancedRemoteServerURL()}
      desc={m.advancedRemoteServerURLDescription()}
    >
      <input
        id="url"
        type="url"
        bind:value={url}
        class="p-2 rounded-md border bg-input min-w-max md:w-96"
        disabled={!stateRemoteServer}
      />
    </Setting>
    <Setting
      name={m.advancedUserAgent()}
      desc={m.advancedUserAgentDescription()}
    >
      <input
        id="userAgent"
        type="text"
        bind:value={userAgent}
        class="p-2 rounded-md border bg-input min-w-max md:w-96"
        disabled={!stateRemoteServer}
      />
    </Setting>
  </SettingToggle>

  <Setting name={m.advancedUpdater()} desc={m.advancedUpdaterDescription()}>
    <Switch bind:state={stateUpdater} />
  </Setting>

  <Setting
    name={m.advancedMigrations()}
    desc={m.advancedMigrationsDescription()}
  >
    <Switch bind:state={stateMigrations} />
  </Setting>

  <Setting name={m.advancedBackups()} desc={m.advancedBackupsDescription()}>
    <Switch bind:state={stateBackups} />
  </Setting>

  <Setting name={m.advancedLogging()} desc={m.advancedLoggingDescription()}>
    <Switch bind:state={stateLogging} />
  </Setting>

  <SettingToggle
    name={m.advancedEnvironmentVariables()}
    desc={m.advancedEnvironmentVariablesDescription()}
    bind:state={stateEnvVars}
  >
    <textarea
      id="env"
      rows="3"
      class="p-2 rounded-md border bg-background min-w-max w-full"
      disabled={!stateEnvVars}
    >
    </textarea>
  </SettingToggle>
</div>
