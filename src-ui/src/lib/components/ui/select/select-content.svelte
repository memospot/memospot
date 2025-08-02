<script lang="ts">
import { Select as SelectPrimitive } from "bits-ui";
import { scale } from "svelte/transition";
import { cn, flyAndScale } from "$lib/utils.js";

interface Props extends SelectPrimitive.ContentProps {
    [key: string]: any;
}

let {
    class: className = undefined,
    sideOffset = 4,
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
</script>

<SelectPrimitive.Content
  {inTransition}
  {inTransitionConfig}
  {outTransition}
  {outTransitionConfig}
  {sideOffset}
  class={cn(
    "bg-popover text-popover-foreground relative z-50 min-w-[8rem] overflow-hidden rounded-md border shadow-md focus:outline-none",
    className,
  )}
  {...rest}
>
  <div class="w-full p-1">
    {@render children?.()}
  </div>
</SelectPrimitive.Content>
