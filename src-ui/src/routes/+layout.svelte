<script lang="ts">
import "../app.css";
import { i18n, initI18n } from "$lib/i18n";
import { getAppTheme } from "$lib/tauri";
import { ParaglideJS } from "@inlang/paraglide-sveltekit";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ModeWatcher, setMode } from "mode-watcher";
import { onMount } from "svelte";

type Theme = "system" | "light" | "dark";

let { children } = $props();

onMount(async () => {
    const initialAppTheme = ((await getAppTheme()) ||
        localStorage.getItem("mode-watcher-mode") ||
        "system") as Theme;

    if (initialAppTheme !== ("system" as Theme)) {
        setMode(initialAppTheme as Theme);
    }
    // All WebView windows are created in a hidden state to prevent flashing unstyled content.
    // This shows the window whenever the component is mounted.
    // It causes a delay to the window creation, but it's the best approach for now.
    await getCurrentWebviewWindow().show();
});
</script>

<ModeWatcher />

<ParaglideJS {i18n}>
  {#await initI18n() then _}
    {@render children?.()}
  {/await}
</ParaglideJS>
