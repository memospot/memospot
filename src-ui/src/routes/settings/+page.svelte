<script lang="ts">
import { tick } from "svelte";
import EyeOpen from "svelte-radix/EyeOpen.svelte";
import Gear from "svelte-radix/Gear.svelte";
import Pencil2 from "svelte-radix/Pencil2.svelte";
import { Button } from "$lib/components/ui/button";
import { Toaster } from "$lib/components/ui/sonner";
import { m } from "$lib/i18n";
import { collectSettingsEntries, type SettingSearchEntry } from "$lib/settingsSearch";
import type { SectionActions } from "$lib/settingsUi";
import Memos from "./Memos.svelte";
import Memospot from "./Memospot.svelte";
import type { Section } from "./Navbar.svelte";
import Navbar from "./Navbar.svelte";
import SettingsSearch from "./SettingsSearch.svelte";
import { navigateToSearchResult } from "./searchNavigation";
import View from "./View.svelte";

const sections: Section[] = [
    { id: "view", label: m.settingsView(), icon: EyeOpen, component: View },
    {
        id: "memospot",
        label: m.settingsMemospot(),
        icon: Gear,
        component: Memospot
    },
    { id: "memos", label: m.settingsMemos(), icon: Pencil2, component: Memos }
];

let activeSection: string = $state(
    sections.find((s) => s.id === window.location.hash.slice(1))?.id ?? sections[0].id
);
let contentPane: HTMLElement | undefined = $state(undefined);
let sectionActions: Record<string, SectionActions> = $state({});
let sectionSearchEntries: Record<string, SettingSearchEntry[]> = $state({});
let highlightedElement: HTMLElement | null = null;
let highlightTimer: ReturnType<typeof setTimeout> | undefined;
let hasPreloadedSearchEntries = $state(false);

const allSearchEntries = $derived(Object.values(sectionSearchEntries).flat());

const activeSectionActions = $derived(sectionActions[activeSection] ?? {});

const reduceAnimation = JSON.parse(localStorage.getItem("reduce-animation") ?? "false");

async function animateSectionTransition() {
    const sectionAnimation = "motion-preset-fade";
    const mainSelector = document.querySelector("main");

    mainSelector?.classList.add(sectionAnimation);
    await new Promise<void>((resolve) => {
        setTimeout(() => {
            mainSelector?.classList.remove(sectionAnimation);
            resolve();
        }, 800);
    });
}

async function updateSection(sectionId: string) {
    await updateSectionWithOptions(sectionId, { scrollTop: true });
}

async function updateSectionWithOptions(
    sectionId: string,
    options: { scrollTop: boolean; updateHash?: boolean; animate?: boolean }
) {
    activeSection = sectionId;
    if (options.updateHash ?? true) {
        window.location.hash = `#${sectionId}`;
    }
    if (options.scrollTop) {
        contentPane?.scrollTo({ top: 0, behavior: reduceAnimation ? "auto" : "smooth" });
    }
    await tick();
    collectSearchEntriesForSection(sectionId);
    if ((options.animate ?? true) && !reduceAnimation) await animateSectionTransition();
}

function registerSectionActions(sectionId: string, actions: SectionActions) {
    sectionActions[sectionId] = actions;
}

function collectSearchEntriesForSection(sectionId: string) {
    if (!contentPane) return;
    const section = sections.find((candidate) => candidate.id === sectionId);
    if (!section) return;

    sectionSearchEntries[sectionId] = collectSettingsEntries(
        sectionId,
        section.label,
        contentPane
    );
}

async function handleSearchResultSelect(entry: SettingSearchEntry) {
    const nextHighlightState = await navigateToSearchResult({
        entry,
        contentPane,
        reduceAnimation,
        tick,
        updateSection: async (sectionId) =>
            await updateSectionWithOptions(sectionId, { scrollTop: false, animate: false }),
        highlightState: { highlightedElement, highlightTimer },
        onHighlightCleared: () => {
            highlightedElement = null;
            highlightTimer = undefined;
        }
    });

    highlightedElement = nextHighlightState.highlightedElement;
    highlightTimer = nextHighlightState.highlightTimer;
}

async function preloadSearchEntriesInBackground() {
    const originalSection = activeSection;
    const sectionsToPreload = sections.filter((section) => section.id !== originalSection);

    for (const section of sectionsToPreload) {
        if (sectionSearchEntries[section.id]?.length) continue;
        await updateSectionWithOptions(section.id, {
            scrollTop: false,
            updateHash: false,
            animate: false
        });
    }

    await updateSectionWithOptions(originalSection, {
        scrollTop: false,
        updateHash: false,
        animate: false
    });
}

function setContentPane(node: HTMLElement) {
    contentPane = node;
    collectSearchEntriesForSection(activeSection);

    if (!hasPreloadedSearchEntries) {
        hasPreloadedSearchEntries = true;
        void preloadSearchEntriesInBackground();
    }

    return () => {
        if (contentPane === node) {
            contentPane = undefined;
        }
    };
}
</script>

<div
  class={{
    "container p-4 min-w-screen":true,
    "motion-preset-fade": !reduceAnimation,
  }}
>
  <div class="flex h-[calc(100vh-2rem)] max-h-[calc(100vh-2rem)] flex-col">
    <header class="sticky top-0 z-20 rounded-xl border bg-card/95 p-3 backdrop-blur supports-backdrop-filter:bg-card/70">
      <div class="grid grid-cols-1 gap-2 md:grid-cols-[1fr_auto_1fr] md:items-center">
        <div class="hidden md:block" aria-hidden="true"></div>
        <SettingsSearch
          entries={allSearchEntries}
          placeholder={m.settingsSearchPlaceholder()}
          clearLabel={m.settingsSearchClear()}
          noResultsLabel={m.settingsSearchNoResults()}
          onSelect={async (entry: SettingSearchEntry) => await handleSearchResultSelect(entry)}
        />

        <div class="flex items-center gap-2 justify-self-end">
          <Button onclick={async () => await activeSectionActions.loadDefaults?.()}>
            {m.settingsLoadDefaults()}
          </Button>
          <Button onclick={async () => await activeSectionActions.reloadCurrent?.()}>
            {m.settingsReloadCurrent()}
          </Button>
          <Button
            variant={activeSectionActions.hasPendingChanges ? "warning" : "primary"}
            disabled={!activeSectionActions.hasPendingChanges}
            onclick={async () => await activeSectionActions.save?.()}
          >
            {m.settingsSave()}
          </Button>
        </div>
      </div>
    </header>

    <div class="mt-3 grid min-h-0 flex-1 gap-3 md:grid-cols-[fit-content(22rem)_minmax(0,1fr)]">
      <aside class="w-fit min-w-48 max-w-88 rounded-xl border bg-card p-2 md:sticky md:top-0 md:h-full">
        <div class="max-h-52 overflow-auto md:max-h-none md:h-full">
          <Navbar
            {sections}
            {activeSection}
            onSectionChange={async (sectionId) => await updateSection(sectionId)}
          />
        </div>
      </aside>

      <main
        {@attach setContentPane}
        class="min-h-0 overflow-y-auto pl-1 pr-4"
      >
        {#each sections as section (section.id)}
          {#if activeSection === section.id}
            <section.component
              onActionsChange={(actions: SectionActions) => registerSectionActions(section.id, actions)}
            />
          {/if}
        {/each}
      </main>
    </div>
  </div>
</div>
<Toaster
  duration={1500}
  visibleToasts={1}
  position="bottom-left"
  toastOptions={{
    class: "[text-shadow:_1px_1px_1px_rgb(0_0_0_/_60%)] text-zinc-50",
    classes: {
      error: "bg-destructive",
      success: "bg-[var(--glow)] text-zinc-950 border-[var(--glow)]",
    },
  }}
/>

<style>
:global(.settings-search-highlight) {
    border-color: hsl(var(--primary)) !important;
    animation: settings-search-highlight 1s ease-out;
}

@keyframes settings-search-highlight {
    0% {
        box-shadow:
            0 0 0 1px hsl(var(--primary) / 0.85),
            0 0 0 4px hsl(var(--primary) / 0.35),
            0 0 18px hsl(var(--primary) / 0.5);
    }

    100% {
        box-shadow: 0 0 0 0 hsl(var(--primary) / 0);
    }
}
</style>
