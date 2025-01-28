<script lang="ts">
import { cn } from "$lib/utils.js";
import { Switch as SwitchPrimitive } from "bits-ui";

interface Props extends SwitchPrimitive.Props {
    class?: string;
    checked?: boolean;
}

let { class: className = undefined, checked = $bindable(false), ...rest }: Props = $props();
</script>

<SwitchPrimitive.Root
  bind:checked
  on:click={() => {
    checked = !checked;
  }}
  class={cn(
    "focus-visible:ring-ring focus-visible:ring-offset-background data-[state=checked]:bg-primary data-[state=unchecked]:bg-input peer inline-flex h-[20px] w-[36px] shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 hover:translate-y-[-2px] hover:drop-shadow",
    className,
  )}
  {...rest}
>
  <SwitchPrimitive.Thumb
    class={cn(
      "bg-card pointer-events-none block h-4 w-4 rounded-full shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-4 data-[state=unchecked]:translate-x-0",
    )}
  />
</SwitchPrimitive.Root>
