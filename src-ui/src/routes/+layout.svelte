<script lang="ts">
import "../app.css";
import { detectLanguage, i18n } from "$lib/i18n";
import { ParaglideJS } from "@inlang/paraglide-sveltekit";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ModeWatcher } from "mode-watcher";
import { onMount } from "svelte";

let { children } = $props();

onMount(() => {
	// All webview windows are created in a hidden state to prevent the flash of unstyled content.
	// This shows the window whenever the component is mounted.
	// It leads to a delay in the window creation, but its the best approach for now.
	(async () => {
		await getCurrentWebviewWindow().show();
	})();
});
</script>

<ModeWatcher />

<ParaglideJS {i18n}>
  {detectLanguage()}
  {@render children?.()}
</ParaglideJS>
