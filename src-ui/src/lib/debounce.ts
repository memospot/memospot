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
