import { assertEquals } from "./deps.ts";
import { makeTripletFromFileName } from "./downloadMemosBuildsHook.ts";

Deno.test("makeTripletFromFileName", () => {
    const goToRustMap: Record<string, string> = {
        "darwin-arm64": "aarch64-apple-darwin",
        "darwin-x86_64": "x86_64-apple-darwin",
        "darwin-x86_64_v2": "x86_64-apple-darwin",
        "darwin-x86_64_v3": "x86_64-apple-darwin",
        "windows-x86_64": "x86_64-pc-windows-msvc",
        "windows-x86_64_v2": "x86_64-pc-windows-msvc",
        "windows-x86_64_v3": "x86_64-pc-windows-msvc",
        "windows-arm64": "aarch64-pc-windows-msvc",
        "linux-x86_64": "x86_64-unknown-linux-gnu",
        "linux-x86_64_v2": "x86_64-unknown-linux-gnu",
        "linux-x86_64_v3": "x86_64-unknown-linux-gnu",
        "linux-riscv64": "riscv64gc-unknown-linux-gnu",
        "linux-i386": "i686-unknown-linux-gnu",
        "linux-arm64": "aarch64-unknown-linux-gnu",
    };

    const randomPrefix = Math.random().toString(36).substring(7);
    for (const [key, value] of Object.entries(goToRustMap)) {
        const goOsGoArch = [randomPrefix, value].join("-");
        const rustTriplet = [randomPrefix, makeTripletFromFileName(key)].join("-");
        assertEquals(goOsGoArch, rustTriplet);
    }
});
