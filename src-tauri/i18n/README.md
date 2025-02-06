# Adding a new language

> [!WARNING]
> The new file name/language tag must be a valid browser locale.
> See [`navigator.language`](https://www.localeplanet.com/support/browser.html)
>
> For simplified/traditional Chinese, use zh-Hans/zh-Hant as the locale.

1. Add a new translation file for the new language at `i18n/{locale}/memospot.ftl`

2. Submit a pull request.

> [!NOTE]
> Edge cases should be handled in `src-tauri/src/localize.rs`.

> [!NOTE]
> Languages added here are only for the desktop side.
> For the web front-end, see `src-ui/i18n/`.

Desktop localization is handled with [Fluent](https://projectfluent.org).

[Fluent syntax guide](https://projectfluent.org/fluent/guide/)
