<script lang="ts">
import type { Snippet } from "svelte";
import type { HTMLAttributes } from "svelte/elements";
import { Switch } from "$lib/components/ui/switch/index";
import { cn } from "$lib/utils.js";

export interface Props extends HTMLAttributes<HTMLDivElement> {
    name?: string;
    desc?: string;
    state?: boolean;
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
    state = $bindable(undefined),
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
    "w-full h-full flex flex-col rounded-xl p-4 space-y-3 border border-opacity-0 hover:border-opacity-100",
    className,
    bg,
  )}
>
  <div class="w-full flex flex-row">
    {#if name || desc}
      <div class="w-full h-full wrap-break-word self-center mr-4">
        {#if name}
          <h1 class="font-semibold tracking-tight text-base leading-tight text-foreground">
            {@html name}
          </h1>
        {/if}
        {#if desc}
          <h2 class="text-muted-foreground text-sm mt-2 text-justify">
            {@html desc}
          </h2>
        {/if}
      </div>
    {/if}

    <div class="self-center justify-end">
      <Switch bind:checked={state} />
    </div>
  </div>
  {#if children}
    <div class={cn("w-full space-y-2", !state && "opacity-50 cursor-not-allowed")}>
      {@render children()}
    </div>
  {/if}
</div>
