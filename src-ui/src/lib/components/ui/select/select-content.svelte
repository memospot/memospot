<script lang="ts">
import { Select as SelectPrimitive } from "bits-ui";
import type { ClassValue } from "svelte/elements";
import { scale } from "svelte/transition";
import { flyAndScale } from "#lib/utils.js";

interface Props extends SelectPrimitive.ContentProps {
    class?: ClassValue;
    reduceAnimation?: boolean;
}

let {
    class: className = undefined,
    sideOffset = 4,
    collisionPadding = 8,
    avoidCollisions = true,
    strategy = "absolute",
    reduceAnimation = false,
    children,
    ...rest
}: Props = $props();

const contentClass = $derived([
    "select-content-scroll bg-popover text-popover-foreground relative isolate z-1000 min-w-32 w-max max-w-[calc(100vw-1rem)] max-h-[80svh] overflow-x-auto overflow-y-auto overscroll-contain rounded-md border shadow-md focus:outline-none",
    className
]);
</script>

{#if reduceAnimation}
  <SelectPrimitive.Content
    {sideOffset}
    {collisionPadding}
    {avoidCollisions}
    {strategy}
    class={contentClass}
    {...rest}
  >
    <div class="w-full p-1">
      {@render children?.()}
    </div>
  </SelectPrimitive.Content>
{:else}
  <SelectPrimitive.Content
    {sideOffset}
    {collisionPadding}
    {avoidCollisions}
    {strategy}
    class={contentClass}
    forceMount
    {...rest}
  >
    {#snippet child({ wrapperProps, props, open })}
      {#if open}
        <div {...wrapperProps}>
          <div {...props} transition:flyAndScale>
            <div class="w-full p-1">
              {@render children?.()}
            </div>
          </div>
        </div>
      {/if}
    {/snippet}
  </SelectPrimitive.Content>
{/if}

<style>
:global(.select-content-scroll) {
    background-color: hsl(var(--popover));
    scrollbar-color: hsl(var(--border)) hsl(var(--popover));
    scrollbar-width: thin;
}

:global(.select-content-scroll::-webkit-scrollbar) {
    width: 0.75rem;
    height: 0.75rem;
}

:global(.select-content-scroll::-webkit-scrollbar-track) {
    background-color: hsl(var(--popover));
}

:global(.select-content-scroll::-webkit-scrollbar-thumb) {
    border: 3px solid hsl(var(--popover));
    border-radius: 9999px;
    background-color: hsl(var(--border));
}
</style>
