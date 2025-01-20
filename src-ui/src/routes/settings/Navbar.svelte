<script lang="ts">
import { cn } from "$lib/utils";
import type { Component } from "svelte";

export type Section = {
	id: string;
	label: string;
	icon?: Component;
	component: Component;
};

let {
	sections,
	activeSection,
	onSectionChange,
}: {
	sections: Section[];
	activeSection: string;
	onSectionChange: (sectionId: string) => void;
} = $props();
</script>

<nav
  class="m-auto flex flex-col space-y-1 md:flex-row md:space-x-1 md:space-y-0 motion-preset-fade"
>
  {#each sections as section}
    <button
      class={cn(
        "w-56 px-4 py-2 text-lg rounded-2xl transition-colors hover:translate-y-[-1px] hover:drop-shadow",
        activeSection === section.id
          ? "bg-secondary text-secondary-foreground border"
          : "hover:bg-secondary/80 text-muted-foreground",
      )}
      onclick={() => onSectionChange(section.id)}
    >
      <div class="flex flex-row items-start">
        {#if section.icon}
          <section.icon
            class="self-center w-6 mr-4 opacity-70 shrink-0 motion-preset-pop"
          />
        {/if}
        {section.label}
      </div>
    </button>
  {/each}
</nav>
