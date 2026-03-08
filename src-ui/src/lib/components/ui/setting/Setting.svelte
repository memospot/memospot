<script lang="ts">
import type { Snippet } from "svelte";
import type { HTMLAttributes } from "svelte/elements";
import { cn } from "$lib/utils.js";

export interface Props extends HTMLAttributes<HTMLDivElement> {
    name?: string;
    desc?: string;
    long?: string;
    children?: Snippet;
    disabled?: boolean;
    bg?: string;
    searchId?: string;
    searchLabel?: string;
    searchAliases?: string[] | string;
}

let {
    name,
    desc,
    children,
    disabled,
    bg = "bg-card",
    searchId,
    searchLabel,
    searchAliases,
    class: className,
    ...restProps
}: Props = $props();

const commonProps: Record<string, any> = $derived(restProps);
const normalizedSearchLabel = $derived(
    (searchLabel ?? name ?? "")
        .replace(/<[^>]*>/g, "")
        .replace(/\s+/g, " ")
        .trim()
);
const normalizedSearchAliases = $derived(
    Array.isArray(searchAliases)
        ? searchAliases.join("|")
        : (searchAliases ?? "").trim().replace(/\s*\|\s*/g, "|")
);
</script>

<div
  {...commonProps}
  data-setting-row="true"
  data-setting-id={searchId}
  data-setting-label={normalizedSearchLabel}
  data-setting-aliases={normalizedSearchAliases}
  class={cn(
    "w-full grid sm:grid-flow-row-dense md:grid-flow-col-dense md:grid-cols-2 rounded-xl p-3 border border-opacity-0 hover:border-opacity-100",
    className,
    bg,
    disabled && "opacity-50 cursor-not-allowed",
  )}
>
  {#if name || desc}
    <div class="w-full h-full wrap-break-word self-center">
      {#if name}
        <h1 class="font-semibold tracking-tight text-base leading-tight text-foreground">
          {@html name}
        </h1>
      {/if}
      {#if desc}
        <h2 class="text-muted-foreground text-sm mt-2 text-justify wrap-break-word mr-10">
          {@html desc}
        </h2>
      {/if}
    </div>
  {/if}
  {#if children}
    <div class="w-max h-fit self-center md:justify-self-end mt-3 md:mt-0">
      {@render children()}
    </div>
  {/if}
</div>
