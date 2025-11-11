# Contributing

[Memospot](https://github.com/memospot/memospot) contributor's guide.

The recommended code editor is [Visual Studio Code](https://code.visualstudio.com/). The project has a pre-configured workspace, which will prompt you to install the recommended extensions.

> [!TIP]
> This project's workflow is heavily based on [Just](https://just.systems), a modern alternative to Makefiles and scripts.
>
> A `justfile` is available with lots of pre-configured [recipes](#using-just) for this project.
> Just installation is covered in the [Pre-requisites](#pre-requisites) section.

## Container build

It's possible to easily build the app for Linux and Windows (NSIS only) using [Docker](https://www.docker.com/) or [Podman](https://podman.io/), via [Earthly](https://earthly.dev/get-earthly). This bypasses the need to setup the base OS and other dependencies.

```bash
earthly +build --target=x86_64-pc-windows-msvc --nosign=1
```

```bash
earthly +build --target=x86_64-unknown-linux-gnu --nosign=1
```

> The `--nosign=1` flag is used to skip signing the updater, which only the repository owner can do.

Listing the additional recipes available in the [Earthfile](./Earthfile):

```bash
earthly ls
```

## Pre-requisites

- A package manager: [Homebrew](https://brew.sh/) or [winget](https://apps.microsoft.com/detail/9NBLGGH4NNS1).
- A system WebView (Edge [WebView2](https://go.microsoft.com/fwlink/p/?LinkId=2124703), Safari, or WebkitGTK), for Tauri to work.
- A modern computer, with at least 8 GB of RAM and a decent CPU. Rust compilation is very CPU-intensive. Also, `rust-analyzer` (language server) utilizes circa 2GB of RAM.
- 20 GB of free disk space, for Rust artifacts.

> [!IMPORTANT]
> After installing a tool, make sure it is available in your system PATH environment variable.
>
> Open a new terminal window and try to run the tool from there. If it doesn't work, read the tool's installation instructions thoroughly and setup it accordingly.

### OS-specific dependencies

See also: [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)

<details>
<summary>Linux</summary>

```bash
sudo apt update -y &&
sudo apt install --no-install-recommends -qq \
    autoconf \
    autotools-dev \
    build-essential \
    ca-certificates \
    curl \
    file \
    patchelf \
    wget \
    git \
    unzip \
    nsis \
    clang \
    lld \
    llvm \
    libayatana-appindicator3-dev \
    libgtk-3-dev \
    librsvg2-dev \
    libssl-dev \
    libwebkit2gtk-4.1-dev \
    libxdo-dev \
    -y
```

> [!NOTE]
> Should you experience an error regarding `soup3-sys`, try manually setting
> `PKG_CONFIG_PATH` with `export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig/:/usr/share/pkgconfig"`.
> You can make this permanent by adding the line to your `.bashrc` or `.bash_profile` file.

</details>

<details>
<summary>macOS</summary>

```bash
xcode-select --install
```

</details>

<details>
<summary>Windows</summary>

[Git](https://git-scm.com/downloads/win)

```powershell
winget install --id Git.Git -e --source winget
```

[Build Tools for Visual Studio 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

```powershell
winget install -e --id Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

[Edge WebView2](https://developer.microsoft.com/microsoft-edge/webview2/#download-section)

```powershell
winget install --id=Microsoft.EdgeWebView2Runtime -e
```

</details>

### Bun

Bun is a fast JavaScript/TypeScript runtime, bundler, test runner, and package manager. It's used to bundle the front end and to run build scripts.

[Official Website](https://bun.sh) | [GitHub](https://github.com/oven-sh/bun)

- Homebrew

  ```bash
  brew install oven-sh/bun/bun
  ```

- Winget

  ```powershell
  winget install --id Oven-sh.Bun
  ```

### Node JS

Node is a JavaScript runtime built on Chrome's V8 JavaScript engine.

> [!NOTE]
> Bun should soon replace Node entirely on this project.
>
> Make sure Node is installed if you get any front-end build errors.

> [!TIP]
> Any version starting from Node 18 should work.

- Homebrew

  ```bash
  brew install node@22
  ```

- Winget

  ```powershell
  winget install --id OpenJS.NodeJS.LTS
  ```

### UPX (optional; Linux-only)

[UPX](https://upx.github.io/) is a packer for executable files.

> [!NOTE]
>
> - Not required to build the app, but it's recommended to reduce its size.
> - Disabled on Windows, as it may cause false-positive AV detections.

- Homebrew

  ```bash
  brew install upx
  ```

### Rust

[Rustup](https://rustup.rs/) | [Official Website](https://www.rust-lang.org/tools/install)

- Homebrew

  ```bash
  brew install rustup-init
  rustup-init -y
  source "$HOME/.cargo/env"
  ```

- Winget

  ```powershell
  winget install --id Rustlang.Rustup
  ```

### Rust toolchain

```bash
rustup default stable
rustup component add clippy rust-analyzer
```

### Cargo binstall

[Binstall](https://github.com/cargo-bins/cargo-binstall) is a tool for installing pre-built Rust binaries.

> [!NOTE]
> It's possible to use `cargo install` or download the binaries manually instead, but it will take a lot of time.

- Homebrew

```bash
brew install cargo-binstall
```

- Powershell

```powershell
Set-ExecutionPolicy Unrestricted -Scope Process; iex (iwr "https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.ps1").Content
```

- Cargo (build from source)

```bash
cargo install cargo-binstall --locked
```

#### Rust tools

- [cargo-cache](https://github.com/matthiaskrgr/cargo-cache): Utility to manage `cargo` cache.
- [cargo-edit](https://github.com/killercup/cargo-edit): Utility to manage `cargo` dependencies.
- [cargo-xwin](https://github.com/rust-cross/cargo-xwin): Cross compile Cargo projects to Windows.
- [dprint](https://github.com/dprint/dprint): A pluggable code formatter.
- [just](https://github.com/casey/just): A command runner.

##### Bash

```bash
cargo binstall \
    --disable-telemetry \
    --target=$(rustc -vV | sed -n 's|host: ||p') \
    cargo-cache@0.8.3 \
    cargo-edit@0.13.7 \
    cargo-xwin@0.20.2 \
    dprint@0.50.2 \
    just@1.43.0 \
    tauri-cli@2.9.4 \
    -y
```

> [!WARNING]
> Should you experience issues with `cargo-tauri` under macOS, like `bad CPU type in executable`, try building it manually:
>
> ```bash
> cargo install --locked --target=$(rustc -vV | sed -n 's|host: ||p') tauri-cli
> ```

##### Powershell

```powershell
cargo binstall `
    --disable-telemetry `
    --target=$(& rustc -vV | Select-String -Pattern "^host:" | ForEach-Object {$_.Line.Split(':')[1].Trim()}) `
    cargo-cache@0.8.3 `
    cargo-edit@0.13.7 `
    dprint@0.50.2 `
    just@1.43.0 `
    tauri-cli@2.9.4 `
    -y
```

> [!TIP]
>
> `cargo binstall` outputs the installed tools to `$HOME/.cargo/bin` which already should be in your PATH.

## Memos server build

[Memos server](https://github.com/usememos/memos) is built separately on the repository [memos-builds](https://github.com/memospot/memos-builds).

A pre-build hook will automatically download the latest release from the companion repository and put it in the `server-dist` folder. Downloaded files will be reused on subsequent builds.

> [!NOTE]
> If you build the server by yourself, you must put the appropriate server binary in the `server-dist` folder, so Tauri can bundle it with the app.

> [!WARNING]  
> Server binaries **must** be named using [target triples](https://clang.llvm.org/docs/CrossCompilation.html#target-triple).

Sample valid server binary names:

- Windows: `memos-x86_64-pc-windows-msvc.exe`
- Linux: `memos-x86_64-unknown-linux-gnu`
- macOS: `memos-x86_64-apple-darwin` / `memos-aarch64-apple-darwin`

> [!TIP]
> You can check your current system target triple with the command `rustc -vV`.

## Cloning the repository

```bash
git clone https://github.com/memospot/memospot.git
```

## Changing into the project directory

```bash
cd memospot
```

## Using Just

> [!IMPORTANT]
> Under Windows, you must allow PowerShell script execution:
>
> ```powershell
> Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
> ```

At this point, everything should be ready to go.

Listing all available Just recipes:

```bash
just --list
```

### Common recipes

- `just dev`: Run the app in development mode.
- `just build`: Build the app.
- `just fmt`: Run all code formatters.
- `just lint`: Run all code checkers/linters.
- `just test`: Run all available tests.
- `just clean`: Remove build artifacts and caches.

## Coding style

- Try your best match the existing code style.
- Run `just pre-commit` on the repository before submitting a pull request. This will run all the code formatters, linters and tests.

## License

By contributing, you agree that all your contributions will be licensed under the [MIT License](https://choosealicense.com/licenses/mit/).

In short, when you submit code changes, your submissions are understood to be under the same _MIT License_ that covers the project. Feel free to contact the maintainer if that's a concern.
