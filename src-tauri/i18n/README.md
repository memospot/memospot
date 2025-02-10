# Adding a new locale

> [!WARNING]
> The new file name/locale tag must be a valid browser locale.
> See [`navigator.language`](https://www.localeplanet.com/support/browser.html)
>
> For simplified/traditional Chinese, use zh-Hans/zh-Hant as the locale.

1. Add a new translation file for the new locale at `i18n/{locale}/memospot.ftl`

2. Submit a pull request.

> [!NOTE]
> Edge cases should be handled in `src-tauri/src/i18n.rs`.

> [!NOTE]
> Locales added here are only for the desktop side.
> For the web front-end, see `src-ui/i18n/`.

Desktop localization is handled with [Fluent](https://projectfluent.org).

[Fluent syntax guide](https://projectfluent.org/fluent/guide/)
