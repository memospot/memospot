import { defineConfig } from "vite";
import htmlMinifier from "vite-plugin-html-minifier";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig({
    build: {
        emptyOutDir: true,
        outDir: "../dist-ui",
        target: ["es2021", "chrome97", "safari13"],
        minify: !process.env.TAURI_ENV_DEBUG ? "terser" : false,
        sourcemap: !!process.env.TAURI_ENV_DEBUG
    },
    // prevent vite from obscuring rust errors
    clearScreen: false,
    // Tauri expects a fixed port, fail if that port is not available
    server: {
        host: host || false,
        port: 1420,
        strictPort: true,
        hmr: host
            ? {
                  protocol: "ws",
                  host: host,
                  port: 1430
              }
            : undefined
    },
    plugins: [
        htmlMinifier({
            minify: true
        })
    ]
});
