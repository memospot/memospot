<script lang="ts">
import { cn } from "$lib/utils.js";
import { Select as SelectPrimitive } from "bits-ui";

interface Props extends SelectPrimitive.LabelProps {
    [key: string]: any;
}

let { class: className = undefined, children, ...rest }: Props = $props();
</script>

<SelectPrimitive.Label
  class={cn("px-2 py-1.5 text-sm font-semibold", className)}
  {...rest}
>
  {@render children?.()}
</SelectPrimitive.Label>
