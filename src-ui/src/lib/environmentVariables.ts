/**
 * Parse plain-text environment variables (key=value) into key-value pairs.
 *
 * - Returns a sorted object.
 * - Allows empty values and leading/ending whitespaces on values, but not on keys.
 * @param vars
 */
export function envToKV(vars: string) {
    return vars
        .split(/\r?\n|\r|\n/g)
        .filter((s) => s.length > 0)
        .map((s) => [s.substring(0, s.indexOf("=")), s.substring(s.indexOf("=") + 1)])
        .filter((a: Array<string>) => a.length === 2)
        .filter(([key, _]) => key.trim().length > 0)
        .sort()
        .reduce(
            (acc, [key, value]) => {
                acc[key.trim()] = value;
                return acc;
            },
            {} as Record<string, string>
        );
}

/**
 * Parse key-value pairs into plain-text environment variables (key=value).
 *
 * - Returns sorted lines.
 * - Allows empty values and leading/ending whitespaces on values, but not on keys.
 * @param vars
 */
export function envFromKV(vars: Record<string, string>) {
    return Object.entries(vars)
        .map(([key, value]) => `${key.trim()}=${value}`)
        .sort()
        .join("\n");
}
