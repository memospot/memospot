import { defineConfig } from "vite";
import htmlMinifier from "vite-plugin-html-minifier";

// https://vitejs.dev/config/
export default defineConfig({
    build: {
        emptyOutDir: true,
        outDir: "../dist-ui",
        target: ["es2021", "chrome97", "safari13"],
        minify: !process.env.TAURI_DEBUG ? "terser" : false,
        sourcemap: !!process.env.TAURI_DEBUG
    },
    // prevent vite from obscuring rust errors
    clearScreen: false,
    // Tauri expects a fixed port, fail if that port is not available
    server: {
        port: 5173,
        strictPort: true,
        fs: {
            allow: ["."]
        }
    },
    plugins: [
        htmlMinifier({
            minify: true
        })
    ]
});
