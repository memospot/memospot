<script lang="ts">
import { Toaster } from "$lib/components/ui/sonner";
import { m } from "$lib/i18n";
import type { Component } from "svelte";
import EyeOpen from "svelte-radix/EyeOpen.svelte";
import Gear from "svelte-radix/Gear.svelte";
import Pencil2 from "svelte-radix/Pencil2.svelte";
import Memos from "./Memos.svelte";
import Memospot from "./Memospot.svelte";
import Navbar from "./Navbar.svelte";
import View from "./View.svelte";

const sections: Array<{
	id: string;
	label: string;
	icon?: Component;
	component: Component;
}> = [
	{ id: "view", label: m.view(), icon: EyeOpen, component: View },
	{ id: "memos", label: m.memos(), icon: Pencil2, component: Memos },
	{ id: "memospot", label: m.memospot(), icon: Gear, component: Memospot },
];

let activeSection: string = $state(
	sections.find((s) => s.id === window.location.hash.slice(1))?.id || sections[0].id,
);

function animateSectionTransition() {
	const sectionAnimation = "motion-preset-fade";
	const mainSelector = document.querySelector("main");

	mainSelector?.classList.add(sectionAnimation);
	setTimeout(() => {
		mainSelector?.classList.remove(sectionAnimation);
	}, 800);
}

function updateSection(sectionId: string) {
	activeSection = sectionId;
	window.location.hash = `#${sectionId}`;
	animateSectionTransition();
}
</script>

<div class="container p-4 min-w-screen motion-preset-fade">
  <div class="flex flex-col gap-4">
    <Navbar {sections} {activeSection} onSectionChange={updateSection} />

    <main class="flex-1 w-full">
      {#each sections as section}
        {#if activeSection === section.id}
          <section.component class="w-6 h-auto opacity-70 shrink-0" />
        {/if}
      {/each}
    </main>
  </div>
</div>
<Toaster duration={1500} visibleToasts={1}/>
