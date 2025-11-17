<script lang="ts">
import "../app.css";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ModeWatcher, modeStorageKey, setMode, systemPrefersMode } from "mode-watcher";
import { onMount } from "svelte";
import { page } from "$app/state";
import { initI18n, locales, localizeHref } from "$lib/i18n";
import { getAppTheme, getReduceAnimationStatus } from "$lib/tauri";

type Theme = "system" | "light" | "dark";

let { children } = $props();

onMount(async () => {
    // Initialize theme
    let initialAppTheme = ((await getAppTheme()) ??
        localStorage.getItem(modeStorageKey.current) ??
        "system") as Theme;

    if (initialAppTheme === "system") {
        initialAppTheme = systemPrefersMode.current || "light";
    }

    // Apply the initial theme
    setMode(initialAppTheme);
    // document.documentElement.setAttribute(
    //     "data-theme",
    //     initialAppTheme === "system"
    //         ? window.matchMedia("(prefers-color-scheme: dark)").matches
    //             ? "dark"
    //             : "light"
    //         : initialAppTheme
    // );

    // Handle reduce motion preference
    await getReduceAnimationStatus().then(async (reduceAnimation) => {
        const stored = localStorage.getItem("reduce-animation");
        if (!stored || JSON.parse(stored) !== reduceAnimation) {
            localStorage.setItem("reduce-animation", JSON.stringify(reduceAnimation));
        }
    });

    // Initialize i18n
    await initI18n();

    // Show the window after everything is initialized
    await getCurrentWebviewWindow().show();
});
</script>

<ModeWatcher />

<!--
The "invisible" anchor tags allow SvelteKit to generate all pages during build time.
-->
<div style="display: none;">
  {#each locales as locale}
    <a href={localizeHref(page.url.pathname, { locale })}>{locale}</a>
  {/each}
</div>

{@render children?.()}
