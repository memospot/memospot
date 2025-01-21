<script lang="ts">
import { m } from "$lib/i18n";
import { getEnv, getMemosURL, pingMemos } from "$lib/tauri";
import { cn } from "$lib/utils";
import { isTauri } from "@tauri-apps/api/core";
import { userPrefersMode } from "mode-watcher";
import { onMount } from "svelte";
import Update from "svelte-radix/Update.svelte";
import { MediaQuery } from "svelte/reactivity";

const CONFIG = {
	MAX_RETRIES: 10,
	RETRY_DELAY_MS: 300,
	EXPONENTIAL_BACKOFF: true,
	DEBUG_NO_REDIRECT: !isTauri(),
};

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

let redirectDetails = $state({
	isError: false,
	isLocalhost: false,
	memosUrl: "",
	displayUrl: "",
	replyMs: 0,
	pingStartTime: 0,
	retries: 0,
});

async function redirectWhenReady() {
	const debugNoRedirect =
		CONFIG.DEBUG_NO_REDIRECT ||
		["true", "on", "1"].includes((await getEnv("MEMOSPOT_NO_REDIRECT")).toLowerCase());

	const memosUrl = await getMemosURL().then((url) =>
		url.endsWith("/") ? url.slice(0, -1) : url,
	);
	redirectDetails.memosUrl = memosUrl;
	redirectDetails.isLocalhost = memosUrl.startsWith("http://localhost");
	redirectDetails.displayUrl =
		memosUrl.replace("http://", "").replace("https://", "") || "_URL_";
	redirectDetails.pingStartTime = Date.now();

	const updateMs = () => {
		redirectDetails.replyMs = Date.now() - redirectDetails.pingStartTime;
	};

	for (
		redirectDetails.retries = 0;
		redirectDetails.retries < CONFIG.MAX_RETRIES;
		redirectDetails.retries++
	) {
		if (!debugNoRedirect && (await pingMemos(memosUrl))) {
			updateMs();

			globalThis.location.replace(memosUrl);
			return redirectDetails;
		}
		await new Promise((resolve) => {
			updateMs();
			setTimeout(
				resolve,
				CONFIG.RETRY_DELAY_MS * (CONFIG.EXPONENTIAL_BACKOFF ? redirectDetails.retries + 1 : 1),
			);
		});
	}

	redirectDetails.isError = true;

	return redirectDetails;
}

onMount(async () => {
	redirectDetails = await redirectWhenReady();
});
</script>

<main
  class="absolute h-full w-full flex flex-col items-center justify-center text-zinc-700 dark:text-zinc-300 motion-preset-fade"
>
  <div>
    <h1 id="status" class="text-xl">
      {redirectDetails.isError ? m.settingsSomethingWentWrong() : m.loaderWaitingForServer()}
    </h1>
  </div>
  <div>
    <a
      href="https://usememos.com/"
      target="_blank"
      referrerpolicy="origin"
      title={m.loaderClickToOpenMemosWebsite()}
    >
      <img
        src="powered_by_memos{theme === 'dark' ? '_dark' : ''}.webp"
        class="!h-60 p-6 logo logo-glow {redirectDetails.isError
          ? 'error'
          : ''}"
        alt="Memos"
      />
    </a>
  </div>
  <div class="text-center">
    <p class="text-sm text-inherit opacity-50">
      <a
        class="hover:underline hover:text-cyan-500"
        href={redirectDetails.memosUrl}
        target="_blank">{redirectDetails.displayUrl}</a
      >
    </p>
    {#if redirectDetails.isError}
      <p class="mt-2">
        {m.loaderNoResponse()}
        <br />
        {#if redirectDetails.isLocalhost}
          {m.loaderTryRestartingMemospot()}
        {:else}
          {@html m.loaderErrRemoteServerNotReachable()}
        {/if}
      </p>
      <div class="mt-4 flex flex-row gap-2 justify-center">
        <button
          title={m.loaderCheckAgainIfServerStarted()}
          class="w-fit px-4 py-2 text-lg rounded-2xl transition-colors hover:bg-secondary/80 text-muted-foreground border hover:translate-y-[-1px] hover:drop-shadow"
          onclick={() => window.location.replace(window.location.href)}
        >
          <Update
            class="h-[1.2rem] w-[1.2rem] m-1 motion-safe:animate-pulse hover:animate-none"
          />
        </button>
      </div>
    {/if}
    <p
      class={cn(
        "mt-2",
        redirectDetails.replyMs > 0 ? "opacity-70" : "opacity-0",
        redirectDetails.retries === 0
          ? "text-primary"
          : redirectDetails.isError
            ? "text-destructive"
            : "text-amber-600",
      )}
    >
      {redirectDetails.replyMs} ms
    </p>
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
    padding: 3.5em; /* Prevent square box on webkit */
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
