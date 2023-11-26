# About

[Memospot](https://github.com/lincolnthalles/memospot) is a self-contained desktop version of [Memos](https://github.com/usememos/memos), a beautiful, privacy-first, lightweight note-taking service.

This project is designed for people who:

- Are new to Docker
- Prefer not to host Memos themselves
- Wish to run Memos on their local machine with no Docker overhead
- Are interested in testing Memos before spinning up a Docker instance.

[![Downloads](https://img.shields.io/github/downloads/lincolnthalles/memospot/total?logo=github)](https://github.com/lincolnthalles/memospot/releases) [![GitHub Stars](https://img.shields.io/github/stars/lincolnthalles/memospot?logo=github)](https://github.com/lincolnthalles/memospot)

## Screenshots {collapsible="true" default-state="expanded"}

![demo](capture.webp){height=1499}

## Key features

- Easy to install and use
- Bundled Memos server build, based on official release tags
- Automated server startup and shutdown
- Internal Memos server listens only on localhost, at a random port

## Requirements

- Windows 10, macOS, or Linux
- System WebView (Edge, Safari, or WebkitGTK).

> On Windows, you will be prompted to install WebView2, if it's not available.

## Installation

Download the latest release for your platform from the [releases page](https://github.com/lincolnthalles/memospot/releases).

> - On Windows, you may get a warning from Windows Defender SmartScreen. This is because the app is not signed.
> Just click `More info` and `Run anyway`.
>
> - Binaries are packed with UPX. This may trigger false positives on some antivirus software. You can unpack the binaries with `upx -d memos*`, if you will.
>
> - On macOS, you may need to [allow the app to run](https://support.apple.com/guide/mac-help/open-a-mac-app-from-an-unidentified-developer-mh40616/mac) first.

## Data Portability {collapsible="true" default-state="expanded"}

If you plan to move your Memos instance to Docker in the future, keep the object storage setting at the default option `Database` for a drop-in migration. Just move `memos_prod.db` to the Docker volume and that's it.

Although, if you are storing huge files, it's a good idea to use `Local Storage` instead, even with this trade-off.

> If you ever need to do a Memos migration, check [Data Migration](MIGRATION.md).

## Special Thanks {collapsible="true" default-state="expanded"}

[![Powered by Memos](powered_by_memos.webp){height=128}](https://www.usememos.com/)

[![Homepage](https://img.shields.io/badge/Home-blue)](https://www.usememos.com) [![Blog](https://img.shields.io/badge/Blog-gray)](https://www.usememos.com/blog) [![Docs](https://img.shields.io/badge/Docs-blue)](https://www.usememos.com/docs) [![Live Demo](https://img.shields.io/badge/Live-Demo-blue)](https://demo.usememos.com/) [![Memos Discord](https://img.shields.io/badge/Discord-chat-5865f2?logo=discord&logoColor=f5f5f5)](https://discord.gg/tfPJa4UmAv) [![GitHub Stars](https://img.shields.io/github/stars/usememos/memos?logo=github)](https://github.com/usememos/memos)

[![Made with Tauri](made_with_tauri.webp){height=128}](https://tauri.app/)

## Star history {collapsible="true" default-state="collapsed"}

[![Star History Chart](https://api.star-history.com/svg?repos=usememos/memos,lincolnthalles/memospot&type=Date&theme=light)](https://star-history.com/#usememos/memos&lincolnthalles/memospot&Date)

---

## Supporting

If you like this project, don't forget to [⭐star](https://github.com/lincolnthalles/memospot) it and consider supporting my work:

[![Buy Me A Coffee](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/lincolnthalles)
