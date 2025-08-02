<script lang="ts">
import "../app.css";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ModeWatcher, setMode } from "mode-watcher";
import { onMount } from "svelte";
import { initI18n } from "$lib/i18n";
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

    localStorage.setItem("reduce-animation", JSON.stringify(await getReduceAnimationStatus()));

    // All WebView windows are created in a hidden state to prevent flashing unstyled content.
    // This shows the window whenever the component is mounted.
    // It causes a delay to the window creation, but it's the best approach for now.
    await getCurrentWebviewWindow().show();
});
</script>

<ModeWatcher />

{#await initI18n() then _}
  {@render children?.()}
{/await}
