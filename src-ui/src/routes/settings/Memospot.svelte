<script lang="ts">
import { onMount } from "svelte";
import { toast } from "svelte-sonner";
import { Setting, SettingToggle } from "$lib/components/ui/setting/index";
import { Switch } from "$lib/components/ui/switch/index";
import { envFromKV, envToKV } from "$lib/environmentVariables";
import { m } from "$lib/i18n";
import { createSettingsSection } from "$lib/settingsSection";
import {
    buildSectionActions,
    keywordsFromLocale,
    type SectionActionsProps
} from "$lib/settingsUi";
import { pingMemos } from "$lib/tauri";
import type { Config } from "$lib/types/gen/Config";

let { onActionsChange }: SectionActionsProps = $props();

type MemospotInput = {
    remoteEnabled: boolean;
    remoteURL: string;
    remoteUserAgent: string;
    updaterEnabled: boolean;
    updaterCheckInterval: string;
    migrationsEnabled: boolean;
    backupsEnabled: boolean;
    loggingEnabled: boolean;
    envVarsEnabled: boolean;
    envVars: string;
};

const section = $state(
    createSettingsSection<MemospotInput>({
        mapping: {
            initial: {
                remoteEnabled: false,
                remoteURL: "",
                remoteUserAgent: "",
                updaterEnabled: false,
                updaterCheckInterval: "",
                migrationsEnabled: false,
                backupsEnabled: false,
                loggingEnabled: false,
                envVarsEnabled: false,
                envVars: ""
            },
            mapToInput: (cfg: Config) => ({
                remoteEnabled: cfg.memospot.remote.enabled ?? false,
                remoteURL: cfg.memospot.remote.url ?? "",
                remoteUserAgent: cfg.memospot.remote.user_agent ?? "",
                updaterEnabled: cfg.memospot.updater.enabled ?? false,
                updaterCheckInterval: cfg.memospot.updater.check_interval ?? "",
                migrationsEnabled: cfg.memospot.migrations.enabled ?? false,
                backupsEnabled: cfg.memospot.backups.enabled ?? false,
                loggingEnabled: cfg.memospot.log.enabled ?? false,
                envVarsEnabled: cfg.memospot.env.enabled ?? false,
                envVars: envFromKV((cfg.memospot.env.vars ?? {}) as Record<string, string>)
            }),
            mapFromInput: (input: MemospotInput, cfg: Config) => {
                cfg.memospot.remote.enabled = input.remoteEnabled;
                cfg.memospot.remote.url = input.remoteURL;
                cfg.memospot.remote.user_agent = input.remoteUserAgent;
                cfg.memospot.updater.enabled = input.updaterEnabled;
                cfg.memospot.updater.check_interval = input.updaterCheckInterval;
                cfg.memospot.migrations.enabled = input.migrationsEnabled;
                cfg.memospot.backups.enabled = input.backupsEnabled;
                cfg.memospot.log.enabled = input.loggingEnabled;
                cfg.memospot.env.enabled = input.envVarsEnabled;
                cfg.memospot.env.vars = envToKV(input.envVars);
            }
        }
    })
);

onMount(async () => {
    await section.init();
});

async function updateEnvVars(_: Event) {
    const kv = envToKV(section.input.envVars);
    section.input.envVars = envFromKV(kv);
}

async function updateRemoteServerUrl(_: Event) {
    let inputURL = section.input.remoteURL.trim();

    if (inputURL.length > 0) {
        if (!inputURL.startsWith("http://") && !inputURL.startsWith("https://")) {
            inputURL = inputURL.replace(":/", "").replaceAll("/", "");
            inputURL = `https://${inputURL}`;
        }
        if (!inputURL.endsWith("/")) {
            inputURL += "/";
        }
        try {
            const url = new URL(inputURL);
            if (url.protocol !== "http:" && url.protocol !== "https:") {
                throw new Error();
            }
        } catch {
            toast.error(m.settingsMemospotErrInvalidServer(), {
                duration: 5000
            });
            section.input.remoteURL = section.baselineInput.remoteURL as string;
            return;
        }
    }

    if (inputURL && !(await pingMemos(inputURL))) {
        toast.error(m.settingsMemospotErrInvalidServer(), {
            duration: 5000
        });
        section.input.remoteURL = section.baselineInput.remoteURL as string;
        return;
    }

    section.input.remoteURL = inputURL;
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
    <h3 class="font-semibold uppercase tracking-[0.09rem] text-sm text-foreground mb-1">
      {m.settingsMemospotDescription()}
    </h3>

    <p class="text-sm text-muted-foreground">{m.settingsOverview()}</p>
  </div>

  <SettingToggle
    name={m.settingsMemospotRemoteServer()}
    desc={m.settingsMemospotRemoteServerDescription()}
    searchId="memospot-remote-server"
    searchKeywords={keywordsFromLocale(m.settingsMemospotRemoteServerSearchKeywords)}
    bind:state={section.input.remoteEnabled}
  >
    <Setting
      name={m.settingsMemospotRemoteServerURL()}
      desc={m.settingsMemospotRemoteServerURLDescription()}
      searchId="memospot-remote-url"
      searchKeywords={keywordsFromLocale(m.settingsMemospotRemoteServerURLSearchKeywords)}
    >
      <input
        id="url"
        type="url"
        bind:value={section.input.remoteURL}
        onfocusout={updateRemoteServerUrl}
        onkeypress={async (e) => e.key === "Enter" && (await updateRemoteServerUrl(e))}
        class="font-mono p-2 rounded-md border bg-input min-w-max md:w-96"
        disabled={!section.input.remoteEnabled}
      />
    </Setting>
    <Setting
      name={m.settingsMemospotUserAgent()}
      desc={m.settingsMemospotUserAgentDescription()}
      searchId="memospot-user-agent"
      searchKeywords={keywordsFromLocale(m.settingsMemospotUserAgentSearchKeywords)}
    >
      <input
        id="userAgent"
        type="text"
        bind:value={section.input.remoteUserAgent}
        class="font-mono p-2 rounded-md border bg-input min-w-max md:w-96"
        disabled={!section.input.remoteEnabled}
      />
    </Setting>
  </SettingToggle>

  <SettingToggle
    name={m.settingsMemospotUpdater()}
    desc={m.settingsMemospotUpdaterDescription()}
    searchId="memospot-updater"
    searchKeywords={keywordsFromLocale(m.settingsMemospotUpdaterSearchKeywords)}
    bind:state={section.input.updaterEnabled}
  >
    <Setting
      name={m.settingsMemospotUpdaterInterval()}
      desc={m.settingsMemospotUpdaterIntervalDescription()}
      searchId="memospot-updater-interval"
      searchKeywords={keywordsFromLocale(m.settingsMemospotUpdaterIntervalSearchKeywords)}
    >
      <input
        id="updaterCheckInterval"
        type="text"
        bind:value={section.input.updaterCheckInterval}
        class="font-mono p-2 rounded-md border bg-input min-w-max md:w-96"
        disabled={!section.input.updaterEnabled}
      />
    </Setting>
  </SettingToggle>

  <Setting
    name={m.settingsMemospotMigrations()}
    desc={m.settingsMemospotMigrationsDescription()}
    searchId="memospot-migrations"
    searchKeywords={keywordsFromLocale(m.settingsMemospotMigrationsSearchKeywords)}
  >
    <Switch bind:checked={section.input.migrationsEnabled} />
  </Setting>

  <Setting
    name={m.settingsMemospotBackups()}
    desc={m.settingsMemospotBackupsDescription()}
    searchId="memospot-backups"
    searchKeywords={keywordsFromLocale(m.settingsMemospotBackupsSearchKeywords)}
  >
    <Switch bind:checked={section.input.backupsEnabled} />
  </Setting>

  <Setting
    name={m.settingsMemospotLogging()}
    desc={m.settingsMemospotLoggingDescription()}
    searchId="memospot-logging"
    searchKeywords={keywordsFromLocale(m.settingsMemospotLoggingSearchKeywords)}
  >
    <Switch bind:checked={section.input.loggingEnabled} />
  </Setting>

  <SettingToggle
    name={m.settingsMemospotEnvironmentVariables()}
    desc={m.settingsMemospotEnvironmentVariablesDescription()}
    searchId="memospot-env-vars"
    searchKeywords={keywordsFromLocale(m.settingsMemospotEnvironmentVariablesSearchKeywords)}
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
