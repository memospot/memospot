type TimerFunction = (tick: number) => void;
type Timer = ReturnType<typeof setTimeout>;

/**
 * Create a blinking effect on an image.
 *
 * - Works in a loop, toggling a pre-existing drop-shadow CSS filter from the supplied element.
 * - When `stopWithError()` is called, sets the value of element's first attribute whose qualified name is errorAttribute.qualifiedName to errorAttribute.value.
 *
 * @param {HTMLImageElement | string } selector - The element selector to apply the effect on.
 * @param errorAttribute.qualifiedName - Default: "id"
 * @param errorAttribute.value - Default: "error"
 */
export class LogoBlinker {
    private running: boolean;
    private interval: number | Timer;
    private element: HTMLImageElement;
    private errorAttribute: { qualifiedName: string; value: string };
    private tick: number;
    private readonly filter: string;

    constructor(selector: string, errorAttribute?: { qualifiedName: string; value: string }) {
        const querySelector = document.querySelector<HTMLImageElement>(selector);
        if (!(querySelector instanceof HTMLImageElement)) {
            throw new Error("Invalid selector");
        }
        this.element = querySelector;

        this.running = false;
        this.interval = 0;
        this.errorAttribute = errorAttribute || { qualifiedName: "id", value: "error" };
        this.tick = 0;
        this.filter = this.element.style.filter;
    }

    start(): void {
        if (this.running) {
            return;
        }

        const blink: TimerFunction = () => {
            this.element.style.filter = this.tick % 2 === 0 ? "none" : this.filter;
            this.tick = this.tick > 2 ? 0 : this.tick + 1;
        };

        this.interval = setInterval(blink, 1000);
        this.running = true;
    }

    stop(): void {
        this.running = false;
        clearInterval(this.interval);
    }

    stopWithError(): void {
        this.stop();
        this.element.removeAttribute("style");
        this.element.setAttribute(this.errorAttribute.qualifiedName, this.errorAttribute.value);
    }
}
