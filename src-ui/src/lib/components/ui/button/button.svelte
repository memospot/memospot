<script lang="ts">
import type { Snippet } from "svelte";
import type { ClassValue, HTMLButtonAttributes } from "svelte/elements";

export type ButtonVariant = "secondary" | "primary" | "warning";

interface Props extends Omit<HTMLButtonAttributes, "class"> {
    class?: ClassValue;
    variant?: ButtonVariant;
    children?: Snippet;
}

let {
    type = "button",
    variant = "secondary",
    class: className = undefined,
    children,
    ...rest
}: Props = $props();

const BUTTON_VARIANT_CLASSES: Record<ButtonVariant, string> = {
    secondary:
        "border bg-secondary active:translate-y-px hover:drop-shadow hover:opacity-80 disabled:opacity-40",
    primary: "bg-primary text-zinc-50 hover:opacity-80 disabled:opacity-50",
    warning:
        "border-amber-600/80 bg-amber-200/70 text-amber-900 dark:border-amber-500/70 dark:bg-amber-500/15 dark:text-amber-300 active:translate-y-px hover:drop-shadow"
};
</script>

<button
  {type}
  class={["border-box inline-flex h-9 cursor-pointer items-center justify-center rounded-md px-3 py-1.5 font-semibold uppercase tracking-[0.05rem] text-xs disabled:cursor-not-allowed disabled:opacity-50", BUTTON_VARIANT_CLASSES[variant], className]}
  {...rest}
>
  {@render children?.()}
</button>
