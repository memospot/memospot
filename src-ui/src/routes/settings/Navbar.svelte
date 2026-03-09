<script lang="ts">
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
    onSectionChange
}: {
    sections: Section[];
    activeSection: string;
    onSectionChange: (sectionId: string) => void;
} = $props();

const reduceAnimation = JSON.parse(localStorage.getItem("reduce-animation") ?? "false");
</script>

<nav
  class={{
    "flex flex-col gap-1 pt-1 md:gap-2": true,
    "motion-preset-fade": !reduceAnimation
  }}
>
  {#each sections as section (section.id)}
    <div
      role="button"
      tabindex="0"
      onclick={() => onSectionChange(section.id)}
      onkeypress={async (e) =>
      ["Enter", " "].includes(e.key) && onSectionChange(section.id)}
      class={{
        "w-full rounded-xl px-3 py-2 text-left text-base whitespace-nowrap transition-colors hover:-translate-y-px hover:drop-shadow":
            true,
        "bg-secondary text-secondary-foreground border": activeSection === section.id,
        "hover:bg-secondary/80 text-muted-foreground": activeSection !== section.id
      }}
    >
      <div class="flex flex-row items-center">
        {#if section.icon}
          <section.icon
            class={{
              "mr-3 w-5 shrink-0 opacity-70": true,
              "motion-preset-pop": !reduceAnimation
            }}
          />
        {/if}
        {section.label}
      </div>
    </div>
  {/each}
</nav>
