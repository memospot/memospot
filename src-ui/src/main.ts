const globalThis = window;

const invoke = ((func: string, ...args: any[]) => {
    if (globalThis.__TAURI__) {
        return globalThis.__TAURI__.tauri.invoke(func, ...args);
    }
    // Used to test the UI in a browser.
    return new Error("Not running in Tauri");
}) as any;

document.addEventListener("DOMContentLoaded", async () => {
    const reload = () => {
        globalThis.location.replace("/");
    };
    const button = document.getElementById("manual-redirect-btn")!;
    button.addEventListener("click", reload);

    const memosPort = (await invoke("js_get_memos_port")) as number;
    const element = document.querySelector<HTMLParagraphElement>("#port")!;
    element.textContent = "Port: " + memosPort;
});

const logoBlinker = {
    running: false,
    interval: 0,
    element: document.querySelector<HTMLImageElement>(".logo.memos")!,
    start() {
        if (this.running) {
            return;
        }
        this.running = true;

        const filter = this.element.style.filter;
        let tick = 0;
        this.interval = setInterval(() => {
            this.element.style.filter = tick % 2 == 0 ? "none" : filter;
            tick = tick > 2 ? 0 : tick + 1;
        }, 1000);
    },
    stop() {
        this.running = false;
        clearInterval(this.interval);
    },
    stopWithError() {
        this.running = false;
        clearInterval(this.interval);
        this.element.removeAttribute("style");
        this.element.setAttribute("id", "error");
    },
};

async function pingMemosServer(endpoint: string): Promise<boolean> {
    try {
        const response = await fetch(endpoint);
        return response.status === 200;
    } catch (_) {
        return false;
    }
}

async function redirectOnResponse() {
    const memosPort = await invoke("get_memos_port");
    const pingAPI = `api/v1/ping`;
    const memosUrl = `http://localhost:${memosPort}`;
    const MemosPingEndpoint = [memosUrl, pingAPI].join("/");

    logoBlinker.start();

    let tries = 0;
    while (true) {
        if (await pingMemosServer(MemosPingEndpoint)) {
            logoBlinker.stop();
            globalThis.location.replace(memosUrl);
            break;
        }

        if (tries > 10) {
            logoBlinker.stopWithError();

            const noResponseError =
                "Server did not respond in a feasible time.<br />Try restarting Memospot.";

            const msg = document.querySelector<Element>("#msg")!;
            msg.innerHTML = noResponseError;

            const waiting = document.querySelector<Element>(".waiting-for-server")!;
            waiting.innerHTML = "Something went wrong 😢";

            const button = document.getElementById("manual-redirect-btn")!;
            button.setAttribute("style", "visibility: visible;");

            break;
        }
        tries++;
        await new Promise((resolve) => setTimeout(resolve, 1000));
    }
}
document.addEventListener("DOMContentLoaded", redirectOnResponse);
