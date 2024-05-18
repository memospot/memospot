import { ResponseType, fetch } from "@tauri-apps/api/http";
import { invoke as TauriInvoke } from "@tauri-apps/api/tauri";
import { LogoBlinker } from "./blinker";

const browserError = new Error("Not running in Tauri!");
const invoke = window.__TAURI__ ? TauriInvoke : async () => browserError.message;

document.addEventListener("DOMContentLoaded", async () => {
    const reload = () => {
        window.location.replace("/");
    };

    const button = document.getElementById("manual-redirect-btn");
    if (button instanceof HTMLElement) {
        button.addEventListener("click", reload);
    }

    const element = document.querySelector<HTMLParagraphElement>("#port");
    if (element instanceof HTMLParagraphElement) {
        const memosPort = (await invoke("get_memos_port")) as number;
        element.textContent = `Port: ${memosPort}`;
    }
});

async function pingMemosServer(endpoint: string): Promise<boolean> {
    try {
        const response = await fetch(endpoint, {
            method: "GET",
            timeout: 1,
            responseType: ResponseType.Text
        });
        return response.ok;
    } catch (_) {
        return false;
    }
}

async function redirectOnResponse() {
    const memosPort = await invoke("get_memos_port");
    const pingAPI = "healthz";
    const MemosUrl = `http://localhost:${memosPort}`;
    const PingEndpoint = [MemosUrl, pingAPI].join("/");

    const noRedirectEnv = (await invoke("get_env", { name: "NO_REDIRECT" })) as string;
    const DebugNoRedirect = noRedirectEnv.toLowerCase() === "true" || noRedirectEnv === "1";

    const logoBlinker = new LogoBlinker(".logo.memos");
    logoBlinker.start();

    let tries = 0;
    while (true) {
        if ((await pingMemosServer(PingEndpoint)) && !DebugNoRedirect) {
            logoBlinker.stop();
            globalThis.location.replace(MemosUrl);
            break;
        }

        if (tries > 30) {
            logoBlinker.stopWithError();

            const msg = document.querySelector<Element>("#msg");
            if (msg instanceof Element) {
                msg.innerHTML =
                    "The server did not respond within a reasonable time.<br />Try restarting Memospot.";
            }

            const waiting = document.querySelector<Element>(".waiting-for-server");
            if (waiting instanceof Element) {
                waiting.innerHTML = "Something went wrong 😢";
            }

            const button = document.getElementById("manual-redirect-btn");
            if (button instanceof Element) {
                button.setAttribute("style", "visibility: visible;");
            }

            break;
        }

        tries++;
        await new Promise((resolve) => setTimeout(resolve, 1000));
    }
}
document.addEventListener("DOMContentLoaded", redirectOnResponse);
