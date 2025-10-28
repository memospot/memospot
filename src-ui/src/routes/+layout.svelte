<script lang="ts">
import "../app.css";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ModeWatcher, setMode } from "mode-watcher";
import { onMount } from "svelte";
import { page } from "$app/state";
import { initI18n, locales, localizeHref } from "$lib/i18n";
import { getAppTheme, getReduceAnimationStatus } from "$lib/tauri";

type Theme = "system" | "light" | "dark";

let { children } = $props();

onMount(async () => {
    const initialAppTheme = (((await getAppTheme()) ||
        localStorage.getItem("mode-watcher-mode")) ??
        "system") as Theme;

    if (initialAppTheme !== ("system" as Theme)) {
        setMode(initialAppTheme as Theme);
    }

    await getReduceAnimationStatus().then(async (reduceAnimation) => {
        const stored = localStorage.getItem("reduce-animation");
        if (!stored || JSON.parse(stored) !== reduceAnimation) {
            localStorage.setItem("reduce-animation", JSON.stringify(reduceAnimation));
        }
    });

    await initI18n();

    // All WebView windows are created in a hidden state to prevent flashing unstyled content.
    // This shows the window whenever the component is mounted.
    // It causes a delay to the window creation, but it's the best approach for now.
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
