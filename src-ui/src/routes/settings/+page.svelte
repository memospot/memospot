<script lang="ts">
import { m } from "$lib/i18n";
import EyeOpen from "svelte-radix/EyeOpen.svelte";
import Gear from "svelte-radix/Gear.svelte";
import Pencil2 from "svelte-radix/Pencil2.svelte";
import Advanced from "./Advanced.svelte";
import Memos from "./Memos.svelte";
import Navbar from "./Navbar.svelte";
import View from "./View.svelte";

// Sections data.
const sections: Array<{
	id: string;
	label: string;
	icon?: any;
	component: any;
}> = [
	{ id: "memos", label: "Memos", icon: Pencil2, component: Memos },
	{ id: "view", label: m.view(), icon: EyeOpen, component: View },
	{ id: "advanced", label: m.advanced(), icon: Gear, component: Advanced },
];

let activeSection: string = $state(
	sections.find((s) => s.id === window.location.hash.slice(1))?.id || sections[0].id,
);

const animateSection = () => {
	const sectionAnimation = "motion-preset-fade";
	const mainSelector = document.querySelector("main");

	mainSelector?.classList.add(sectionAnimation);
	setTimeout(() => {
		mainSelector?.classList.remove(sectionAnimation);
	}, 800);
};

const updateSection = (sectionId: string) => {
	activeSection = sectionId;
	window.location.hash = `#${sectionId}`;
	animateSection();
};
</script>

<div class="container p-4 min-w-screen motion-preset-fade">
  <div class="flex flex-col gap-4">
    <!-- Navbar -->
    <Navbar {sections} {activeSection} onSectionChange={updateSection} />

    <!-- Main content -->
    <main class="flex-1 w-full">
      {#each sections as section}
        {#if activeSection === section.id}
          <section.component class="w-6 h-auto opacity-70 shrink-0" />
        {/if}
      {/each}
    </main>
  </div>
</div>
