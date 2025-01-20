<script lang="ts">
import "../app.css";
import { detectLanguage, i18n } from "$lib/i18n";
import { getAppConfig, getAppLanguage } from "$lib/tauri";
import type { Config } from "$lib/types/gen/Config";
import { ParaglideJS } from "@inlang/paraglide-sveltekit";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ModeWatcher, setMode } from "mode-watcher";
import { onMount } from "svelte";

let { children } = $props();

type Theme = "system" | "light" | "dark";

onMount(async () => {
	const initialJSON = await getAppConfig();
	const initialConfig: Config = JSON.parse(initialJSON);

	const theme = (initialConfig.memospot.window.theme ||
		localStorage.getItem("mode-watcher-mode") ||
		"system") as Theme;

	if (theme !== ("system" as Theme)) {
		setMode(theme);
	}

	// All WebView windows are created in a hidden state to prevent flashing unstyled content.
	// This shows the window whenever the component is mounted.
	// It causes a delay to the window creation, but it's the best approach for now.
	(async () => {
		await getCurrentWebviewWindow().show();
	})();
});
</script>

<ModeWatcher />

{#await getAppLanguage() then appLanguage}
  <ParaglideJS {i18n}>
    {console.log(`appLanguage: ${appLanguage}`)}
    {detectLanguage(appLanguage)}
    {@render children?.()}
  </ParaglideJS>
{/await}
