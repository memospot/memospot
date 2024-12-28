<script lang="ts">
let {
	sections,
	activeSection,
	onSectionChange,
}: {
	sections: Array<{ id: string; label: string; icon?: any }>;
	activeSection: string;
	onSectionChange: (sectionId: string) => void;
} = $props();
</script>

<nav
  class="m-auto flex flex-col space-y-1 md:flex-row md:space-x-1 md:space-y-0 motion-preset-fade"
>
  {#each sections as section}
    <button
      class="w-56 px-4 py-2 text-lg rounded-2xl transition-colors hover:translate-y-[-1px] hover:drop-shadow
                    {activeSection === section.id
        ? 'bg-secondary text-secondary-foreground border'
        : 'hover:bg-secondary/80 text-muted-foreground'}"
      onclick={() => onSectionChange(section.id)}
    >
      <div class="flex flex-row items-start">
        {#if section.icon}
          <section.icon
            class="self-center w-6 mr-4 opacity-70 shrink-0 motion-preset-pop
"
          />
        {/if}
        {section.label}
      </div>
    </button>
  {/each}
</nav>
