# Adding a new language

> [!WARNING]
> The new file name/language tag must be a valid browser locale.
> See [`navigator.language`](https://www.localeplanet.com/support/browser.html)

> [!IMPORTANT]
> For now, it's best to use specific locale keys rather than generic ones.
> Language fallbacks and overrides need work on the Rust side, as they don't work on all platforms.

1. Add the new language to the `project.inlang/settings.json` file.

2. Add a new translation file for the new language at `i18n/{locale}.json`

3. Submit a pull request.

> [!NOTE]
> Edge cases should be handled in `src-ui/lib/i18n.ts`.

> [!NOTE]
> Languages added here are only for the web front-end.
> For the desktop side, see `src-tauri/i18n/`.

Front-end localization is handled with [Paraglide-SvelteKit](https://inlang.com/m/dxnzrydw/paraglide-sveltekit-i18n/getting-started).
