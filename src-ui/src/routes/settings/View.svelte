<script lang="ts">
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue
} from "$lib/components/ui/select";
import { Setting } from "$lib/components/ui/setting/index";
import { Switch } from "$lib/components/ui/switch/index";
import { debouncePromise } from "$lib/debounce";
import { detectLanguage, m } from "$lib/i18n";
import { availableLanguageTags, setLanguageTag } from "$lib/paraglide/runtime.js";
import { patchConfig } from "$lib/settings";
import { getAppConfig, getDefaultAppConfig, setAppLanguage } from "$lib/tauri";
import type { Config } from "$lib/types/gen/Config";
import type { Selected } from "bits-ui";
import * as jsonpatch from "fast-json-patch";
import { resetMode as resetTheme, setMode as setTheme } from "mode-watcher";
import { onMount } from "svelte";
import LightningBolt from "svelte-radix/LightningBolt.svelte";
import Moon from "svelte-radix/Moon.svelte";
import Sun from "svelte-radix/Sun.svelte";

type Language = (typeof availableLanguageTags)[number];
type Theme = "system" | "light" | "dark";

let initialConfig = $state({}) as Config;
let currentConfig = $state({}) as Config;
let input = $state({
    resizable: false,
    maximized: false,
    fullscreen: false,
    centered: false,
    language: "system" as Language,
    theme: "system" as Theme
});

const themeNames = {
    dark: m.settingsViewDark(),
    light: m.settingsViewLight(),
    system: m.settingsViewSystem()
} as const;

let selectedTheme = $derived({
    label: themeNames[input.theme as Theme],
    value: input.theme
});

let languagesDisplayNames: Record<string, string> = {};
for (const tag of availableLanguageTags.toSorted()) {
    const displayName = new Intl.DisplayNames([tag], { type: "language" }).of(tag);
    if (!displayName) {
        const error = new Error(`Language tag "${tag}" is not recognized by the browser.`);
        console.error(error);
        if (import.meta.env.DEV) {
            alert(error.message);
        }
        continue;
    }
    languagesDisplayNames[tag] = displayName.slice(0, 1).toUpperCase() + displayName.slice(1);
}

let selectedLanguageTag = $derived({
    label:
        input.language === ("system" as Language)
            ? m.settingsViewSystem()
            : languagesDisplayNames[input.language as Language],
    value: input.language
});

onMount(async () => {
    const initialJSON = await getAppConfig();
    initialConfig = JSON.parse(initialJSON);
    currentConfig = jsonpatch.deepClone(initialConfig);
    await setPageToInitialConfig();
});

async function setPageToInitialConfig() {
    input = {
        resizable: (initialConfig.memospot.window.resizable as boolean) || false,
        maximized: (initialConfig.memospot.window.maximized as boolean) || false,
        fullscreen: (initialConfig.memospot.window.fullscreen as boolean) || false,
        centered: (initialConfig.memospot.window.center as boolean) || false,
        language: (initialConfig.memospot.window.language || "system") as Language,
        theme: (initialConfig.memospot.window.theme ||
            localStorage.getItem("mode-watcher-mode") ||
            "system") as Theme
    };

    currentConfig.memospot.window = jsonpatch.deepClone(initialConfig.memospot.window);
}

async function setPageToDefaultConfig() {
    const defaultJSON = JSON.parse(await getDefaultAppConfig()) as Config;
    input = {
        resizable: (defaultJSON.memospot.window.resizable as boolean) || false,
        maximized: (defaultJSON.memospot.window.maximized as boolean) || false,
        fullscreen: (defaultJSON.memospot.window.fullscreen as boolean) || false,
        centered: (defaultJSON.memospot.window.center as boolean) || false,
        language: (defaultJSON.memospot.window.language || "system") as Language,
        theme: (defaultJSON.memospot.window.theme || "system") as Theme
    };

    currentConfig.memospot.window = jsonpatch.deepClone(defaultJSON.memospot.window);
}

async function updateTheme(s: Selected<string> | undefined) {
    input.theme = (s?.value || "system") as Theme;
    currentConfig.memospot.window.theme = input.theme;
}

async function updateLanguage(s: Selected<string> | undefined) {
    input.language = (s?.value || "system") as Language;
    currentConfig.memospot.window.language = input.language;

    if (input.language === ("system" as Language)) {
        detectLanguage(input.language);
    } else {
        setLanguageTag(input.language);
    }

    await setAppLanguage(input.language);
}

async function updateSetting(updateFn?: () => void): Promise<void> {
    const languageChanged =
        initialConfig.memospot.window.language !== currentConfig.memospot.window.language;

    await debouncePromise(async () => {
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

    if (input.theme === "system") {
        resetTheme();
    } else {
        setTheme(input.theme);
    }

    if (languageChanged) {
        window.location.reload();
    }
}
</script>

<div class="space-y-4 w-full">
  <div>
    <h3 class="text-lg font-medium">
      {m.settingsViewDescription()}
    </h3>

    <p class="text-sm text-muted-foreground">{m.settingsOverview()}</p>
  </div>

  <Setting name={m.settingsViewTheme()} desc={m.settingsViewThemeDescription()}>
    <Select selected={selectedTheme} onSelectedChange={updateTheme}>
      <SelectTrigger class="ml-1 w-52">
        <SelectValue placeholder={m.settingsViewTheme()} />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="system">
          {themeNames.system}<LightningBolt class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </SelectItem>
        <SelectItem value="dark">
          {themeNames.dark}<Moon class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </SelectItem>
        <SelectItem value="light">
          {themeNames.light}<Sun class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </SelectItem>
      </SelectContent>
    </Select>
  </Setting>

  <Setting
    name={m.settingsViewLanguage()}
    desc={m.settingsViewLanguageDescription()}
  >
    <Select selected={selectedLanguageTag} onSelectedChange={updateLanguage}>
      <SelectTrigger class="ml-2 w-64">
        <SelectValue placeholder={m.settingsViewLanguage()} />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="system">
          {m.settingsViewSystem()} <LightningBolt class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </SelectItem>
        {#each Object.entries(languagesDisplayNames) as [code, displayName]}
          <SelectItem value={code}>
            {displayName}
          </SelectItem>
        {/each}
      </SelectContent>
    </Select>
  </Setting>

  <Setting
    name={m.settingsViewResizable()}
    desc={m.settingsViewResizableDescription()}
  >
    <Switch
      bind:checked={input.resizable}
      onclick={() => {
        currentConfig.memospot.window.resizable = input.resizable;
      }}
    />
  </Setting>

  <Setting
    name={m.settingsViewCentered()}
    desc={m.settingsViewCenteredDescription()}
  >
    <Switch
      bind:checked={input.centered}
      onclick={() => {
        currentConfig.memospot.window.center = input.centered;
      }}
    />
  </Setting>

  <Setting
    name={m.settingsViewMaximized()}
    desc={m.settingsViewMaximizedDescription()}
  >
    <Switch
      bind:checked={input.maximized}
      onclick={() => {
        currentConfig.memospot.window.maximized = input.maximized;
      }}
    />
  </Setting>

  <Setting
    name={m.settingsViewFullscreen()}
    desc={m.settingsViewFullscreenDescription()}
  >
    <Switch
      bind:checked={input.fullscreen}
      onclick={() => {
        currentConfig.memospot.window.fullscreen = input.fullscreen;
      }}
    />
  </Setting>

  <div class="flex flex-row space-x-1 justify-end">
    <button
      class="border-box inline-flex items-center justify-center rounded-md bg-secondary text-base px-4 py-2 h-10 cursor-pointer hover:opacity-80 hover:translate-y-[-1px]"
      type="button"
      onclick={async () => await setPageToDefaultConfig()}
    >
      {m.settingsLoadDefaults()}
    </button>
    <button
      class="border-box inline-flex items-center justify-center rounded-md bg-secondary text-base px-4 py-2 h-10 cursor-pointer hover:opacity-80 hover:translate-y-[-1px]"
      type="button"
      onclick={async () => await setPageToInitialConfig()}
    >
      {m.settingsReloadCurrent()}
    </button>
    <button
      class="border-box inline-flex items-center justify-center rounded-md bg-primary text-zinc-50 text-base px-4 py-2 h-10 cursor-pointer hover:opacity-80 hover:translate-y-[-1px] [text-shadow:_1px_1px_0_rgb(0_0_0_/_90%)]"
      type="button"
      onclick={async () => await updateSetting()}
    >
      {m.settingsSave()}
    </button>
  </div>
</div>
