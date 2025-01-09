type Timer = ReturnType<typeof setTimeout>;

/**
 * Debounce an async function, ensuring only the last call's result is returned.
 * @param func The async function to debounce
 * @param intervalMs Delay in milliseconds. Defaults to 300ms.
 * @returns A debounced version of the async function
 */
export function debouncePromise(
    func: (...args: any[]) => Promise<any>,
    intervalMs = 300
): () => ReturnType<typeof func> {
    let handle: Timer;
    let resolves: Array<(value?: unknown) => void> = [];

    return async (...args: unknown[]) => {
        clearTimeout(handle);
        handle = setTimeout(() => {
            const result = func(...args);
            for (const resolve of resolves) {
                resolve(result);
            }
            resolves = [];
        }, intervalMs);

        return new Promise((resolve) => resolves.push(resolve));
    };
}

/**
 * Debounce a function, ensuring only the last call's result is returned.
 * @param func The function to debounce
 * @param wait Delay in milliseconds. Defaults to 300ms.
 * @returns A debounced version of the function
 */
export function debounce<F extends (...args: any[]) => any>(
    func: F,
    wait = 300
): (...args: Parameters<F>) => void {
    let timeoutId: Timer | null = null;

    return (...args: Parameters<F>) => {
        if (timeoutId) {
            clearTimeout(timeoutId);
        }

        timeoutId = setTimeout(() => {
            func(...args);
        }, wait);
    };
}
