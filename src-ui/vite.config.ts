import { paraglideVitePlugin } from "@inlang/paraglide-js";
import { sveltekit } from "@sveltejs/kit/vite";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
    plugins: [
        paraglideVitePlugin({
            project: "./i18n",
            outdir: "./src/lib/paraglide",
            strategy: ["localStorage", "preferredLanguage", "baseLocale"]
        }),
        sveltekit()
    ],
    build: {
        emptyOutDir: true, // SvelteKit output is fixed at ./build
        target: ["es2021", "chrome97", "safari13"],
        sourcemap: !!process.env.TAURI_ENV_DEBUG,
        rollupOptions: {
            output: {
                manualChunks() {
                    // Lower the number of output files from ~30 to ~16.
                    return "vendor";
                }
            }
        }
    },
    preprocess: vitePreprocess(),
    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host
            ? {
                  protocol: "ws",
                  host,
                  port: 1421
              }
            : undefined,
        watch: {
            // 3. tell vite to ignore watching `crates/memospot`
            ignored: ["**/crates/memospot/**"]
        }
    }
}));
