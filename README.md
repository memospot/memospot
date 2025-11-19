# Memospot

<img height="128" src="assets/app-icon-lossless.webp" alt="app-icon" align="right" />

Self-contained desktop version of [Memos](https://github.com/usememos/memos) -a privacy-first, lightweight note-taking
service. Available for Windows, macOS, and Linux.

This project allows you to use Memos locally without the Docker overhead or to easily test Memos before deploying a
container. Data can be seamlessly moved between instances.

[![Downloads](https://img.shields.io/github/downloads/memospot/memospot/total?logo=github)](https://github.com/memospot/memospot/releases) [![GitHub Stars](https://img.shields.io/github/stars/memospot/memospot?logo=github)](https://github.com/memospot/memospot)

## Screenshot

<div align="center">
    <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/memospot/memospot/main/assets/capture_dark.webp" />
        <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/memospot/memospot/main/assets/capture_light.webp" />
        <img alt="Demo screen" src="https://raw.githubusercontent.com/memospot/memospot/main/assets/capture_light.webp" />
    </picture>
</div>

## Key features

- Easy-to-use
- Full-featured [Memos experience](https://www.usememos.com/docs)
- Minimal overhead
- Data portability
- Enhanced privacy

## Requirements

- Windows 10, macOS, or Linux
- System WebView (Edge, Safari, or WebkitGTK).

> [!TIP]
> On Windows, you will be prompted to install WebView2, if it's not available.

## Installation

Download the latest release for your platform from the [GitHub Releases](https://github.com/memospot/memospot/releases)
page.

> [!WARNING]
>
> - On Windows, you may get a warning from Windows Defender SmartScreen.
>   This is because the app is not digitally signed. To solve, click `More info` and then `Run anyway`.
> - On macOS, you may need to
>   [allow the app to run](https://support.apple.com/guide/mac-help/open-a-mac-app-from-an-unidentified-developer-mh40616/mac)
>   first.
>
>   ```bash
>   # Remove the quarantine attribute:
>   xattr -dr com.apple.quarantine /Applications/Memospot.app
>
>   # Self-sign the app:
>   codesign -s - --deep --force /Applications/Memospot.app/Contents/MacOS/memospot
>   codesign -s - --deep --force /Applications/Memospot.app/Contents/MacOS/memos
>   ```

## Extra information

For advanced configuration, troubleshooting and standalone Memos server updates, see <https://memospot.github.io/>.

## Data Portability

> [!TIP]
> The current version of Memos stores assets in a portable format.
>
> Data can be seamlessly moved between Memos containers and Memospot instances, regardless of the operating system.

> [!IMPORTANT]
> If your data is coming from Memos v0.18.1 or earlier, see
> [Memospot data migration](https://memospot.github.io/data-migration#migrating-data-from-earlier-memos-versions).

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

If you appreciate this project, be sure to [⭐star](https://github.com/memospot/memospot) it on GitHub.

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
