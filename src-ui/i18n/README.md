# Adding a new language

> [!WARNING]
> The new file name/language tag must be a valid browser locale.
> See [`navigator.language`](https://www.localeplanet.com/support/browser.html)

> [!IMPORTANT]
> For now, it's best to use specific locale keys rather than generic ones.
> Language fallbacks and overrides need work on the Rust side, as they don't work on all platforms.

1. Add the new language to the `i18n/settings.json` file, under `locales`.

2. Add a new translation file for the new language at `i18n/{locale}.json`

3. Submit a pull request.

> [!NOTE]
> Edge cases should be handled in `src-ui/lib/i18n.ts`.

> [!NOTE]
> Languages added here are only for the web front-end.
> For the native desktop translations (error messages and native window menus), see `crates/memospot/i18n/`.

Front-end localization is handled with [@inlang/paraglide-js](https://inlang.com/m/gerre34r/library-inlang-paraglideJs/sveltekit).
