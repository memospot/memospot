# Development

The recommended IDE is [Visual Studio Code](https://code.visualstudio.com/). The project has a pre-configured workspace, which will prompt you to install the recommended extensions.

> This project's workflow is heavily based on [Task](https://taskfile.dev/installation), a modern alternative to Makefiles and scripts.
>
> A Taskfile is available with lots of pre-configured tasks for this project. {style="note"}

## Pre-requisites

- [Task](https://taskfile.dev/#/installation)
- A package manager: [Homebrew](https://brew.sh/), [Chocolatey](https://chocolatey.org/install#individual), [winget](https://docs.microsoft.com/windows/package-manager/winget/) or [Scoop](https://scoop.sh/).
- A system WebView (Edge [WebView2](https://go.microsoft.com/fwlink/p/?LinkId=2124703), Safari, or WebkitGTK), for Tauri to work.
- 20 GB of free disk space on the repository drive, for Rust artifacts and sccache.
- A modern computer, with at least 8 GB of RAM and a decent CPU. Rust compilation is _very_ CPU intensive. Also, rust-analyzer (language server used by Rust's VSCode extension) utilizes circa 2GB of RAM.

## Toolchain setup

After fulfilling the pre-requisites, open a terminal, cd into the project directory and run `task setup` to install the project toolchain.

If some step fails, you can install the tools manually.

### Manual setup {collapsible="true" default-state="collapsed"}

#### OS-specific dependencies {collapsible="true" default-state="collapsed"}

##### macOS

```bash
xcode-select --install
```

##### Linux

```bash
sudo apt update -y &&
sudo apt install -y \
  build-essential \
  curl \
  file \
  libgtk-3-dev \
  librsvg2-dev \
  libssl-dev \
  libwebkit2gtk-4.0-dev \
  patchelf \
  wget
# libayatana-appindicator3-1" may conflict with
# libappindicator3-dev, it's ok to ignore
sudo apt install -y libappindicator3-dev || true
```

#### Project dependencies {collapsible="true" default-state="collapsed"}

> After installing a tool, make sure it is available in your system PATH environment variable.
>
> Open a new terminal window and try to run the tool from there.
> If it doesn't work, read the tool's installation instructions and setup's output thoroughly.{style="warning"}

##### [Deno]

Deno is a JavaScript/TypeScript runtime. It's used to bundle the frontend and to run build scripts.

- [Official Website](https://deno.com/#installation)
- [GitHub](https://github.com/denoland/deno)

> After the installation, make sure Deno works from any terminal window.
>
> `deno --version`

##### [Rust]

Rust is used to build the Tauri app.

- [Rustup](https://rustup.rs/)
- [Official Website](https://www.rust-lang.org/tools/install)

Run `rustup default stable` to install the latest stable toolchain.

> After the installation, make sure `rustc` and `cargo` works in any terminal.
>
> `rustc --version`
>
> `cargo --version`

##### Extra Rust tools

- [clippy](https://github.com/rust-lang/rust-clippy): A collection of lints to catch common mistakes and improve Rust code.
- [sccache](https://github.com/mozilla/sccache): Used to speed up Rust builds.
- [taplo](https://github.com/tamasfe/taplo): A TOML parser and formatter.
- [cargo-edit](https://github.com/killercup/cargo-edit): A utility for managing cargo dependencies.
- [dprint](https://github.com/dprint/dprint): A pluggable and configurable code formatter.

All of these tools can be installed via `cargo install`, though it will take a while to compile them.

```bash
rustup component add clippy
cargo install tauri-cli --locked
cargo install cargo-edit --locked
cargo install taplo-cli --locked
cargo install dprint --locked
cargo install sccache --locked
```

> After the installation, make sure `cargo tauri`, `cargo set-version`, `taplo` `dprint` and `sccache` works in any terminal.
>
> `cargo install` outputs the builds to `$HOME/.cargo/bin`, which already should be in your PATH.

## Memos server build {collapsible="true" default-state="expanded"}

The Memos server is built separately on the repository [memos-builds](https://github.com/lincolnthalles/memos-builds).

A pre-build hook will automatically download the latest release from the repository and put it in the `server-dist` folder. Downloaded files will be reused on subsequent builds.

> If you build the server by yourself, you must put the appropriate server binary in the `server-dist` folder, so Tauri can bundle it with the app.
>
> Also, server binaries **must** be named
> using [target triples](https://clang.llvm.org/docs/CrossCompilation.html#target-triple). {style="warning"}

Sample valid server binary names:

- Windows: `memos-x86_64-pc-windows-msvc.exe`
- Linux: `memos-x86_64-unknown-linux-gnu`
- macOS: `memos-x86_64-apple-darwin` / `memos-aarch64-apple-darwin`

> You can check your current system target triplet with the command `rustc -vV`. {style="note"}

## Using Task {collapsible="true" default-state="expanded"}

After installing Task and cloning the project repository, you can open a terminal, `cd` into the project directory and run `task --list-all` to see all available tasks.

### Common tasks

- `task dev`: Run the app in development mode.
- `task build`: Build the app.
- `task format`: Run all code formatters in parallel.
- `task lint`: Run all code checkers/linters in parallel.
- `task fix`: Run available code fixers in parallel.
- `task test`: Run available tests.
- `task clean`: Remove ALL build artifacts and caches.
