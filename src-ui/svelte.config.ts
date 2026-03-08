// This file is used by SvelteKit to configure the build process and other settings for the Svelte application.
//
// Note: this file should be named svelte.config.js, but running everything via bun makes it work as typescript.
//
// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import type { Config } from "@sveltejs/kit";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

const config: Config = {
    runes: true,
    preprocess: vitePreprocess(),
    kit: {
        adapter: adapter(),
        embedded: true
    }
};

export default config;
