<script lang="ts">
import { Select as SelectPrimitive } from "bits-ui";
import Check from "svelte-radix/Check.svelte";
import { cn } from "$lib/utils.js";

interface Props extends SelectPrimitive.ItemProps {
    [key: string]: any;
}

let {
    class: className = undefined,
    value,
    label = undefined,
    disabled = undefined,
    children,
    ...rest
}: Props = $props();
</script>

<SelectPrimitive.Item
  {value}
  {disabled}
  {label}
  class={cn(
    "data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-2 pr-8 text-sm outline-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
    className,
  )}
  {...rest}
  on:click
  on:pointermove
  on:focusin
>
  <span class="absolute right-2 flex h-3.5 w-3.5 items-center justify-center">
    <SelectPrimitive.ItemIndicator>
      <Check class="h-4 w-4" />
    </SelectPrimitive.ItemIndicator>
  </span>
  {#if children}{@render children()}{:else}
    {label || value}
  {/if}
</SelectPrimitive.Item>
