<script lang="ts">
import { Setting, SettingToggle } from "$lib/components/ui/setting/index";
import { Switch } from "$lib/components/ui/switch/index";
import { debouncePromise } from "$lib/debounce";
import { envFromKV, envToKV } from "$lib/environmentVariables";
import { m } from "$lib/i18n";
import { patchConfig } from "$lib/settings";
import { getAppConfig, getDefaultAppConfig, pingMemos } from "$lib/tauri";
import type { Config } from "$lib/types/gen/Config";
import * as jsonpatch from "fast-json-patch";
import { onMount } from "svelte";
import { toast } from "svelte-sonner";

let initialConfig = $state({}) as Config;
let currentConfig = $state({}) as Config;
let input = $state({
	remoteEnabled: false,
	remoteURL: "",
	remoteUserAgent: "",
	updaterEnabled: false,
	migrationsEnabled: false,
	backupsEnabled: false,
	loggingEnabled: false,
	envVarsEnabled: false,
	envVars: "",
});

onMount(async () => {
	const initialJSON = await getAppConfig();
	initialConfig = JSON.parse(initialJSON);
	currentConfig = jsonpatch.deepClone(initialConfig);
	await setPageToInitialConfig();
});

async function setPageToInitialConfig() {
	input = {
		remoteEnabled: (initialConfig.memospot.remote.enabled as boolean) || false,
		remoteURL: (initialConfig.memospot.remote.url as string) || "",
		remoteUserAgent: (initialConfig.memospot.remote.user_agent as string) || "",
		updaterEnabled: (initialConfig.memospot.updater.enabled as boolean) || false,
		migrationsEnabled: (initialConfig.memospot.migrations.enabled as boolean) || false,
		backupsEnabled: (initialConfig.memospot.backups.enabled as boolean) || false,
		loggingEnabled: (initialConfig.memospot.log.enabled as boolean) || false,
		envVarsEnabled: (initialConfig.memospot.env.enabled as boolean) || false,
		envVars: envFromKV((initialConfig.memospot.env.vars as Record<string, string>) || {}),
	};

	currentConfig.memospot = jsonpatch.deepClone(initialConfig.memospot);
}

async function setPageToDefaultConfig() {
	const defaultJSON = JSON.parse(await getDefaultAppConfig()) as Config;
	input = {
		remoteEnabled: (defaultJSON.memospot.remote.enabled as boolean) || false,
		remoteURL: (defaultJSON.memospot.remote.url as string) || "",
		remoteUserAgent: (defaultJSON.memospot.remote.user_agent as string) || "",
		updaterEnabled: (defaultJSON.memospot.updater.enabled as boolean) || false,
		migrationsEnabled: (defaultJSON.memospot.migrations.enabled as boolean) || false,
		backupsEnabled: (defaultJSON.memospot.backups.enabled as boolean) || false,
		loggingEnabled: (defaultJSON.memospot.log.enabled as boolean) || false,
		envVarsEnabled: (defaultJSON.memospot.env.enabled as boolean) || false,
		envVars: envFromKV((defaultJSON.memospot.env.vars as Record<string, string>) || {}),
	};

	currentConfig.memospot = jsonpatch.deepClone(defaultJSON.memospot);
}

async function updateEnvVars(e: Event) {
	const kv = envToKV(input.envVars);
	currentConfig.memospot.env.vars = kv;
	input.envVars = envFromKV(kv);
}

/**
 * Update remote server URL.
 *
 * Checks if the URL is valid.
 *
 * @param e
 */
async function updateRemoteServerUrl(e: Event) {
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
				duration: 5000,
			});
			input.remoteURL = initialConfig.memospot.remote.url || "";
			return;
		}
	}

	const intervalMs = 300;
	await debouncePromise(async () => {
		if (inputURL && !(await pingMemos(inputURL))) {
			toast.error(m.settingsMemospotErrInvalidServer(), {
				duration: 5000,
			});
			input.remoteURL = initialConfig.memospot.remote.url || "";
			return;
		}

		currentConfig.memospot.remote.url = inputURL;
		input.remoteURL = inputURL;
	}, intervalMs)();
}

async function updateSetting(updateFn?: () => void): Promise<boolean> {
	return await debouncePromise(async () => {
		updateFn?.();
		return await patchConfig(initialConfig, currentConfig).then(
			(_ok) => {
				initialConfig = jsonpatch.deepClone(currentConfig);
			},
			(_err) => {
				currentConfig = jsonpatch.deepClone(initialConfig);
			},
		);
	})();
}
</script>

<div class="space-y-4">
  <div>
    <h3 class="text-lg font-medium">
      {m.settingsMemospotDescription()}
    </h3>

    <p class="text-sm text-muted-foreground">{m.settingsOverview()}</p>
  </div>

  <SettingToggle
    name={m.settingsMemospotRemoteServer()}
    desc={m.settingsMemospotRemoteServerDescription()}
    bind:state={input.remoteEnabled}
    onclick={() => {
      currentConfig.memospot.remote.enabled = input.remoteEnabled;
    }}
  >
    <Setting
      name={m.settingsMemospotRemoteServerURL()}
      desc={m.settingsMemospotRemoteServerURLDescription()}
    >
      <input
        id="url"
        type="url"
        bind:value={input.remoteURL}
        onfocusout={updateRemoteServerUrl}
        onkeypress={async (e) => e.key === "Enter" && await updateRemoteServerUrl(e)}
        class="p-2 rounded-md border bg-input min-w-max md:w-96"
        disabled={!input.remoteEnabled}
      />
    </Setting>
    <Setting
      name={m.settingsMemospotUserAgent()}
      desc={m.settingsMemospotUserAgentDescription()}
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

  <Setting name={m.settingsMemospotUpdater()} desc={m.settingsMemospotUpdaterDescription()}>
    <Switch
      bind:checked={input.updaterEnabled}
      onclick={() => {
        currentConfig.memospot.updater.enabled = input.updaterEnabled;
      }}
    />
  </Setting>

  <Setting
    name={m.settingsMemospotMigrations()}
    desc={m.settingsMemospotMigrationsDescription()}
  >
    <Switch
      bind:checked={input.migrationsEnabled}
      onclick={() => {
        currentConfig.memospot.migrations.enabled = input.migrationsEnabled;
      }}
    />
  </Setting>

  <Setting name={m.settingsMemospotBackups()} desc={m.settingsMemospotBackupsDescription()}>
    <Switch
      bind:checked={input.backupsEnabled}
      onclick={() => {
        currentConfig.memospot.backups.enabled = input.backupsEnabled;
      }}
    />
  </Setting>

  <Setting name={m.settingsMemospotLogging()} desc={m.settingsMemospotLoggingDescription()}>
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
    bind:state={input.envVarsEnabled}
    onclick={() => {
      currentConfig.memospot.env.enabled = input.envVarsEnabled;
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
      class="border-box inline-flex items-center justify-center rounded-md bg-secondary text-base px-4 py-2 h-10 cursor-pointer hover:opacity-80 hover:translate-y-[-1px]"
      type="button"
      onclick={async () => await setPageToDefaultConfig()}
      >{m.settingsLoadDefaults()}</button
    >
    <button
      class="border-box inline-flex items-center justify-center rounded-md bg-secondary text-base px-4 py-2 h-10 cursor-pointer hover:opacity-80 hover:translate-y-[-1px]"
      type="button"
      onclick={async () => await setPageToInitialConfig()}
      >{m.settingsReloadCurrent()}</button
    >
    <button
      class="border-box inline-flex items-center justify-center rounded-md bg-primary text-zinc-50 text-base px-4 py-2 h-10 cursor-pointer hover:opacity-80 hover:translate-y-[-1px] [text-shadow:_0_1px_0_rgb(0_0_0_/_40%)]"
      type="button"
      onclick={async () => await updateSetting()}>{m.settingsSave()}</button
    >
  </div>
</div>
