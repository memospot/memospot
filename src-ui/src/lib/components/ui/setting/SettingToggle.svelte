<script lang="ts">
import { Switch } from "$lib/components/ui/switch/index";
import { cn } from "$lib/utils.js";
import type { Snippet } from "svelte";
import type { HTMLAttributes } from "svelte/elements";

export interface Props extends HTMLAttributes<HTMLDivElement> {
	name?: string;
	desc?: string;
	state?: boolean;
	children?: Snippet;
	disabled?: boolean;
	bg?: string;
}

let {
	name,
	desc,
	state = $bindable(undefined),
	children,
	disabled,
	bg = "bg-card",
	class: className,
	...restProps
}: Props = $props();

const commonProps: Record<string, any> = $derived(restProps);
</script>

<div
  class={cn(
    "w-full h-full flex flex-col rounded-xl p-5 space-y-4 border border-opacity-0 hover:border-opacity-100",
    className,
    bg,
  )}
  {...commonProps}
>
  <div class="w-full flex flex-row">
    {#if name || desc}
      <div class="w-full h-full break-words self-center mr-4">
        {#if name}
          <h1
            class="font-semibold tracking-tight text-md leading-none text-foreground"
          >
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
      <Switch bind:state />
    </div>
  </div>
  {#if children}
    <div
      class={cn("w-full space-y-3", !state && "opacity-50 cursor-not-allowed")}
    >
      {@render children()}
    </div>
  {/if}
</div>
