import { LogoBlinker } from "./blinker";
import { getEnv, getMemosURL, pingMemos } from "./tauri";

async function addManualRedirectButton() {
    document.getElementById("manual-redirect-btn")?.addEventListener("click", () => {
        window.location.replace("/");
    });

    const urlElement = document.querySelector<HTMLParagraphElement>("#url");
    if (urlElement) {
        const memosUrl = await getMemosURL();
        let displayUrl = memosUrl.replace("http://", "").replace("https://", "");
        if (displayUrl.endsWith("/")) {
            displayUrl = displayUrl.slice(0, -1);
        }
        urlElement.innerHTML = `<a href="${memosUrl}" target="_blank">${displayUrl}</a>`;
    }
}
document.addEventListener("DOMContentLoaded", addManualRedirectButton);

async function redirectOnResponse() {
    let memosUrl = await getMemosURL();
    memosUrl = memosUrl.endsWith("/") ? memosUrl.slice(0, -1) : memosUrl;

    const noRedirectEnv = await getEnv("MEMOSPOT_NO_REDIRECT");
    const debugNoRedirect = ["true", "on", "1"].includes(noRedirectEnv.toLowerCase());
    const logoBlinker = new LogoBlinker(".logo.memos");
    logoBlinker.start();
    const isLocalhost = memosUrl.startsWith("http://localhost");

    let tries = 0;
    while (true) {
        if ((await pingMemos()) && !debugNoRedirect) {
            logoBlinker.stop();
            globalThis.location.replace(memosUrl);
            return;
        }

        if (tries === 30) {
            logoBlinker.stopWithError();

            const msgElement = document.querySelector<Element>("#msg");
            if (msgElement) {
                msgElement.innerHTML =
                    "The server did not respond within a reasonable time.<br />";
                msgElement.innerHTML += isLocalhost
                    ? "Try restarting Memospot."
                    : "Check your settings and ensure that Memos is reachable.";
            }

            const waitingElement = document.querySelector<Element>(".waiting-for-server");
            if (waitingElement instanceof Element) {
                waitingElement.innerHTML = "Something went wrong 😢";
            }

            const button = document.getElementById("manual-redirect-btn");
            if (button instanceof Element) {
                button.setAttribute("style", "visibility: visible;");
            }

            return;
        }

        tries++;
        await new Promise((resolve) => setTimeout(resolve, 1000));
    }
}
document.addEventListener("DOMContentLoaded", redirectOnResponse);
