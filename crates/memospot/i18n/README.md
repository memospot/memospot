# Adding a new locale

## Via localization platform

You can join the
[Crowdin](https://crowdin.com/project/memospot) platform to help make this project accessible to a broader audience!
[![Crowdin](https://badges.crowdin.net/memospot/localized.svg)](https://crowdin.com/project/memospot)

## Manually

> [!WARNING]
> The new file name/locale tag must be a valid browser locale.
> See [`navigator.language`](https://www.localeplanet.com/support/browser.html)

> [!IMPORTANT]
> For now, it's best to use specific locales rather than generic ones.
> Language fallbacks and overrides need work, as they don't work on all platforms.

1. Add a new translation file for the new locale at `i18n/{locale}/memospot.ftl`

2. Submit a pull request.

> [!NOTE]
> Edge cases should be handled in `crates/memospot/src/i18n.rs`.

> [!NOTE]
> Locales added here are only for the desktop side.
> For the web front-end, see `src-ui/i18n/`.

## Fluent format

Desktop localization is handled with [Fluent](https://projectfluent.org).

[Fluent syntax guide](https://projectfluent.org/fluent/guide/)

### Mnemonics

For menu items, keyboard hot-key mnemonics are set with `&` at any position of a word.

Make sure they don't conflict with other mnemonics in the same context.

You can freely move the `&` around to different positions in the same entry if there are conflicts in the same context, but if there are no good options, you may need to change the wording or even remove the mnemonic.
