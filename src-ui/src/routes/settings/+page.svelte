<script lang="ts">
import Fuse from "fuse.js";
import { onMount, tick } from "svelte";
import EyeOpen from "svelte-radix/EyeOpen.svelte";
import Gear from "svelte-radix/Gear.svelte";
import Pencil2 from "svelte-radix/Pencil2.svelte";
import { Toaster } from "$lib/components/ui/sonner";
import { m } from "$lib/i18n";
import { collectSettingsEntries, type SettingSearchEntry } from "$lib/settingsSearch";
import type { SectionActions } from "$lib/settingsUi";
import Memos from "./Memos.svelte";
import Memospot from "./Memospot.svelte";
import type { Section } from "./Navbar.svelte";
import Navbar from "./Navbar.svelte";
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
let searchQuery = $state("");
let contentPane: HTMLElement | undefined = $state(undefined);
let sectionActions: Record<string, SectionActions> = $state({});
let sectionSearchEntries: Record<string, SettingSearchEntry[]> = $state({});
let highlightedResultIndex = $state(0);
let isSearchDropdownOpen = $state(false);
let highlightedElement: HTMLElement | null = null;
let highlightTimer: ReturnType<typeof setTimeout> | undefined;
let hasPreloadedSearchEntries = $state(false);
let searchInputElement: HTMLInputElement | undefined = $state(undefined);

const PREFERRED_FOCUS_SELECTORS = [
    "input[type='text']:not([disabled]):not([readonly])",
    "input[type='search']:not([disabled]):not([readonly])",
    "input[type='url']:not([disabled]):not([readonly])",
    "input[type='email']:not([disabled]):not([readonly])",
    "input[type='tel']:not([disabled]):not([readonly])",
    "input[type='password']:not([disabled]):not([readonly])",
    "input:not([type]):not([disabled]):not([readonly])",
    "textarea:not([disabled]):not([readonly])",
    "select:not([disabled])",
    "[contenteditable='true']",
    "button:not([disabled])",
    "input:not([type='hidden']):not([disabled])",
    "[tabindex]:not([tabindex='-1'])"
];

function getPreferredFocusTarget(container: HTMLElement): HTMLElement | null {
    for (const selector of PREFERRED_FOCUS_SELECTORS) {
        const candidate = container.querySelector<HTMLElement>(selector);
        if (candidate) return candidate;
    }
    return null;
}

function findSettingRowById(settingId: string): HTMLElement | null {
    if (!contentPane) return null;
    const rows = contentPane.querySelectorAll<HTMLElement>("[data-setting-row='true']");
    for (const row of rows) {
        if ((row.dataset.settingId ?? "") === settingId) {
            return row;
        }
    }
    return null;
}

const allSearchEntries = $derived(Object.values(sectionSearchEntries).flat());

const fuzzyResults = $derived.by(() => {
    const query = searchQuery.trim();
    if (query.length === 0 || allSearchEntries.length === 0) return [];

    const fuse = new Fuse(allSearchEntries, {
        threshold: 0.35,
        ignoreLocation: true,
        keys: [
            { name: "label", weight: 0.75 },
            { name: "keywords", weight: 0.25 }
        ]
    });

    return fuse
        .search(query, { limit: 25 })
        .map((result: { item: SettingSearchEntry }) => result.item);
});

const groupedFuzzyResults = $derived.by(() => {
    const grouped = new Map<string, { sectionLabel: string; items: SettingSearchEntry[] }>();

    for (const result of fuzzyResults) {
        const group = grouped.get(result.sectionId);
        if (!group) {
            grouped.set(result.sectionId, {
                sectionLabel: result.sectionLabel,
                items: [result]
            });
            continue;
        }
        if (group.items.length < 5) {
            group.items.push(result);
        }
    }

    return Array.from(grouped.entries()).map(([sectionId, group]) => ({
        sectionId,
        sectionLabel: group.sectionLabel,
        items: group.items
    }));
});

const flatFuzzyResults = $derived(groupedFuzzyResults.flatMap((group) => group.items));

const showNoResults = $derived(
    searchQuery.trim().length > 0 && groupedFuzzyResults.length === 0
);

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

async function handleSearchSubmit(event: SubmitEvent) {
    event.preventDefault();
    if (flatFuzzyResults.length > 0) {
        await navigateToSearchResult(
            flatFuzzyResults[highlightedResultIndex] ?? flatFuzzyResults[0]
        );
    }
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

async function navigateToSearchResult(entry: SettingSearchEntry) {
    await updateSectionWithOptions(entry.sectionId, { scrollTop: false, animate: false });
    await tick();

    let target = findSettingRowById(entry.id);
    if (!target) {
        for (let attempt = 0; attempt < 6 && !target; attempt += 1) {
            await new Promise((resolve) => setTimeout(resolve, 50));
            await tick();
            target = findSettingRowById(entry.id);
        }
    }
    if (!target) return;

    if (contentPane) {
        const paneRect = contentPane.getBoundingClientRect();
        const targetRect = target.getBoundingClientRect();
        const offsetWithinPane = targetRect.top - paneRect.top;
        const centeredTop =
            contentPane.scrollTop +
            offsetWithinPane -
            (contentPane.clientHeight / 2 - targetRect.height / 2);

        contentPane.scrollTo({
            top: Math.max(0, centeredTop),
            behavior: reduceAnimation ? "auto" : "smooth"
        });
    } else {
        target.scrollIntoView({
            block: "center",
            behavior: reduceAnimation ? "auto" : "smooth"
        });
    }

    const focusTarget = getPreferredFocusTarget(target);
    if (focusTarget) {
        focusTarget.focus({ preventScroll: true });
        if (focusTarget instanceof HTMLInputElement) {
            const isTextualInput = ["text", "search", "url", "tel", "password"].includes(
                focusTarget.type
            );
            if (isTextualInput) {
                focusTarget.select();
            }
        }
    }

    if (highlightedElement) {
        highlightedElement.classList.remove("settings-search-highlight");
    }
    if (highlightTimer) {
        clearTimeout(highlightTimer);
    }

    highlightedElement = target;
    highlightedElement.classList.add("settings-search-highlight");
    highlightTimer = setTimeout(() => {
        highlightedElement?.classList.remove("settings-search-highlight");
        highlightedElement = null;
    }, 1400);

    isSearchDropdownOpen = false;
}

function handleSearchKeydown(event: KeyboardEvent) {
    if (!isSearchDropdownOpen && event.key !== "Escape") {
        isSearchDropdownOpen = searchQuery.trim().length > 0;
    }

    if (event.key === "Escape") {
        isSearchDropdownOpen = false;
        return;
    }

    if (flatFuzzyResults.length === 0) {
        return;
    }

    if (event.key === "ArrowDown") {
        event.preventDefault();
        highlightedResultIndex = (highlightedResultIndex + 1) % flatFuzzyResults.length;
    }

    if (event.key === "ArrowUp") {
        event.preventDefault();
        highlightedResultIndex =
            (highlightedResultIndex - 1 + flatFuzzyResults.length) % flatFuzzyResults.length;
    }
}

$effect(() => {
    searchQuery;
    highlightedResultIndex = 0;
});

$effect(() => {
    if (!contentPane) return;
    collectSearchEntriesForSection(activeSection);
});

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

function focusSearchInput() {
    searchInputElement?.focus();
    searchInputElement?.select();
    isSearchDropdownOpen = searchQuery.trim().length > 0;
}

onMount(() => {
    const onGlobalKeydown = (event: KeyboardEvent) => {
        const key = event.key.toLowerCase();
        const isCtrlOrMeta = event.ctrlKey || event.metaKey;
        const isSearchShortcut = key === "f3" || (isCtrlOrMeta && (key === "f" || key === "k"));

        if (!isSearchShortcut) {
            return;
        }

        event.preventDefault();
        focusSearchInput();
    };

    window.addEventListener("keydown", onGlobalKeydown);
    return () => {
        window.removeEventListener("keydown", onGlobalKeydown);
    };
});

$effect(() => {
    if (!contentPane || hasPreloadedSearchEntries) return;
    hasPreloadedSearchEntries = true;
    void preloadSearchEntriesInBackground();
});
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
        <form
          class="relative mx-auto flex w-full max-w-xl items-center gap-2"
          onsubmit={handleSearchSubmit}
          onfocusout={(event) => {
            const nextTarget = event.relatedTarget as Node | null;
            if (!nextTarget || !event.currentTarget.contains(nextTarget)) {
              isSearchDropdownOpen = false;
            }
          }}
        >
          <div class="relative w-full">
            <input
              type="search"
              bind:this={searchInputElement}
              bind:value={searchQuery}
              placeholder={m.settingsSearchPlaceholder()}
              class="w-full rounded-md border bg-background px-3 py-2 pr-14 text-sm placeholder:text-center md:min-w-72"
              aria-label={m.settingsSearchPlaceholder()}
              onfocus={() => {
                isSearchDropdownOpen = searchQuery.trim().length > 0;
              }}
              onkeydown={handleSearchKeydown}
            />
            {#if searchQuery.trim().length > 0}
              <button
                type="button"
                class="absolute right-2 top-1/2 -translate-y-1/2 rounded-md border px-2 py-1 text-xs hover:bg-secondary"
                onclick={() => {
                  searchQuery = "";
                  isSearchDropdownOpen = false;
                }}
              >
                {m.settingsSearchClear()}
              </button>
            {/if}
          </div>

          {#if isSearchDropdownOpen && searchQuery.trim().length > 0}
            <div class="absolute left-0 top-full z-30 mt-2 max-h-[60vh] w-full overflow-y-auto rounded-lg border bg-popover p-2 shadow-lg md:min-w-md">
              {#if groupedFuzzyResults.length > 0}
                {#each groupedFuzzyResults as group (group.sectionId)}
                  <div class="mb-2 border-t border-border/60 pt-1 first:border-t-0 first:pt-0 last:mb-0">
                    <p class="mb-1 mt-1 px-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground first:mt-0">
                      {group.sectionLabel}
                    </p>
                    <div class="space-y-1">
                      {#each group.items as result (result.id)}
                        <button
                          type="button"
                          class={{
                            "block w-full rounded-md px-2 py-1.5 text-left text-sm":true,
                             "bg-accent text-accent-foreground": flatFuzzyResults[highlightedResultIndex]?.id === result.id,
                             "hover:bg-secondary/70": !flatFuzzyResults[highlightedResultIndex]?.id === result.id
                          }}
                          onmousedown={async (event) => {
                            event.preventDefault();
                            await navigateToSearchResult(result);
                          }}
                        >
                          <div class="font-medium">{result.label}</div>
                          {#if result.keywords.length > 0}
                            <div class="text-xs text-muted-foreground">
                              {result.keywords.slice(0, 2).join(" · ")}
                            </div>
                          {/if}
                        </button>
                      {/each}
                    </div>
                  </div>
                {/each}
              {:else if showNoResults}
                <p class="px-2 py-2 text-sm text-muted-foreground">
                  {m.settingsSearchNoResults()}
                </p>
              {/if}
            </div>
          {/if}
        </form>

        <div class="flex items-center gap-2 justify-self-end">
          <button
            type="button"
            class="border border-box inline-flex h-9 cursor-pointer items-center justify-center rounded-md bg-secondary px-3 py-1.5 text-sm hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-40"
            onclick={async () => await activeSectionActions.loadDefaults?.()}
          >
            {m.settingsLoadDefaults()}
          </button>
          <button
            type="button"
            class="border border-box inline-flex h-9 cursor-pointer items-center justify-center rounded-md bg-secondary px-3 py-1.5 text-sm hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-40"
            onclick={async () => await activeSectionActions.reloadCurrent?.()}
          >
            {m.settingsReloadCurrent()}
          </button>
          <button
            type="button"
            class={{
              "border-box inline-flex h-9 cursor-pointer items-center justify-center rounded-md px-3 py-1.5 text-sm  hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50" : true,
              "border-box inline-flex h-9 cursor-pointer items-center justify-center rounded-md bg-primary px-3 py-1.5 text-sm text-zinc-50 hover:opacity-80 disabled:cursor-not-allowed disabled:opacity-50":!activeSectionActions.hasPendingChanges,
             "border-amber-600/80 bg-amber-200/70 text-amber-900 dark:border-amber-500/70 dark:bg-amber-500/15 dark:text-amber-300": activeSectionActions.hasPendingChanges
            }}
            disabled={!activeSectionActions.hasPendingChanges}
            onclick={async () => await activeSectionActions.save?.()}
          >
            {m.settingsSave()}
          </button>
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
        bind:this={contentPane}
        class="min-h-0 overflow-y-auto rounded-xl border bg-card p-3"
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
      success: "bg-primary",
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
