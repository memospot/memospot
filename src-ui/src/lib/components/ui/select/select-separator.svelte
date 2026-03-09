<script lang="ts">
import { Select as SelectPrimitive } from "bits-ui";
import type { ClassValue } from "svelte/elements";

interface Props extends SelectPrimitive.SeparatorProps {
    class?: ClassValue;
    [key: string]: any;
}

let { class: className = undefined, ...rest }: Props = $props();
</script>

<SelectPrimitive.Separator
  class={["bg-muted -mx-1 my-1 h-px", className]}
  {...rest}
/>
