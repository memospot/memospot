<script lang="ts">
import { Select as SelectPrimitive } from "bits-ui";
import { cn } from "$lib/utils.js";

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
