<script lang="ts">
import { cn } from "$lib/utils.js";
import { Select as SelectPrimitive } from "bits-ui";

interface Props extends SelectPrimitive.SeparatorProps {
	[key: string]: any;
}

let { class: className = undefined, ...rest }: Props = $props();
</script>

<SelectPrimitive.Separator
  class={cn("bg-muted -mx-1 my-1 h-px", className)}
  {...rest}
/>
