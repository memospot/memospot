<script lang="ts">
import { cn } from "$lib/utils.js";
import { Select as SelectPrimitive } from "bits-ui";
import CaretSort from "svelte-radix/CaretSort.svelte";

interface Props extends SelectPrimitive.TriggerProps {
    [key: string]: any;
}

let { class: className = undefined, children, ...rest }: Props = $props();
</script>

<SelectPrimitive.Trigger
  class={cn(
    "ring-offset-background placeholder:text-muted-foreground focus-visible:ring-ring aria-[invalid]:border-destructive data-[placeholder]:[&>span]:text-muted-foreground flex h-9 w-full items-center justify-between whitespace-nowrap rounded-md border bg-transparent px-3 py-2 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1 hover:translate-y-[-2px] hover:drop-shadow",
    className,
  )}
  {...rest}
>
  {@render children?.()}
  <div>
    <CaretSort class="h-4 w-4 opacity-50" />
  </div>
</SelectPrimitive.Trigger>
