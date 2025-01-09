<script lang="ts">
import * as Select from "$lib/components/ui/select";
import { Setting } from "$lib/components/ui/setting/index";
import { Switch } from "$lib/components/ui/switch/index";
import { debouncePromise } from "$lib/debounce";
import { m } from "$lib/i18n";
import { availableLanguageTags, setLanguageTag } from "$lib/paraglide/runtime.js";
import { patchConfig } from "$lib/settings";
import { getConfig } from "$lib/tauri";
import type { Config } from "$lib/types/gen/Config";
import * as jsonpatch from "fast-json-patch";
import { resetMode, setMode } from "mode-watcher";
import { onMount } from "svelte";
import LightningBolt from "svelte-radix/LightningBolt.svelte";
import Moon from "svelte-radix/Moon.svelte";
import Sun from "svelte-radix/Sun.svelte";

// Theme settings
let theme: string = $state(localStorage.getItem("mode-watcher-mode") || "system");
$effect(() => {
	if (theme === "system") {
		resetMode();
		return;
	}
	if (["light", "dark"].includes(theme)) {
		setMode(theme as "light" | "dark");
	}
});

const themes = {
	dark: m.viewDark(),
	light: m.viewLight(),
	system: m.viewSystem(),
} as const;

let selectedTheme = $derived({
	label: themes[theme as keyof typeof themes],
	value: theme,
});

let language: string = $state(localStorage.getItem("i18n-user-preference") || "system");

let languages: Record<string, string> = {};
for (const lang of availableLanguageTags.toSorted()) {
	const displayName = new Intl.DisplayNames([lang], { type: "language" }).of(lang);
	if (!displayName) {
		console.error(`Language "${lang}" is not recognized by the browser.`);
		continue;
	}
	languages[lang] = displayName.slice(0, 1).toUpperCase() + displayName.slice(1);
}

$effect(() => {
	localStorage.setItem("i18n-user-preference", language);
	if (language === "system") {
		return;
	}
	setLanguageTag(language as (typeof availableLanguageTags)[number]);
});

let selectedLanguageTag = $derived({
	label:
		language === "system"
			? m.viewSystem()
			: languages[language as (typeof availableLanguageTags)[number]],
	value: language,
});

let initialConfig = $state({}) as Config;
let currentConfig = $state({}) as Config;
let input = $state({
	resizable: false,
	maximized: false,
	fullscreen: false,
	centered: false,
});

onMount(async () => {
	const initialJSON = await getConfig();
	initialConfig = JSON.parse(initialJSON);
	currentConfig = jsonpatch.deepClone(initialConfig);
	input = {
		resizable: currentConfig.memospot.window.resizable as boolean,
		maximized: currentConfig.memospot.window.maximized as boolean,
		fullscreen: currentConfig.memospot.window.fullscreen as boolean,
		centered: currentConfig.memospot.window.center as boolean,
	};
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
</script>

<div class="space-y-4 w-full">
  <div>
    <h3 class="text-lg font-medium">
      {m.viewDescription()}
    </h3>
  </div>

  <Setting name={m.viewTheme()} desc={m.viewThemeDescription()}>
    <Select.Root
      selected={selectedTheme}
      onSelectedChange={(s) => s && (theme = s.value ?? "system")}
    >
      <Select.Trigger class="ml-1 w-52">
        <Select.Value placeholder={m.viewTheme()} />
      </Select.Trigger>
      <Select.Content>
        <Select.Item value="system" class="text-primary">
          {themes.system}<LightningBolt class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </Select.Item>
        <Select.Item value="dark">
          {themes.dark}<Moon class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </Select.Item>
        <Select.Item value="light">
          {themes.light}<Sun class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </Select.Item>
      </Select.Content>
    </Select.Root>
  </Setting>

  <Setting name={m.viewLanguage()} desc={m.viewLanguageDescription()}>
    <Select.Root
      selected={selectedLanguageTag}
      onSelectedChange={(s) =>
        s && (language = s.value) && window.location.reload()}
    >
      <Select.Trigger class="ml-2 w-64">
        <Select.Value placeholder="system" />
      </Select.Trigger>
      <Select.Content>
        <Select.Item value="system" class="text-primary">
          {m.viewSystem()}
          <LightningBolt class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </Select.Item>
        {#each Object.entries(languages) as [code, displayName]}
          <Select.Item value={code}>
            {displayName}
          </Select.Item>
        {/each}
      </Select.Content>
    </Select.Root>
  </Setting>

  <Setting name={m.viewResizable()} desc={m.viewResizableDescription()}>
    <Switch
      bind:checked={input.resizable}
      onclick={() => {
        currentConfig.memospot.window.resizable = input.resizable;
        updateSetting();
      }}
    />
  </Setting>

  <Setting name={m.viewCentered()} desc={m.viewCenteredDescription()}>
    <Switch
      bind:checked={input.centered}
      onclick={() => {
        currentConfig.memospot.window.center = input.centered;
        updateSetting();
      }}
    />
  </Setting>

  <Setting name={m.viewMaximized()} desc={m.viewMaximizedDescription()}>
    <Switch
      bind:checked={input.maximized}
      onclick={() => {
        currentConfig.memospot.window.maximized = input.maximized;
        updateSetting();
      }}
    />
  </Setting>

  <Setting name={m.viewFullscreen()} desc={m.viewFullscreenDescription()}>
    <Switch
      bind:checked={input.fullscreen}
      onclick={() => {
        currentConfig.memospot.window.fullscreen = input.fullscreen;
        updateSetting();
      }}
    />
  </Setting>
</div>
