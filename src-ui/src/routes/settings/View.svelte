<script lang="ts">
import type { Selected } from "bits-ui";
import { modeStorageKey, resetMode, setMode } from "mode-watcher";
import { onMount } from "svelte";
import LightningBolt from "svelte-radix/LightningBolt.svelte";
import Moon from "svelte-radix/Moon.svelte";
import Sun from "svelte-radix/Sun.svelte";
import { toast } from "svelte-sonner";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue
} from "$lib/components/ui/select";
import { Setting } from "$lib/components/ui/setting/index";
import { Switch } from "$lib/components/ui/switch/index";
import { applyLocalePreference, type Locale, locales, m } from "$lib/i18n";
import { createSettingsSection } from "$lib/settingsSection";
import {
    buildSectionActions,
    keywordsFromLocale,
    type SectionActionsProps
} from "$lib/settingsUi";
import { getLocalePreference, setAppLocale } from "$lib/tauri";
import type { Config } from "$lib/types/gen/Config";

type Theme = "system" | "light" | "dark";

let { onActionsChange }: SectionActionsProps = $props();

type ViewInput = {
    resizable: boolean;
    maximized: boolean;
    fullscreen: boolean;
    centered: boolean;
    locale: Locale;
    reduce_animation: boolean;
    theme: Theme;
};

const section = $state(
    createSettingsSection<ViewInput>({
        mapping: {
            initial: {
                resizable: false,
                maximized: false,
                fullscreen: false,
                centered: false,
                locale: "system" as Locale,
                reduce_animation: false,
                theme: "system" as Theme
            },
            mapToInput: (cfg: Config) => ({
                resizable: cfg.memospot.window.resizable ?? false,
                maximized: cfg.memospot.window.maximized ?? false,
                fullscreen: cfg.memospot.window.fullscreen ?? false,
                centered: cfg.memospot.window.center ?? false,
                locale: (cfg.memospot.window.locale ?? "system") as Locale,
                reduce_animation: cfg.memospot.window.reduce_animation ?? false,
                theme: (cfg.memospot.window.theme ??
                    localStorage.getItem(modeStorageKey.current) ??
                    "system") as Theme
            }),
            mapFromInput: (input: ViewInput, cfg: Config) => {
                cfg.memospot.window.resizable = input.resizable;
                cfg.memospot.window.maximized = input.maximized;
                cfg.memospot.window.fullscreen = input.fullscreen;
                cfg.memospot.window.center = input.centered;
                cfg.memospot.window.locale = input.locale;
                cfg.memospot.window.reduce_animation = input.reduce_animation;
                cfg.memospot.window.theme = input.theme;
            },
            live: ["locale"]
        }
    })
);

const themeNames = {
    dark: m.settingsViewDark(),
    light: m.settingsViewLight(),
    system: m.settingsViewSystem()
} as const;

let selectedTheme = $derived({
    label: themeNames[section.input.theme as Theme],
    value: section.input.theme
});

let localeDisplayNames: Record<string, string> = {};
for (const locale of locales.toSorted()) {
    const displayName = new Intl.DisplayNames([locale], {
        type: "language"
    }).of(locale);
    if (!displayName) {
        const error = new Error(`Locale "${locale}" is not recognized by the browser.`);
        console.error(error);
        if (import.meta.env.DEV) {
            alert(error.message);
        }
        continue;
    }
    localeDisplayNames[locale] = displayName.slice(0, 1).toUpperCase() + displayName.slice(1);
}

let selectedLocale = $derived({
    label:
        section.input.locale === ("system" as Locale)
            ? m.settingsViewSystem()
            : localeDisplayNames[section.input.locale as Locale],
    value: section.input.locale
});

onMount(async () => {
    await section.init();
});

async function updateTheme(s: Selected<string> | undefined) {
    section.input.theme = (s?.value ?? "system") as Theme;
}

async function updateLocale(s: Selected<string> | undefined) {
    const baselineLocale = section.baselineInput.locale as Locale;
    const newLocale = (s?.value ?? "system") as Locale;
    section.input.locale = newLocale;

    try {
        await setAppLocale(newLocale);
        section.commitLive("locale", newLocale);
        await applyLocalePreference(newLocale);
        if (baselineLocale !== newLocale) {
            window.location.reload();
        }
    } catch (_err) {
        let committedLocale = baselineLocale;
        try {
            committedLocale = (await getLocalePreference()) as Locale;
        } catch {
            // keep existing baseline
        }
        section.commitLive("locale", committedLocale);
        toast.error(m.settingsConfigSaveFail());
    }
}

async function handleSave(): Promise<boolean> {
    const localeBefore = section.baselineInput.locale;
    const saved = await section.save();
    if (saved) {
        localStorage.setItem(
            "reduce-animation",
            JSON.stringify(section.input.reduce_animation)
        );
    } else if (section.input.reduce_animation !== section.baselineInput.reduce_animation) {
        localStorage.setItem(
            "reduce-animation",
            JSON.stringify(section.baselineInput.reduce_animation)
        );
    }
    if (section.input.theme === "system") {
        resetMode();
    } else {
        setMode(section.input.theme);
    }
    if (localeBefore !== section.input.locale && saved) {
        window.location.reload();
    }
    return saved;
}

$effect(() => {
    onActionsChange?.(
        buildSectionActions(
            () => section.loadDefaults(),
            () => section.reset(),
            handleSave,
            section.hasPendingChanges
        )
    );
});
</script>

<div class="space-y-3">
  <div class="mb-4">
    <h3 class="font-semibold uppercase tracking-[0.09rem] text-sm text-foreground mb-1">
      {m.settingsViewDescription()}
    </h3>

    <p class="text-sm text-muted-foreground">{m.settingsOverview()}</p>
  </div>

  <Setting
    name={m.settingsViewTheme()}
    desc={m.settingsViewThemeDescription()}
    searchId="view-theme"
    searchKeywords={keywordsFromLocale(m.settingsViewThemeSearchKeywords)}
  >
    <Select portal={null} selected={selectedTheme} onSelectedChange={updateTheme}>
      <SelectTrigger class="ml-1 w-52">
        <SelectValue placeholder={m.settingsViewTheme()} />
      </SelectTrigger>
      <SelectContent reduceAnimation={section.input.reduce_animation}>
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
    name={m.settingsViewReduceAnimation()}
    desc={m.settingsViewReduceAnimationDescription()}
    searchId="view-reduce-animation"
    searchKeywords={keywordsFromLocale(m.settingsViewReduceAnimationSearchKeywords)}
  >
    <Switch bind:checked={section.input.reduce_animation} />
  </Setting>

  <Setting
    name={m.settingsViewLocale()}
    desc={m.settingsViewLocaleDescription()}
    searchId="view-locale"
    searchKeywords={keywordsFromLocale(m.settingsViewLocaleSearchKeywords)}
  >
    <Select portal={null} selected={selectedLocale} onSelectedChange={updateLocale}>
      <SelectTrigger class="ml-2 w-64">
        <SelectValue placeholder={m.settingsViewLocale()} />
      </SelectTrigger>
      <SelectContent reduceAnimation={section.input.reduce_animation}>
        <SelectItem value="system">
          {m.settingsViewSystem()} <LightningBolt class="h-[1.2rem] w-[1.2rem] ml-auto" />
        </SelectItem>
        {#each Object.entries(localeDisplayNames) as [code, displayName] (code)}
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
    searchId="view-resizable"
    searchKeywords={keywordsFromLocale(m.settingsViewResizableSearchKeywords)}
  >
    <Switch bind:checked={section.input.resizable} />
  </Setting>

  <Setting
    name={m.settingsViewCentered()}
    desc={m.settingsViewCenteredDescription()}
    searchId="view-centered"
    searchKeywords={keywordsFromLocale(m.settingsViewCenteredSearchKeywords)}
  >
    <Switch bind:checked={section.input.centered} />
  </Setting>

  <Setting
    name={m.settingsViewMaximized()}
    desc={m.settingsViewMaximizedDescription()}
    searchId="view-maximized"
    searchKeywords={keywordsFromLocale(m.settingsViewMaximizedSearchKeywords)}
  >
    <Switch bind:checked={section.input.maximized} />
  </Setting>

  <Setting
    name={m.settingsViewFullscreen()}
    desc={m.settingsViewFullscreenDescription()}
    searchId="view-fullscreen"
    searchKeywords={keywordsFromLocale(m.settingsViewFullscreenSearchKeywords)}
  >
    <Switch bind:checked={section.input.fullscreen} />
  </Setting>
</div>
