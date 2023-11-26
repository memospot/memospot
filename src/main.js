const { invoke } = window.__TAURI__.tauri;

const memosPort = await invoke("js_get_memos_port");
const MemosServerAddress = "http://localhost:" + memosPort;
const MemosApiPing = "/api/v1/ping";

document.querySelector("#port").textContent = "Port: " + memosPort;

function blinkLogo() {
    const logo = document.querySelector(".logo.memos");
    const cssFilter = logo.style.filter;
    let tick = 0;
    setInterval(() => {
        logo.style.filter =
            tick % 2 == 0 ? "drop-shadow(0 0 0 #0000)" : cssFilter;
        tick = tick > 2 ? 0 : tick + 1;
    }, 1000);
}

async function pingMemosServer() {
    try {
        const response = await fetch(MemosServerAddress + MemosApiPing);
        return response.status === 200;
    } catch (err) {
        return false;
    }
}

blinkLogo();

let tries = 0;
while (true) {
    if (await pingMemosServer()) {
        window.location.replace(MemosServerAddress);
        break;
    }

    if (tries > 10) {
        let msg = document.querySelector("#msg");
        msg.innerHTML =
            "Server did not respond within 10 seconds.<br />Try relaunching this application.";
        break;
    }
    tries++;
    await new Promise((resolve) => setTimeout(resolve, 1000));
}
