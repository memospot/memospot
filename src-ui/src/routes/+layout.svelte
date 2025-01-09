<script lang="ts">
import "../app.css";
import { detectLanguage, i18n } from "$lib/i18n";
import { ParaglideJS } from "@inlang/paraglide-sveltekit";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ModeWatcher } from "mode-watcher";
import { onMount } from "svelte";

let { children } = $props();

onMount(() => {
	// All WebView windows are created in a hidden state to prevent flashing unstyled content.
	// This shows the window whenever the component is mounted.
	// It causes a delay to the window creation, but it's the best approach for now.
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
