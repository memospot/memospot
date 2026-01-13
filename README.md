# [Memospot](https://memospot.github.io)

<img height="128" src="assets/app-icon-lossless.webp" alt="app-icon" align="right" />

Self-contained desktop version of [Memos](https://github.com/usememos/memos) -a privacy-first, lightweight note-taking
service. Available for Windows, macOS, and Linux.

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/memospot/memospot) [![Downloads](https://img.shields.io/github/downloads/memospot/memospot/total?logo=github)](https://github.com/memospot/memospot/releases) [![Crowdin](https://badges.crowdin.net/memospot/localized.svg)](https://crowdin.com/project/memospot) [![GitHub Stars](https://img.shields.io/github/stars/memospot/memospot?logo=github)](https://github.com/memospot/memospot)

## Key features

- Full-featured [Memos experience](https://www.usememos.com/docs):
  The same features from a hosted Memos instance are available offline in Memospot.
- Client mode:
  Optionally, connect to a remote Memos server while using Memospot as a desktop client.
- Data portability:
  Data can be seamlessly moved between hosted Memos and local Memospot instances.
  Start taking notes on your desktop and later migrate them to a hosted Memos server if desired.
- Enhanced privacy:
  All data is stored locally on your device, ensuring maximum privacy and control.
  Neither Memospot nor Memos tracks any user data.
- Lightweight:
  Memospot is designed to be lightweight and efficient, minimizing resource usage on your device.
- Cross-platform:
  Available for Windows, macOS, and Linux.

## Preview

<div align="center">
    <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/memospot/memospot/main/assets/capture_dark.webp" />
        <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/memospot/memospot/main/assets/capture_light.webp" />
        <img alt="Demo screen" src="https://raw.githubusercontent.com/memospot/memospot/main/assets/capture_light.webp" />
    </picture>
</div>

## Requirements

- Windows 10, macOS, or Linux
- System WebView (Edge, Safari, or WebKit2GTK).

> [!TIP]
> On Windows, you will be prompted to install WebView2, if it's not available.

## Installation

Download the latest release for your platform from the [GitHub Releases](https://github.com/memospot/memospot/releases)
page.

> [!WARNING]
>
> - On Windows, you may get a warning from Windows Defender SmartScreen, as the
>   app is not digitally signed. To solve it, click `More info` and then `Run anyway`.
> - On macOS, you may need to
>   [allow the app to run](https://support.apple.com/guide/mac-help/open-a-mac-app-from-an-unidentified-developer-mh40616/mac)
>   first.
>
>   ```bash
>   # Remove the quarantine attribute:
>   xattr -dr com.apple.quarantine /Applications/Memospot.app
>   ```

## Extra information

For advanced configuration, troubleshooting and standalone Memos server updates, see:

<https://memospot.github.io/>

## Data Portability

> [!TIP]
> Data can be seamlessly moved between Memos containers and Memospot instances, regardless of the operating system.

> [!IMPORTANT]
> If your data is coming from Memos v0.18.1 or earlier, see
> [Memospot data migration](https://memospot.github.io/guides/data-migration/).

## Special Thanks

This project is made possible by the following open-source projects:

<div align="center">
  <a href="https://www.usememos.com/">
    <picture>
      <source
        media="(prefers-color-scheme: dark)"
        srcset="https://raw.githubusercontent.com/memospot/memospot/main/assets/powered_by_memos_dark.webp"
      />
      <source
        media="(prefers-color-scheme: light)"
        srcset="https://raw.githubusercontent.com/memospot/memospot/main/assets/powered_by_memos.webp"
      />
      <img height="128"
        alt="powered by memos"
        src="https://raw.githubusercontent.com/memospot/memospot/main/assets/powered_by_memos.webp"
      />
    </picture>
  </a>

[![Homepage](https://img.shields.io/badge/Home-blue)](https://www.usememos.com) [![Blog](https://img.shields.io/badge/Blog-gray)](https://www.usememos.com/blog) [![Docs](https://img.shields.io/badge/Docs-blue)](https://www.usememos.com/docs) [![Live Demo](https://img.shields.io/badge/Live-Demo-blue)](https://demo.usememos.com/) [![Memos Discord](https://img.shields.io/badge/Discord-chat-5865f2?logo=discord&logoColor=f5f5f5)](https://discord.gg/tfPJa4UmAv) [![GitHub Stars](https://img.shields.io/github/stars/usememos/memos?logo=github)](https://github.com/usememos/memos)

  <a href="https://tauri.app/">
    <picture>
      <source
        media="(prefers-color-scheme: dark)"
        srcset="https://raw.githubusercontent.com/memospot/memospot/main/assets/made_with_tauri_dark.webp"
      />
      <source
        media="(prefers-color-scheme: light)"
        srcset="https://raw.githubusercontent.com/memospot/memospot/main/assets/made_with_tauri.webp"
      />
      <img height="128"
        alt="made with Tauri"
        src="https://raw.githubusercontent.com/memospot/memospot/main/assets/made_with_tauri.webp"
      />
    </picture>
  </a>
</div>

## Supporting

You can show your support for this project by [⭐starring](https://github.com/memospot/memospot) it on GitHub.

<details>
<summary>Star History</summary>
    <div align="center">
        <picture>
          <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=memospot/memospot,memospot/memos-builds&type=Date&theme=dark" />
          <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=memospot/memospot,memospot/memos-builds&type=Date" />
          <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=memospot/memospot,memospot/memos-builds&type=Date" />
        </picture>
    </div>
</details>
