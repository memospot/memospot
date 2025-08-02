<script lang="ts">
import EyeOpen from "svelte-radix/EyeOpen.svelte";
import Gear from "svelte-radix/Gear.svelte";
import Pencil2 from "svelte-radix/Pencil2.svelte";
import { Toaster } from "$lib/components/ui/sonner";
import { m } from "$lib/i18n";
import { cn } from "$lib/utils";
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

const reduceAnimation = JSON.parse(localStorage.getItem("reduce-animation") ?? "false");

async function animateSectionTransition() {
    const sectionAnimation = "motion-preset-fade";
    const mainSelector = document.querySelector("main");

    mainSelector?.classList.add(sectionAnimation);
    await new Promise(() => {
        setTimeout(() => {
            mainSelector?.classList.remove(sectionAnimation);
        }, 800);
    });
}

async function updateSection(sectionId: string) {
    activeSection = sectionId;
    window.location.hash = `#${sectionId}`;
    if (!reduceAnimation) await animateSectionTransition();
}
</script>

<div
  class={cn(
    "container p-4 min-w-screen",
    reduceAnimation ? "" : "motion-preset-fade",
  )}
>
  <div class="flex flex-col gap-4">
    <Navbar
      {sections}
      {activeSection}
      onSectionChange={async (sectionId) => await updateSection(sectionId)}
    />

    <main class="flex-1 w-full">
      {#each sections as section}
        {#if activeSection === section.id}
          <section.component class="w-6 h-auto opacity-70 shrink-0" />
        {/if}
      {/each}
    </main>
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
