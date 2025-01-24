<script lang="ts">
import { cn } from "$lib/utils.js";
import type { Snippet } from "svelte";
import type { HTMLAttributes } from "svelte/elements";

export interface Props extends HTMLAttributes<HTMLDivElement> {
    name?: string;
    desc?: string;
    long?: string;
    children?: Snippet;
    disabled?: boolean;
    bg?: string;
}

let {
    name,
    desc,
    children,
    disabled,
    bg = "bg-card",
    class: className,
    ...restProps
}: Props = $props();

const commonProps: Record<string, any> = $derived(restProps);
</script>

<div
  {...commonProps}
  class={cn(
    "w-full grid sm:grid-flow-row-dense md:grid-flow-col-dense md:grid-cols-2 rounded-xl p-4 border border-opacity-0 hover:border-opacity-100",
    className,
    bg,
    disabled && "opacity-50 cursor-not-allowed",
  )}
>
  {#if name || desc}
    <div class="w-full h-full break-words self-center">
      {#if name}
        <h1 class="font-semibold tracking-tight text-md leading-none text-foreground">
          {@html name}
        </h1>
      {/if}
      {#if desc}
        <h2 class="text-muted-foreground text-sm mt-2 text-justify break-words mr-10">
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
