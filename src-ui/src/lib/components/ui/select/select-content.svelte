<script lang="ts">
import { Select as SelectPrimitive } from "bits-ui";
import type { ClassValue } from "svelte/elements";
import { scale } from "svelte/transition";
import { flyAndScale } from "$lib/utils.js";

interface Props extends SelectPrimitive.ContentProps {
    class?: ClassValue;
    reduceAnimation?: boolean;
    [key: string]: any;
}

let {
    class: className = undefined,
    sideOffset = 4,
    sameWidth = false,
    collisionPadding = 8,
    avoidCollisions = true,
    fitViewport = true,
    strategy = "absolute",
    reduceAnimation = false,
    inTransition = flyAndScale,
    inTransitionConfig = undefined,
    outTransition = scale,
    outTransitionConfig = {
        start: 0.95,
        opacity: 0,
        duration: 50
    },
    children,
    ...rest
}: Props = $props();

const resolvedInTransition = $derived(reduceAnimation ? undefined : inTransition);
const resolvedInTransitionConfig = $derived(reduceAnimation ? undefined : inTransitionConfig);
const resolvedOutTransition = $derived(reduceAnimation ? undefined : outTransition);
const resolvedOutTransitionConfig = $derived(reduceAnimation ? undefined : outTransitionConfig);
</script>

<SelectPrimitive.Content
  inTransition={resolvedInTransition}
  inTransitionConfig={resolvedInTransitionConfig}
  outTransition={resolvedOutTransition}
  outTransitionConfig={resolvedOutTransitionConfig}
  {sideOffset}
  {sameWidth}
  {collisionPadding}
  {avoidCollisions}
  {fitViewport}
  {strategy}
  class={[
    "select-content-scroll bg-popover text-popover-foreground relative isolate z-1000 min-w-32 w-max max-w-[calc(100vw-1rem)] max-h-[80svh] overflow-x-auto overflow-y-auto overscroll-contain rounded-md border shadow-md focus:outline-none",
    className
  ]}
  {...rest}
>
  <div class="w-full p-1">
    {@render children?.()}
  </div>
</SelectPrimitive.Content>

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
