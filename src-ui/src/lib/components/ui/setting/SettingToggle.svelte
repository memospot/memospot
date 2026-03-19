<script lang="ts">
import type { Snippet } from "svelte";
import type { HTMLAttributes } from "svelte/elements";
import { Switch } from "$lib/components/ui/switch/index";
import {
    normalizeSettingSearchKeywords,
    normalizeSettingSearchLabel
} from "$lib/settingsSearchMetadata";

export interface Props extends HTMLAttributes<HTMLDivElement> {
    name?: string;
    desc?: string;
    state?: boolean;
    children?: Snippet;
    disabled?: boolean;
    bg?: string;
    searchId?: string;
    searchLabel?: string;
    searchKeywords?: string[] | string;
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
    searchKeywords,
    class: className,
    ...restProps
}: Props = $props();

const commonProps: Record<string, any> = $derived(restProps);
const normalizedSearchLabel = $derived(normalizeSettingSearchLabel(searchLabel ?? name));
const normalizedSearchKeywords = $derived(normalizeSettingSearchKeywords(searchKeywords));
</script>

<div
  {...commonProps}
  data-setting-row="true"
  data-setting-id={searchId}
  data-setting-label={normalizedSearchLabel}
  data-setting-keywords={normalizedSearchKeywords}
  class={[
    "w-full h-full flex flex-col rounded-xl p-4 space-y-3 border border-opacity-0 hover:border-opacity-100",
    className,
    bg
  ]}
>
  <div class="w-full flex flex-row">
    {#if name || desc}
      <div class="w-full h-full wrap-break-word self-center mr-4">
        {#if name}
          <h2 class="font-semibold uppercase tracking-[0.09rem] text-xs text-foreground mb-1">
            {@html name}
          </h2>
        {/if}
        {#if desc}
          <span class="text-muted-foreground/90 text-sm text-justify wrap-break-word mr-10">
            {@html desc}
          </span>
        {/if}
      </div>
    {/if}

    <div class="self-center justify-end">
      <Switch bind:checked={state} />
    </div>
  </div>
  {#if children}
    <div class={["w-full space-y-2", !state && "opacity-50 cursor-not-allowed"]}>
      {@render children()}
    </div>
  {/if}
</div>
