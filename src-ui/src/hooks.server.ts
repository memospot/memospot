import type { Handle } from "@sveltejs/kit/hooks";
import { getTextDirection } from "#lib/i18n";
import { paraglideMiddleware } from "#lib/paraglide/server";

// creating a handle to use the paraglide middleware
const paraglideHandle: Handle = ({ event, resolve }) =>
    paraglideMiddleware(event.request, ({ request: localizedRequest, locale }) => {
        return resolve(
            { ...event, request: localizedRequest },
            {
                transformPageChunk: ({ html }) => {
                    return html
                        .replace("%lang%", locale)
                        .replace("%textDirection%", getTextDirection(locale));
                }
            }
        );
    });

export const handle: Handle = paraglideHandle;
