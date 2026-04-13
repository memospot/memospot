/*
 * Reduce animation polyfill.
 *
 * Auto-injected when the user has enabled "Reduce motion" in their system settings.
 *
 * This polyfill removes the `animation` and `transition` CSS properties from the page,
 * as well as the `scroll-behavior: smooth` option from scroll methods, to reduce motion.
 *
 * This greatly improves the user experience on Linux.
 */
(() => {
    const normalizeScrollOptions = (arg) => {
        if (!arg || typeof arg !== "object") {
            return arg;
        }

        if (arg.behavior !== "smooth") {
            return arg;
        }

        return { ...arg, behavior: "auto" };
    };

    const patchScrollMethod = (prototype, methodName) => {
        const original = prototype?.[methodName];
        if (typeof original !== "function") {
            return;
        }

        prototype[methodName] = function (...args) {
            const patchedArgs = [...args];

            if (patchedArgs.length > 0) {
                patchedArgs[0] = normalizeScrollOptions(patchedArgs[0]);
            }

            return original.apply(this, patchedArgs);
        };
    };

    const style = document.createElement("style");
    style.id = "memos-disable-motion";
    style.textContent = `
    *, *::before, *::after {
      animation: none !important;
      animation-delay: 0s !important;
      animation-duration: 0s !important;
      animation-iteration-count: 1 !important;
      transition: none !important;
      transition-delay: 0s !important;
      transition-duration: 0s !important;
      scroll-behavior: auto !important;
    }

    html {
      view-transition-name: none !important;
    }

    ::view-transition-group(*),
    ::view-transition-old(*),
    ::view-transition-new(*) {
      animation: none !important;
      transition: none !important;
    }

    video[src*="motion=true"] {
      opacity: 0 !important;
      transition: none !important;
    }
  `;
    document.documentElement.appendChild(style);

    const noViewTransition = (callback) => {
        callback();
        return {
            ready: Promise.resolve(),
            updateCallbackDone: Promise.resolve(),
            finished: Promise.resolve(),
            skipTransition() {
                return undefined;
            }
        };
    };

    try {
        Object.defineProperty(document, "startViewTransition", {
            configurable: true,
            value: noViewTransition
        });
    } catch {
        document.startViewTransition = noViewTransition;
    }

    patchScrollMethod(window, "scrollTo");
    patchScrollMethod(window, "scrollBy");
    patchScrollMethod(Element.prototype, "scrollTo");
    patchScrollMethod(Element.prototype, "scrollBy");
    patchScrollMethod(Element.prototype, "scrollIntoView");
})();
