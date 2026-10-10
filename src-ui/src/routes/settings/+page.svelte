<script lang="ts">
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { tick, untrack } from "svelte";
import EyeOpen from "svelte-radix/EyeOpen.svelte";
import Gear from "svelte-radix/Gear.svelte";
import Pencil2 from "svelte-radix/Pencil2.svelte";
import { Button } from "#lib/components/ui/button";
import { Toaster } from "#lib/components/ui/sonner";
import { m } from "#lib/i18n";
import { readReduceAnimation } from "#lib/reduceAnimation";
import type { SettingSearchEntry } from "#lib/settingsSearch";
import { createSettingsSearchIndex } from "#lib/settingsSearchIndex.svelte";
import { navigateToSearchResult } from "#lib/settingsSearchNavigation";
import type { OnSectionActionsChange, SectionActions } from "#lib/settingsUi.svelte";
import Memos from "./Memos.svelte";
import Memospot from "./Memospot.svelte";
import type { Section } from "./Navbar.svelte";
import Navbar from "./Navbar.svelte";
import SettingsSearch from "./SettingsSearch.svelte";
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
let highlightedElement: HTMLElement | null = null;
let highlightTimer: ReturnType<typeof setTimeout> | undefined;

const searchIndex = createSettingsSearchIndex((id) => sections.find((s) => s.id === id));

const allSearchEntries = $derived(searchIndex.allEntries);

const activeSectionActions = $derived(sectionActions[activeSection] ?? {});

const reduceAnimation = readReduceAnimation();

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
    searchIndex.collectForSection(sectionId, contentPane);
    if ((options.animate ?? true) && !reduceAnimation) await animateSectionTransition();
}

const sectionReporters = new Map<string, OnSectionActionsChange>();

function reporterForSection(sectionId: string): OnSectionActionsChange {
    const existing = sectionReporters.get(sectionId);
    if (existing) return existing;
    const reporter = (actions: SectionActions) => {
        // Child reports arrive on every pending-flag change; skip the write
        // when the flag did not move so the header does not re-render.
        if (sectionActions[sectionId]?.hasPendingChanges === actions.hasPendingChanges) {
            return;
        }
        sectionActions[sectionId] = actions;
    };
    sectionReporters.set(sectionId, reporter);
    return reporter;
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

function setContentPane(node: HTMLElement) {
    // Runs inside the attach effect: keep everything untracked so the
    // attach effect never subscribes to the state it writes. Collecting only
    // writes on real change, so the pipeline settles instead of looping.
    untrack(() => {
        contentPane = node;
        searchIndex.collectForSection(activeSection, node);
    });

    return () => {
        untrack(() => {
            if (contentPane === node) {
                contentPane = undefined;
            }
        });
    };
}
</script>

<div
  class={{
      "container p-4 min-w-screen": true,
      "motion-preset-fade": !reduceAnimation
  }}
>
  <div class="flex h-[calc(100vh-2rem)] max-h-[calc(100vh-2rem)] flex-col">
    <header class="sticky top-0 z-20 rounded-xl border bg-card/95 p-3 backdrop-blur supports-backdrop-filter:bg-card/70">
      <div class="grid grid-cols-1 gap-2 md:grid-cols-[1fr_auto_1fr] md:items-center">
        <div class="flex items-start gap-2 justify-self-start">
          <Button onclick={async () => await getCurrentWebviewWindow().close()}>
            X
          </Button>
        </div>
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
        class="relative isolate min-h-0 overflow-y-auto pl-1 pr-4"
      >
        {#each sections as section (section.id)}
          {#if activeSection === section.id}
            <section.component onActionsChange={reporterForSection(section.id)} />
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
          success: "bg-[var(--glow)] text-zinc-950 border-[var(--glow)]"
      }
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
