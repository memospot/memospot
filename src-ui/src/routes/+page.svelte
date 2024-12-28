<script lang="ts">
import { getEnv, getMemosURL, pingMemos } from "$lib/tauri";
import { userPrefersMode } from "mode-watcher";
import { onMount } from "svelte";
import Update from "svelte-radix/Update.svelte";
import { MediaQuery } from "svelte/reactivity";

import { m } from "$lib/i18n";
import { isTauri } from "@tauri-apps/api/core";

const CONFIG = {
	MAX_RETRIES: 30,
	RETRY_DELAY_MS: 200,
	EXPONENTIAL_BACKOFF: false,
	DEBUG_NO_REDIRECT: !isTauri(),
};

let connectionStatus = $state({
	isError: false,
	isLocalhost: false,
	displayUrl: "",
	replyMs: 0,
});
let theme = $state("system");

userPrefersMode.subscribe((value) => {
	theme = value;
});

const darkTheme = new MediaQuery("prefers-color-scheme: dark");
$effect(() => {
	if (theme === "system") {
		theme = darkTheme.current ? "dark" : "light";
	}
	document.documentElement.setAttribute("data-theme", theme);
});

onMount(async () => {
	const debugNoRedirect =
		CONFIG.DEBUG_NO_REDIRECT ||
		["true", "on", "1"].includes((await getEnv("MEMOSPOT_NO_REDIRECT")).toLowerCase());

	const memosUrl = await getMemosURL().then((url) =>
		url.endsWith("/") ? url.slice(0, -1) : url,
	);
	connectionStatus.isLocalhost = memosUrl.startsWith("http://localhost");
	connectionStatus.displayUrl = memosUrl.replace("http://", "").replace("https://", "");

	const startTime = Date.now();
	for (let tries = 0; tries < CONFIG.MAX_RETRIES; tries++) {
		if (!debugNoRedirect && (await pingMemos())) {
			connectionStatus.replyMs = Date.now() - startTime;
			globalThis.location.replace(memosUrl);
			return;
		}
		await new Promise((resolve) =>
			setTimeout(resolve, CONFIG.RETRY_DELAY_MS * (CONFIG.EXPONENTIAL_BACKOFF ? tries + 1 : 1)),
		);
	}

	connectionStatus.replyMs = Date.now() - startTime;
	connectionStatus.isError = true;
});
</script>

<main
  class="absolute h-full w-full flex flex-col items-center justify-center text-zinc-700 dark:text-zinc-300 motion-preset-fade"
>
  <div>
    <h1 id="status" class="text-xl">
      {connectionStatus.isError ? m.somethingWentWrong() : m.waitingForServer()}
    </h1>
  </div>
  <div>
    <a
      href="https://usememos.com/"
      target="_blank"
      referrerpolicy="origin"
      title={m.clickToOpenMemosWebsite()}
    >
      <img
        src="powered_by_memos{theme === 'dark' ? '_dark' : ''}.webp"
        class="!h-44 p-6 logo logo-glow {connectionStatus.isError
          ? 'error'
          : ''}"
        alt="Memos"
      />
    </a>
  </div>
  <div class="text-center">
    {#await getMemosURL() then memosUrl}
      <p class="text-sm text-inherit opacity-50">
        <a
          class="hover:underline hover:text-cyan-500"
          href={memosUrl}
          target="_blank">{connectionStatus.displayUrl || "_URL_"}</a
        >
      </p>
      {#if connectionStatus.isError}
        <p class="mt-2">
          {m.noResponse()}
          <br />
          {#if connectionStatus.isLocalhost}
            {m.tryRestartingMemospot()}
          {:else}
            {@html m.errRemoteServerNotReachable()}
          {/if}
        </p>
        <div class="mt-4 flex flex-row gap-2 justify-center">
          <button
            title={m.checkAgainIfServerStarted()}
            class="w-fit px-4 py-2 text-lg rounded-2xl transition-colors hover:bg-secondary/80 text-muted-foreground border hover:translate-y-[-1px] hover:drop-shadow"
            onclick={() => window.location.replace("/")}
          >
            <Update
              class="h-[1.2rem] w-[1.2rem] m-1 motion-safe:animate-pulse hover:animate-none"
            />
          </button>
        </div>
        <p class="mt-2 text-amber-600 opacity-50">
          {connectionStatus.replyMs} ms
        </p>
      {:else if connectionStatus.replyMs > 0}
        <p class="mt-2 text-emerald-600 opacity-50">
          {connectionStatus.replyMs} ms
        </p>
      {/if}
    {/await}
  </div>
</main>

<style>
  img {
    image-rendering: high-quality;
  }
  @keyframes glow {
    0%,
    100% {
      filter: drop-shadow(0 0 4vh var(--glow));
    }
    50% {
      filter: none;
    }
  }
  .logo-glow {
    animation: glow 2s ease-in-out infinite;
    transition:
      filter 1s ease-in-out,
      transform 0.3s ease-in-out;
    will-change: filter, transform;
  }
  .logo-glow.error {
    animation: 0.5s ease-in-out;
    filter: drop-shadow(0 0 4vh var(--glow-error));
  }
  .logo-glow:hover {
    transform: scale(1.02);
  }
</style>
