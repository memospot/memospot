# Adding a new language

> [!WARNING]
> The new file name/language tag must be a valid browser language tag.
> See [`navigator.language`](https://www.localeplanet.com/support/browser.html)
>
> For simplified/traditional Chinese, use zh-Hans/zh-Hant as the language tag.

1. Add the new language to the `project.inlang/settings.json` file.

2. Add a new translation file for the new language at `i18n/{languageTag}.json`

3. Submit a pull request.

> [!NOTE]
> Edge cases are handled in `src/lib/i18n.ts`.

> [!NOTE]
> Languages added here are only for the web front-end.
