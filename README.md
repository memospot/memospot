<img height="128" src="assets/app-icon-lossless.webp" alt="app-icon" align="right" />

# Memospot

[Memospot](https://github.com/lincolnthalles/memospot) is a self-contained desktop version of [Memos](https://github.com/usememos/memos), a beautiful, privacy-first, lightweight note-taking service.

This project is designed for people who:

- Are new to Docker
- Prefer not to host Memos themselves
- Wish to run Memos on their local machine with no Docker overhead
- Are interested in testing Memos before spinning up a Docker instance.

[![Downloads](https://img.shields.io/github/downloads/lincolnthalles/memospot/total?logo=github)](https://github.com/lincolnthalles/memospot/releases) [![GitHub Stars](https://img.shields.io/github/stars/lincolnthalles/memospot?logo=github)](https://github.com/lincolnthalles/memospot)

## Screenshots

<p align="center" width="100%">

![demo dark](/assets/capture_dark.webp)
![demo light](/assets/capture_light.webp)

</p>

## Key features

- Easy to install and use
- Bundled Memos server build, based on official release tags
- Automated server startup and shutdown
- Internal Memos server listens only on localhost, at a random port

## Requirements

- Windows 10, macOS, or Linux
- System WebView (Edge, Safari, or WebkitGTK).
- Memos (bundled)

> On Windows, you will be prompted to install WebView2, if it's not available.

## Installation

Download the latest release for your platform from the [releases page](https://github.com/lincolnthalles/memospot/releases).

> - On Windows, you may get a warning from Windows Defender SmartScreen. This is because the app is not signed.
> Just click `More info` and `Run anyway`.
>
> - Binaries are packed with UPX. This may trigger false positives on some antivirus software. You can unpack the binaries with `upx -d memos*`, if you will.
>
> - On macOS, you may need to [allow the app to run](https://support.apple.com/guide/mac-help/open-a-mac-app-from-an-unidentified-developer-mh40616/mac) first.

## Manual server update

Download the latest server release from [memos-builds](https://github.com/lincolnthalles/memos-builds) and replace the `memos` binary in the installation folder.

### Windows Updater script

Open Powershell and run the following command:

```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force; iex ((New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/lincolnthalles/memospot/main/memos-server-updater.ps1'))
```

## Data Portability

If you plan to move your Memos instance to Docker in the future, keep the object storage setting at the default option `Database` for a drop-in migration. Just move `memos_prod.db` to the Docker volume and that's it.

Although, if you are storing huge files, it's a good idea to use `Local Storage` instead, even with this trade-off.

Data storage:

- Windows: data is stored under `%AppData%\memospot`
  
- POSIX systems: data is stored under `~/.memospot`

## Special Thanks

This project is made possible by the following open-source projects:

<p align="center" width="100%">
  <a href="https://tauri.app/">
    <picture>
      <source
        media="(prefers-color-scheme: dark)"
        srcset="docs/images/made_with_tauri_dark.webp"
      />
      <source
        media="(prefers-color-scheme: light)"
        srcset="docs/images/made_with_tauri.webp"
      />
      <img height="128"
        alt="made with Tauri"
        src="docs/images/made_with_tauri.webp"
      />
    </picture>
  </a>
</p>

<p align="center" width="100%">
  <a href="https://www.usememos.com/">
    <picture>
      <source
        media="(prefers-color-scheme: dark)"
        srcset="docs/images/powered_by_memos_dark.webp"
      />
      <source
        media="(prefers-color-scheme: light)"
        srcset="docs/images/powered_by_memos.webp"
      />
      <img height="128"
        alt="powered by memos"
        src="docs/images/powered_by_memos.webp"
      />
    </picture>
  </a>
</p>

<div align="center" width="100%" style="display: flex; justify-content: center;">
  <p align="center" width="100%">

[![Homepage](https://img.shields.io/badge/Home-blue)](https://www.usememos.com) [![Blog](https://img.shields.io/badge/Blog-gray)](https://www.usememos.com/blog) [![Docs](https://img.shields.io/badge/Docs-blue)](https://www.usememos.com/docs) [![Live Demo](https://img.shields.io/badge/Live-Demo-blue)](https://demo.usememos.com/) [![Memos Discord](https://img.shields.io/badge/Discord-chat-5865f2?logo=discord&logoColor=f5f5f5)](https://discord.gg/tfPJa4UmAv) [![GitHub Stars](https://img.shields.io/github/stars/usememos/memos?logo=github)](https://github.com/usememos/memos)

  </p>
</div>

## Star History

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=lincolnthalles/memospot,lincolnthalles/memos-builds&usememos/memos&type=Date&theme=dark" />
  <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=lincolnthalles/memospot,lincolnthalles/memos-builds&usememos/memos&type=Date" />
  <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=lincolnthalles/memospot,lincolnthalles/memos-builds&usememos/memos&type=Date" />
</picture>

## Support

If you like this project, don't forget to [⭐star](https://github.com/lincolnthalles/memospot) it and consider supporting my work:

<p align="center" width="100%">

  <a href="https://www.buymeacoffee.com/lincolnthalles">
    <img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" />
  </a>
</p>
