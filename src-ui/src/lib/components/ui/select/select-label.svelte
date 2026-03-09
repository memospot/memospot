<script lang="ts">
import { Select as SelectPrimitive } from "bits-ui";
import type { ClassValue } from "svelte/elements";

interface Props extends SelectPrimitive.LabelProps {
    class?: ClassValue;
    [key: string]: any;
}

let { class: className = undefined, children, ...rest }: Props = $props();
</script>

<SelectPrimitive.Label
  class={["px-2 py-1 text-sm font-semibold", className]}
  {...rest}
>
  {@render children?.()}
</SelectPrimitive.Label>
