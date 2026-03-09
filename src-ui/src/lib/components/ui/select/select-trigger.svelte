<script lang="ts">
import { Select as SelectPrimitive } from "bits-ui";
import type { ClassValue } from "svelte/elements";
import CaretSort from "svelte-radix/CaretSort.svelte";

interface Props extends SelectPrimitive.TriggerProps {
    class?: ClassValue;
    [key: string]: any;
}

let { class: className = undefined, children, ...rest }: Props = $props();
</script>

<SelectPrimitive.Trigger
  class={[
    "ring-offset-background placeholder:text-muted-foreground focus-visible:ring-ring aria-[invalid]:border-destructive data-placeholder:[&>span]:text-muted-foreground flex h-9 w-full items-center justify-between whitespace-nowrap rounded-md border bg-transparent px-3 py-1.5 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1 hover:-translate-y-0.5 hover:drop-shadow",
    className
  ]}
  {...rest}
>
  {@render children?.()}
  <div>
    <CaretSort class="h-4 w-4 opacity-50" />
  </div>
</SelectPrimitive.Trigger>
