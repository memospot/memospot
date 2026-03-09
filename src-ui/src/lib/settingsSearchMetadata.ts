export function normalizeSettingSearchLabel(value?: string): string {
    return (value ?? "")
        .replace(/<[^>]*>/g, "")
        .replace(/\s+/g, " ")
        .trim();
}

export function normalizeSettingSearchKeywords(value?: string[] | string): string {
    if (Array.isArray(value)) {
        return value.join("|");
    }

    return (value ?? "").trim().replace(/\s*\|\s*/g, "|");
}
