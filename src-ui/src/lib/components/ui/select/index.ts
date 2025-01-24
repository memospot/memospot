import { Select as SelectPrimitive } from "bits-ui";
import type { Selected } from "bits-ui";

import Content from "./select-content.svelte";
import Item from "./select-item.svelte";
import Label from "./select-label.svelte";
import Separator from "./select-separator.svelte";
import Trigger from "./select-trigger.svelte";

const Root = SelectPrimitive.Root;
const Group = SelectPrimitive.Group;
const Input = SelectPrimitive.Input;
const Value = SelectPrimitive.Value;

export {
    Content as SelectContent,
    Group as SelectGroup,
    Input as SelectInput,
    Item as SelectItem,
    Label as SelectLabel,
    Root as Select,
    type Selected,
    Separator as SelectSeparator,
    Trigger as SelectTrigger,
    Value as SelectValue
};
