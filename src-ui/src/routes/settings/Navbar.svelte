<script lang="ts">
import type { Component } from "svelte";
import { cn } from "$lib/utils";

export type Section = {
    id: string;
    label: string;
    icon?: Component;
    component: Component;
};

let {
    sections,
    activeSection,
    onSectionChange
}: {
    sections: Section[];
    activeSection: string;
    onSectionChange: (sectionId: string) => void;
} = $props();

const reduceAnimation = JSON.parse(localStorage.getItem("reduce-animation") ?? "false");
</script>

<nav
  class={cn(
    "m-auto flex flex-col space-y-1 md:flex-row md:space-x-1 md:space-y-0",
    reduceAnimation ? "" : "motion-preset-fade",
  )}
>
  {#each sections as section}
    <div
      role="button"
      tabindex="0"
      onclick={() => onSectionChange(section.id)}
      onkeypress={async (e) =>
      ["Enter", " "].includes(e.key) && onSectionChange(section.id)}
      class={cn(
        "w-56 px-4 py-2 text-lg rounded-2xl transition-colors hover:translate-y-[-1px] hover:drop-shadow",
        activeSection === section.id
          ? "bg-secondary text-secondary-foreground border"
          : "hover:bg-secondary/80 text-muted-foreground",
      )}
    >
      <div class="flex flex-row items-start">
        {#if section.icon}
          <section.icon
            class={cn(
              "self-center w-6 mr-4 opacity-70 shrink-0",
              reduceAnimation ? "" : "motion-preset-pop",
            )}
          />
        {/if}
        {section.label}
      </div>
    </div>
  {/each}
</nav>
