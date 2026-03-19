<script lang="ts">
import Fuse from "fuse.js";
import { tick } from "svelte";
import type { SettingSearchEntry } from "$lib/settingsSearch";

type Props = {
    entries: SettingSearchEntry[];
    placeholder: string;
    clearLabel: string;
    noResultsLabel: string;
    onSelect: (entry: SettingSearchEntry) => void | Promise<void>;
};

let { entries, placeholder, clearLabel, noResultsLabel, onSelect }: Props = $props();

let searchQuery = $state("");
let highlightedResultIndex = $state(0);
let isSearchDropdownOpen = $state(false);
let searchFormElement: HTMLFormElement | undefined = $state(undefined);
let searchInputElement: HTMLInputElement | undefined = $state(undefined);
let searchDropdownElement: HTMLDivElement | undefined = $state(undefined);

const fuzzyResults = $derived.by(() => {
    const query = searchQuery.trim();
    if (query.length === 0 || entries.length === 0) return [];

    const fuse = new Fuse(entries, {
        threshold: 0.35,
        ignoreLocation: true,
        keys: [
            { name: "label", weight: 0.75 },
            { name: "keywords", weight: 0.25 }
        ]
    });

    return fuse.search(query, { limit: 25 }).map((result) => result.item);
});

const groupedFuzzyResults = $derived.by(() => {
    const grouped: Array<{
        sectionId: string;
        sectionLabel: string;
        items: SettingSearchEntry[];
    }> = [];

    for (const result of fuzzyResults) {
        const group = grouped.find((candidate) => candidate.sectionId === result.sectionId);
        if (!group) {
            grouped.push({
                sectionId: result.sectionId,
                sectionLabel: result.sectionLabel,
                items: [result]
            });
            continue;
        }

        if (group.items.length < 5) {
            group.items.push(result);
        }
    }

    return grouped;
});

const flatFuzzyResults = $derived(groupedFuzzyResults.flatMap((group) => group.items));

const showNoResults = $derived(
    searchQuery.trim().length > 0 && groupedFuzzyResults.length === 0
);

function focusSearchInput() {
    searchInputElement?.focus();
    searchInputElement?.select();
    isSearchDropdownOpen = searchQuery.trim().length > 0;
}

function setSearchInput(node: HTMLInputElement) {
    searchInputElement = node;
    return () => {
        if (searchInputElement === node) {
            searchInputElement = undefined;
        }
    };
}

function setSearchForm(node: HTMLFormElement) {
    searchFormElement = node;
    return () => {
        if (searchFormElement === node) {
            searchFormElement = undefined;
        }
    };
}

function blurFocusedSearchElement() {
    const activeElement = document.activeElement;
    if (
        activeElement instanceof HTMLElement &&
        searchFormElement?.contains(activeElement)
    ) {
        activeElement.blur();
    }
}

function handleGlobalKeydown(event: KeyboardEvent) {
    const key = event.key.toLowerCase();
    const isCtrlOrMeta = event.ctrlKey || event.metaKey;
    const isSearchShortcut = key === "f3" || (isCtrlOrMeta && (key === "f" || key === "k"));

    if (!isSearchShortcut) {
        return;
    }

    event.preventDefault();
    focusSearchInput();
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
        void scrollHighlightedResultIntoView();
    }

    if (event.key === "ArrowUp") {
        event.preventDefault();
        highlightedResultIndex =
            (highlightedResultIndex - 1 + flatFuzzyResults.length) % flatFuzzyResults.length;
        void scrollHighlightedResultIntoView();
    }
}

function getResultKey(result: Pick<SettingSearchEntry, "sectionId" | "id">): string {
    return `${result.sectionId}::${result.id}`;
}

function isHighlightedResult(result: SettingSearchEntry): boolean {
    const activeResult = flatFuzzyResults[highlightedResultIndex];
    if (!activeResult) return false;
    return getResultKey(activeResult) === getResultKey(result);
}

function handleResultHover(result: SettingSearchEntry) {
    const hoveredResultKey = getResultKey(result);
    const nextIndex = flatFuzzyResults.findIndex(
        (candidate) => getResultKey(candidate) === hoveredResultKey
    );
    if (nextIndex >= 0) {
        highlightedResultIndex = nextIndex;
    }
}

async function scrollHighlightedResultIntoView() {
    await tick();
    if (!isSearchDropdownOpen || !searchDropdownElement) return;

    const resultButtons = searchDropdownElement.querySelectorAll<HTMLButtonElement>(
        "button[data-search-result='true']"
    );
    const activeElement = resultButtons[highlightedResultIndex];
    if (!activeElement) return;

    const viewTop = searchDropdownElement.scrollTop;
    const viewBottom = viewTop + searchDropdownElement.clientHeight;
    const itemTop = activeElement.offsetTop;
    const itemBottom = itemTop + activeElement.offsetHeight;

    if (itemTop < viewTop) {
        searchDropdownElement.scrollTop = itemTop;
        return;
    }

    if (itemBottom > viewBottom) {
        searchDropdownElement.scrollTop = itemBottom - searchDropdownElement.clientHeight;
    }
}

async function selectResult(entry: SettingSearchEntry) {
    isSearchDropdownOpen = false;
    try {
        await onSelect(entry);
    } finally {
        blurFocusedSearchElement();
    }
}

async function handleSearchSubmit(event: SubmitEvent) {
    event.preventDefault();
    if (flatFuzzyResults.length > 0) {
        await selectResult(flatFuzzyResults[highlightedResultIndex] ?? flatFuzzyResults[0]);
    }
}

</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<form
  {@attach setSearchForm}
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
      bind:value={searchQuery}
      {@attach setSearchInput}
      {placeholder}
      class="w-full appearance-none rounded-md border bg-input px-3 py-2 text-sm placeholder:text-center focus:outline-none md:min-w-72"
      aria-label={placeholder}
      oninput={() => {
        highlightedResultIndex = 0;
      }}
      onfocus={() => {
        isSearchDropdownOpen = searchQuery.trim().length > 0;
        if (isSearchDropdownOpen) {
            void scrollHighlightedResultIntoView();
        }
      }}
      onkeydown={handleSearchKeydown}
    />
    {#if searchQuery.trim().length > 0}
      <button
        type="button"
        class="bg-input absolute right-2 top-1/2 -translate-y-1/2 rounded-md border px-3 py-1 text-xs hover:bg-secondary"
        onclick={() => {
          searchQuery = "";
          highlightedResultIndex = 0;
          isSearchDropdownOpen = false;
        }}
      >
        {clearLabel}
      </button>
    {/if}
  </div>

  {#if isSearchDropdownOpen && searchQuery.trim().length > 0}
    <div
      bind:this={searchDropdownElement}
      class="absolute left-0 top-full z-30 mt-2 max-h-[60vh] w-full overflow-y-auto rounded-lg border bg-popover p-2 shadow-lg md:min-w-md"
    >
      {#if groupedFuzzyResults.length > 0}
        {#each groupedFuzzyResults as group (group.sectionId)}
          <div class="mb-2 border-t border-border/60 pt-1 first:border-t-0 first:pt-0 last:mb-0">
            <p class="mb-1 mt-1 px-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground first:mt-0">
              {group.sectionLabel}
            </p>
            <div class="space-y-1">
              {#each group.items as result (getResultKey(result))}
                <button
                  type="button"
                  data-search-result="true"
                  data-search-result-key={getResultKey(result)}
                  class={{
                    "block w-full rounded-md px-2 py-1.5 text-left text-sm": true,
                    "bg-accent text-accent-foreground": isHighlightedResult(result),
                    "hover:bg-secondary/70": !isHighlightedResult(result)
                  }}
                  onclick={async () => {
                    await selectResult(result);
                  }}
                  onmouseenter={() => {
                    handleResultHover(result);
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
          {noResultsLabel}
        </p>
      {/if}
    </div>
  {/if}
</form>
