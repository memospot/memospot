import { sequence } from "@sveltejs/kit/hooks";
import { i18n } from "$lib/i18n";

export const handle = sequence(i18n.handle());
