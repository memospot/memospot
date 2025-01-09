<script lang="ts">
import { Setting, SettingToggle } from "$lib/components/ui/setting/index";
import { Toaster } from "$lib/components/ui/sonner";
import { Switch } from "$lib/components/ui/switch/index";
import { debouncePromise } from "$lib/debounce";
import { envFromKV, envToKV } from "$lib/environmentVariables";
import { m } from "$lib/i18n";
import { patchConfig } from "$lib/settings";
import { getConfig, pingMemos, setConfig } from "$lib/tauri";
import * as jsonpatch from "fast-json-patch";
import { onMount } from "svelte";
import { toast } from "svelte-sonner";

let initialConfig: any = $state({});
let currentConfig: any = $state({});
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

async function updateEnvVars(e: Event) {
	const inputText = (e.target as HTMLInputElement).value;
	const kv = envToKV(inputText);

	currentConfig.memospot.env.vars = kv;
	if (await updateSetting()) {
		initialConfig.memospot.env.vars = kv;
		input.envVars = envFromKV(kv);
	} else {
		currentConfig.memospot.env.vars = initialConfig.memospot.env.vars;
	}
}

/**
 * Update remote server URL.
 *
 * Checks if the URL is valid.
 *
 * @param e
 */
async function updateRemoteServerUrl(e: Event) {
	if (!e || !e.target) return;
	let inputText = (e.target as HTMLInputElement).value.trim();
	if (inputText.length > 0) {
		if (!inputText.startsWith("http://") && !inputText.startsWith("https://")) {
			inputText = inputText.replace(":/", "").replaceAll("/", "");
			inputText = `https://${inputText}`;
		}
		if (!inputText.endsWith("/")) {
			inputText += "/";
		}
		try {
			const url = new URL(inputText);
			if (url.protocol !== "http:" && url.protocol !== "https:") {
				throw new Error();
			}
		} catch (error) {
			toast.error("Invalid URL. Value restored.");
			(e.target as HTMLInputElement).value = initialConfig.memospot.remote.url;
			return;
		}
	}

	const intervalMs = 300;
	await debouncePromise(async () => {
		if (inputText && !(await pingMemos(inputText))) {
			// toast.error(m.memospotRemoteServerURLInvalid());
			toast.error("No Memos server found. Value restored.", {
				duration: 5000,
			});
			(e.target as HTMLInputElement).value = initialConfig.memospot.remote.url;
			return;
		}

		currentConfig.memospot.remote.url = inputText;
		if (await updateSetting()) {
			initialConfig.memospot.remote.url = inputText;
		} else {
			currentConfig.memospot.remote.url = initialConfig.memospot.remote.url;
		}
		input.remoteURL = currentConfig.memospot.remote.url;
	}, intervalMs)();
}

onMount(async () => {
	const initialJSON = await getConfig();
	initialConfig = JSON.parse(initialJSON);
	currentConfig = jsonpatch.deepClone(initialConfig);

	input = {
		remoteEnabled: currentConfig.memospot.remote.enabled,
		remoteURL: currentConfig.memospot.remote.url,
		remoteUserAgent: currentConfig.memospot.remote.user_agent,
		updaterEnabled: currentConfig.memospot.updater.enabled,
		migrationsEnabled: currentConfig.memospot.migrations.enabled,
		backupsEnabled: currentConfig.memospot.backups.enabled,
		loggingEnabled: currentConfig.memospot.log.enabled,
		envVarsEnabled: currentConfig.memospot.env.enabled,
		envVars: envFromKV(currentConfig.memospot.env.vars),
	};
});
</script>

<div class="space-y-4">
  <div>
    <h3 class="text-lg font-medium">
      {m.memospotDescription()}
    </h3>
  </div>

  <SettingToggle
    name={m.memospotRemoteServer()}
    desc={m.memospotRemoteServerDescription()}
    bind:state={input.remoteEnabled}
    onclick={() => {
      currentConfig.memospot.remote.enabled = input.remoteEnabled;
      updateSetting();
    }}
  >
    <Setting
      name={m.memospotRemoteServerURL()}
      desc={m.memospotRemoteServerURLDescription()}
    >
      <input
        id="url"
        type="url"
        bind:value={input.remoteURL}
        onfocusout={updateRemoteServerUrl}
        class="p-2 rounded-md border bg-input min-w-max md:w-96"
        disabled={!input.remoteEnabled}
      />
    </Setting>
    <Setting
      name={m.memospotUserAgent()}
      desc={m.memospotUserAgentDescription()}
    >
      <input
        id="userAgent"
        type="text"
        bind:value={input.remoteUserAgent}
        onfocusout={() => {
          currentConfig.memospot.remote.user_agent = input.remoteUserAgent;
          updateSetting();
        }}
        onkeypress={(e) =>
          e.key === "Enter" &&
          updateSetting(() => {
            currentConfig.memospot.remote.user_agent = input.remoteUserAgent;
          })}
        class="p-2 rounded-md border bg-input min-w-max md:w-96"
        disabled={!input.remoteEnabled}
      />
    </Setting>
  </SettingToggle>

  <Setting name={m.memospotUpdater()} desc={m.memospotUpdaterDescription()}>
    <Switch
      bind:checked={input.updaterEnabled}
      onclick={() => {
        currentConfig.memospot.updater.enabled = input.updaterEnabled;
        updateSetting();
      }}
    />
  </Setting>

  <Setting
    name={m.memospotMigrations()}
    desc={m.memospotMigrationsDescription()}
  >
    <Switch
      bind:checked={input.migrationsEnabled}
      onclick={() => {
        currentConfig.memospot.migrations.enabled = input.migrationsEnabled;
        updateSetting();
      }}
    />
  </Setting>

  <Setting name={m.memospotBackups()} desc={m.memospotBackupsDescription()}>
    <Switch
      bind:checked={input.backupsEnabled}
      onclick={() => {
        currentConfig.memospot.backups.enabled = input.backupsEnabled;
        updateSetting();
      }}
    />
  </Setting>

  <Setting name={m.memospotLogging()} desc={m.memospotLoggingDescription()}>
    <Switch
      bind:checked={input.loggingEnabled}
      onclick={() => {
        currentConfig.memospot.log.enabled = input.loggingEnabled;
        updateSetting();
      }}
    />
  </Setting>

  <SettingToggle
    name={m.memospotEnvironmentVariables()}
    desc={m.memospotEnvironmentVariablesDescription()}
    bind:state={input.envVarsEnabled}
    onclick={() => {
      currentConfig.memospot.env.enabled = input.envVarsEnabled;
      updateSetting();
    }}
  >
    <textarea
      id="env"
      rows="5"
      class="p-2 rounded-md border bg-background min-w-max w-full leading-tight"
      bind:value={input.envVars}
      onfocusout={updateEnvVars}
      onkeypress={(e) => e.key === "Enter" && updateEnvVars(e)}
      disabled={!input.envVarsEnabled}
    >
    </textarea>
  </SettingToggle>
</div>
