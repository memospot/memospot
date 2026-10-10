import { Select as SelectPrimitive } from "bits-ui";

import Content from "./select-content.svelte";
import Item from "./select-item.svelte";
import Trigger from "./select-trigger.svelte";

const Root = SelectPrimitive.Root;
const Group = SelectPrimitive.Group;
const Value = SelectPrimitive.Value;

export {
    Content as SelectContent,
    Group as SelectGroup,
    Item as SelectItem,
    Root as Select,
    Trigger as SelectTrigger,
    Value as SelectValue
};
