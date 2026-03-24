import type { SettingSearchEntry } from "$lib/settingsSearch";

export type SearchNavigationHighlightState = {
    highlightedElement: HTMLElement | null;
    highlightTimer: ReturnType<typeof setTimeout> | undefined;
};

type NavigateToSearchResultOptions = {
    entry: SettingSearchEntry;
    contentPane: HTMLElement | undefined;
    reduceAnimation: boolean;
    updateSection: (sectionId: string) => Promise<void>;
    tick: () => Promise<void>;
    highlightState: SearchNavigationHighlightState;
    onHighlightCleared?: () => void;
    highlightClassName?: string;
    highlightDurationMs?: number;
};

const PREFERRED_FOCUS_SELECTORS = [
    "input[type='text']:not([disabled]):not([readonly])",
    "input[type='search']:not([disabled]):not([readonly])",
    "input[type='url']:not([disabled]):not([readonly])",
    "input[type='email']:not([disabled]):not([readonly])",
    "input[type='tel']:not([disabled]):not([readonly])",
    "input[type='password']:not([disabled]):not([readonly])",
    "input:not([type]):not([disabled]):not([readonly])",
    "textarea:not([disabled]):not([readonly])",
    "select:not([disabled])",
    "[contenteditable='true']",
    "button:not([disabled])",
    "input:not([type='hidden']):not([disabled])",
    "[tabindex]:not([tabindex='-1'])"
];

const TEXTUAL_INPUT_TYPES = new Set(["text", "search", "url", "email", "tel", "password"]);

function findSettingRowById(
    contentPane: HTMLElement | undefined,
    settingId: string
): HTMLElement | null {
    if (!contentPane) return null;
    const rows = contentPane.querySelectorAll<HTMLElement>("[data-setting-row='true']");
    for (const row of rows) {
        if ((row.dataset.settingId ?? "") === settingId) {
            return row;
        }
    }
    return null;
}

function getPreferredFocusTarget(container: HTMLElement): HTMLElement | null {
    for (const selector of PREFERRED_FOCUS_SELECTORS) {
        const candidate = container.querySelector<HTMLElement>(selector);
        if (candidate) return candidate;
    }
    return null;
}

function scrollTargetIntoPaneCenter(
    contentPane: HTMLElement | undefined,
    target: HTMLElement,
    reduceAnimation: boolean
) {
    if (contentPane) {
        const paneRect = contentPane.getBoundingClientRect();
        const targetRect = target.getBoundingClientRect();
        const offsetWithinPane = targetRect.top - paneRect.top;
        const centeredTop =
            contentPane.scrollTop +
            offsetWithinPane -
            (contentPane.clientHeight / 2 - targetRect.height / 2);

        contentPane.scrollTo({
            top: Math.max(0, centeredTop),
            behavior: reduceAnimation ? "auto" : "smooth"
        });
        return;
    }

    target.scrollIntoView({
        block: "center",
        behavior: reduceAnimation ? "auto" : "smooth"
    });
}

function focusTarget(target: HTMLElement) {
    const needsTemporaryTabIndex = !target.hasAttribute("tabindex") && target.tabIndex < 0;
    if (needsTemporaryTabIndex) {
        target.setAttribute("tabindex", "-1");
    }

    target.focus({ preventScroll: true });

    if (needsTemporaryTabIndex) {
        queueMicrotask(() => {
            target.removeAttribute("tabindex");
        });
    }
}

function focusPreferredTarget(settingRow: HTMLElement) {
    const preferredTarget = getPreferredFocusTarget(settingRow);
    const targetToFocus = preferredTarget ?? settingRow;
    focusTarget(targetToFocus);

    if (preferredTarget instanceof HTMLInputElement) {
        if (TEXTUAL_INPUT_TYPES.has(preferredTarget.type)) {
            preferredTarget.select();
        }
        return;
    }

    if (preferredTarget instanceof HTMLTextAreaElement) {
        preferredTarget.select();
    }
}

function applyTargetHighlight(
    target: HTMLElement,
    highlightState: SearchNavigationHighlightState,
    onHighlightCleared?: () => void,
    highlightClassName = "settings-search-highlight",
    highlightDurationMs = 1400
): SearchNavigationHighlightState {
    if (highlightState.highlightedElement) {
        highlightState.highlightedElement.classList.remove(highlightClassName);
    }
    if (highlightState.highlightTimer) {
        clearTimeout(highlightState.highlightTimer);
    }

    target.classList.add(highlightClassName);
    const highlightTimer = setTimeout(() => {
        target.classList.remove(highlightClassName);
        onHighlightCleared?.();
    }, highlightDurationMs);

    return {
        highlightedElement: target,
        highlightTimer
    };
}

export async function navigateToSearchResult(
    options: NavigateToSearchResultOptions
): Promise<SearchNavigationHighlightState> {
    const {
        entry,
        contentPane,
        reduceAnimation,
        updateSection,
        tick,
        highlightState,
        onHighlightCleared,
        highlightClassName,
        highlightDurationMs
    } = options;

    await updateSection(entry.sectionId);
    await tick();

    let target = findSettingRowById(contentPane, entry.id);
    if (!target) {
        for (let attempt = 0; attempt < 6 && !target; attempt += 1) {
            await new Promise((resolve) => setTimeout(resolve, 50));
            await tick();
            target = findSettingRowById(contentPane, entry.id);
        }
    }
    if (!target) {
        return highlightState;
    }

    scrollTargetIntoPaneCenter(contentPane, target, reduceAnimation);
    focusPreferredTarget(target);

    return applyTargetHighlight(
        target,
        highlightState,
        onHighlightCleared,
        highlightClassName,
        highlightDurationMs
    );
}
