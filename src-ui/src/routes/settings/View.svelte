<script lang="ts">
import * as Select from "$lib/components/ui/select";
import { Setting } from "$lib/components/ui/setting/index";
import { Switch } from "$lib/components/ui/switch/index";
import { m } from "$lib/i18n";
import { availableLanguageTags, setLanguageTag } from "$lib/paraglide/runtime.js";
import { resetMode, setMode } from "mode-watcher";
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

let stateResizable = $state(false);
let stateMaximized = $state(false);
let stateFullscreen = $state(false);
let stateCentered = $state(false);
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
    <Switch bind:state={stateResizable} />
  </Setting>

  <Setting name={m.viewCentered()} desc={m.viewCenteredDescription()}>
    <Switch bind:state={stateCentered} />
  </Setting>

  <Setting name={m.viewMaximized()} desc={m.viewMaximizedDescription()}>
    <Switch bind:state={stateMaximized} />
  </Setting>

  <Setting name={m.viewFullscreen()} desc={m.viewFullscreenDescription()}>
    <Switch bind:state={stateFullscreen} />
  </Setting>
</div>
