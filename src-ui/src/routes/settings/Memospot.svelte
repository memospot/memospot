<script lang="ts">
import * as jsonpatch from "fast-json-patch";
import { onMount } from "svelte";
import { toast } from "svelte-sonner";
import { Setting, SettingToggle } from "$lib/components/ui/setting/index";
import { Switch } from "$lib/components/ui/switch/index";
import { debouncePromise } from "$lib/debounce";
import { envFromKV, envToKV } from "$lib/environmentVariables";
import { m } from "$lib/i18n";
import { patchConfig } from "$lib/settings";
import {
    buildSectionActions,
    keywordsFromLocale,
    type SectionActionsProps
} from "$lib/settingsUi";
import { getAppConfig, getDefaultAppConfig, pingMemos } from "$lib/tauri";
import type { Config } from "$lib/types/gen/Config";

let { onActionsChange }: SectionActionsProps = $props();

let initialConfig = $state({}) as Config;
let currentConfig = $state({}) as Config;
let input = $state({
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
});

onMount(async () => {
    const initialJSON = await getAppConfig();
    initialConfig = JSON.parse(initialJSON);
    currentConfig = jsonpatch.deepClone(initialConfig);
    await setPageToInitialConfig();
});

async function setPageToInitialConfig() {
    input = {
        remoteEnabled: initialConfig.memospot.remote.enabled ?? false,
        remoteURL: initialConfig.memospot.remote.url ?? "",
        remoteUserAgent: initialConfig.memospot.remote.user_agent ?? "",
        updaterEnabled: initialConfig.memospot.updater.enabled ?? false,
        updaterCheckInterval: initialConfig.memospot.updater.check_interval ?? "",
        migrationsEnabled: initialConfig.memospot.migrations.enabled ?? false,
        backupsEnabled: initialConfig.memospot.backups.enabled ?? false,
        loggingEnabled: initialConfig.memospot.log.enabled ?? false,
        envVarsEnabled: initialConfig.memospot.env.enabled ?? false,
        envVars: envFromKV((initialConfig.memospot.env.vars ?? {}) as Record<string, string>)
    };

    currentConfig.memospot = jsonpatch.deepClone(initialConfig.memospot);
}

async function setPageToDefaultConfig() {
    const defaultJSON = JSON.parse(await getDefaultAppConfig()) as Config;
    input = {
        remoteEnabled: defaultJSON.memospot.remote.enabled ?? false,
        remoteURL: defaultJSON.memospot.remote.url ?? "",
        remoteUserAgent: defaultJSON.memospot.remote.user_agent ?? "",
        updaterEnabled: defaultJSON.memospot.updater.enabled ?? false,
        updaterCheckInterval: defaultJSON.memospot.updater.check_interval ?? "",
        migrationsEnabled: defaultJSON.memospot.migrations.enabled ?? false,
        backupsEnabled: defaultJSON.memospot.backups.enabled ?? false,
        loggingEnabled: defaultJSON.memospot.log.enabled ?? false,
        envVarsEnabled: defaultJSON.memospot.env.enabled ?? false,
        envVars: envFromKV((defaultJSON.memospot.env.vars ?? {}) as Record<string, string>)
    };

    currentConfig.memospot = jsonpatch.deepClone(defaultJSON.memospot);
}

function syncCurrentConfigFromInput() {
    currentConfig.memospot.remote.enabled = input.remoteEnabled;
    currentConfig.memospot.remote.url = input.remoteURL;
    currentConfig.memospot.remote.user_agent = input.remoteUserAgent;
    currentConfig.memospot.updater.enabled = input.updaterEnabled;
    currentConfig.memospot.updater.check_interval = input.updaterCheckInterval;
    currentConfig.memospot.migrations.enabled = input.migrationsEnabled;
    currentConfig.memospot.backups.enabled = input.backupsEnabled;
    currentConfig.memospot.log.enabled = input.loggingEnabled;
    currentConfig.memospot.env.enabled = input.envVarsEnabled;
    currentConfig.memospot.env.vars = envToKV(input.envVars);
}

async function updateEnvVars(_: Event) {
    const kv = envToKV(input.envVars);
    currentConfig.memospot.env.vars = kv;
    input.envVars = envFromKV(kv);
}

/**
 * Update remote server URL.
 *
 * Checks if the URL is valid.
 */
async function updateRemoteServerUrl(_: Event) {
    let inputURL = input.remoteURL.trim();

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
        } catch (error) {
            toast.error(m.settingsMemospotErrInvalidServer(), {
                duration: 5000
            });
            input.remoteURL = initialConfig.memospot.remote.url ?? "";
            return;
        }
    }

    const intervalMs = 300;
    await debouncePromise(async () => {
        if (inputURL && !(await pingMemos(inputURL))) {
            toast.error(m.settingsMemospotErrInvalidServer(), {
                duration: 5000
            });
            input.remoteURL = initialConfig.memospot.remote.url ?? "";
            return;
        }

        currentConfig.memospot.remote.url = inputURL;
        input.remoteURL = inputURL;
    }, intervalMs)();
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
    if (!currentConfig.memospot) return;
    syncCurrentConfigFromInput();
});

const hasPendingChanges = $derived(
    input.remoteEnabled !== (initialConfig.memospot?.remote?.enabled ?? false) ||
        input.remoteURL !== (initialConfig.memospot?.remote?.url ?? "") ||
        input.remoteUserAgent !== (initialConfig.memospot?.remote?.user_agent ?? "") ||
        input.updaterEnabled !== (initialConfig.memospot?.updater?.enabled ?? false) ||
        input.updaterCheckInterval !==
            (initialConfig.memospot?.updater?.check_interval ?? "") ||
        input.migrationsEnabled !== (initialConfig.memospot?.migrations?.enabled ?? false) ||
        input.backupsEnabled !== (initialConfig.memospot?.backups?.enabled ?? false) ||
        input.loggingEnabled !== (initialConfig.memospot?.log?.enabled ?? false) ||
        input.envVarsEnabled !== (initialConfig.memospot?.env?.enabled ?? false) ||
        input.envVars !==
            envFromKV((initialConfig.memospot?.env?.vars ?? {}) as Record<string, string>)
);

$effect(() => {
    onActionsChange?.(
        buildSectionActions(
            setPageToDefaultConfig,
            setPageToInitialConfig,
            updateSetting,
            hasPendingChanges
        )
    );
});
</script>

<div class="space-y-3">
  <div>
    <h3 class="text-lg mb-1">
      {m.settingsMemospotDescription()}
    </h3>

    <p class="text-sm text-muted-foreground">{m.settingsOverview()}</p>
  </div>

  <SettingToggle
    name={m.settingsMemospotRemoteServer()}
    desc={m.settingsMemospotRemoteServerDescription()}
    searchId="memospot-remote-server"
    searchKeywords={keywordsFromLocale(m.settingsMemospotRemoteServerSearchKeywords)}
    bind:state={input.remoteEnabled}
    onclick={() => {
      currentConfig.memospot.remote.enabled = input.remoteEnabled;
    }}
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
        bind:value={input.remoteURL}
        onfocusout={updateRemoteServerUrl}
        onkeypress={async (e) =>
        e.key === "Enter" && (await updateRemoteServerUrl(e))}
        class="p-2 rounded-md border bg-input min-w-max md:w-96"
        disabled={!input.remoteEnabled}
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
        bind:value={input.remoteUserAgent}
        onfocusout={() => {
          currentConfig.memospot.remote.user_agent = input.remoteUserAgent;
        }}
        class="p-2 rounded-md border bg-input min-w-max md:w-96"
        disabled={!input.remoteEnabled}
      />
    </Setting>
  </SettingToggle>

  <SettingToggle
    name={m.settingsMemospotUpdater()}
    desc={m.settingsMemospotUpdaterDescription()}
    searchId="memospot-updater"
    searchKeywords={keywordsFromLocale(m.settingsMemospotUpdaterSearchKeywords)}
    bind:state={input.updaterEnabled}
    onclick={() => {
      currentConfig.memospot.updater.enabled = input.updaterEnabled;
    }}
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
        bind:value={input.updaterCheckInterval}
        onfocusout={() => {
          currentConfig.memospot.updater.check_interval = input.updaterCheckInterval;
        }}
        class="p-2 rounded-md border bg-input min-w-max md:w-96"
        disabled={!input.updaterEnabled}
      />
    </Setting>
  </SettingToggle>

  <Setting
    name={m.settingsMemospotMigrations()}
    desc={m.settingsMemospotMigrationsDescription()}
    searchId="memospot-migrations"
    searchKeywords={keywordsFromLocale(m.settingsMemospotMigrationsSearchKeywords)}
  >
    <Switch
      bind:checked={input.migrationsEnabled}
      onclick={() => {
        currentConfig.memospot.migrations.enabled = input.migrationsEnabled;
      }}
    />
  </Setting>

  <Setting
    name={m.settingsMemospotBackups()}
    desc={m.settingsMemospotBackupsDescription()}
    searchId="memospot-backups"
    searchKeywords={keywordsFromLocale(m.settingsMemospotBackupsSearchKeywords)}
  >
    <Switch
      bind:checked={input.backupsEnabled}
      onclick={() => {
        currentConfig.memospot.backups.enabled = input.backupsEnabled;
      }}
    />
  </Setting>

  <Setting
    name={m.settingsMemospotLogging()}
    desc={m.settingsMemospotLoggingDescription()}
    searchId="memospot-logging"
    searchKeywords={keywordsFromLocale(m.settingsMemospotLoggingSearchKeywords)}
  >
    <Switch
      bind:checked={input.loggingEnabled}
      onclick={() => {
        currentConfig.memospot.log.enabled = input.loggingEnabled;
      }}
    />
  </Setting>

  <SettingToggle
    name={m.settingsMemospotEnvironmentVariables()}
    desc={m.settingsMemospotEnvironmentVariablesDescription()}
    searchId="memospot-env-vars"
    searchKeywords={keywordsFromLocale(m.settingsMemospotEnvironmentVariablesSearchKeywords)}
    bind:state={input.envVarsEnabled}
    onclick={() => {
      currentConfig.memospot.env.enabled = input.envVarsEnabled;
    }}
  >
    <textarea
      id="env"
      rows="5"
      class="p-2 rounded-md border bg-background min-w-max w-full font-mono leading-tight"
      bind:value={input.envVars}
      onfocusout={updateEnvVars}
      onkeypress={async (e) => e.key === "Enter" && (await updateEnvVars(e))}
      disabled={!input.envVarsEnabled}
    >
    </textarea>
  </SettingToggle>
</div>
