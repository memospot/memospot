# Development

Recommended IDE Setup:
[Visual Studio Code](https://code.visualstudio.com/) + [Tauri Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

> To ease developer onboarding, it's highly recommended that you install [Task](https://taskfile.dev/#/installation).
>
> It's an alternative to Makefiles and shell scripts.
>
> A Taskfile is available with lots of pre-configured tasks for this project. {style="note"}

## Pre-requisites {collapsible="true" default-state="expanded"}

- [Task](https://taskfile.dev/#/installation)
- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/en/download/)
- [pnpm](https://pnpm.io/installation)
- [Tauri CLI](https://tauri.studio/en/docs/getting-started/intro)
- A system WebView (Edge [WebView2](https://go.microsoft.com/fwlink/p/?LinkId=2124703), Safari, or WebkitGTK)
- A Memos server build

## Memos build {collapsible="true" default-state="expanded"}

The Memos server is built separately. You may download a pre-built binary or build it yourself.

> You must put the server binary in the `server-dist` folder, so Tauri can bundle it with the app.
>
> Also, server binaries **must** be named
> using [target triplets](https://clang.llvm.org/docs/CrossCompilation.html#target-triple). {style="warning"}

Sample valid server binary names:

- Windows: `memos-x86_64-pc-windows-msvc.exe`
- Linux: `memos-x86_64-unknown-linux-gnu`
- macOS: `memos-x86_64-apple-darwin`

> You can check your current system target triplet with the command `rustc -vV`. {style="note"}

## Installing Task {collapsible="true" default-state="expanded"}

- Windows

```bash
# Chocolatey
choco install go-task
# Scoop
scoop install task
# winget
winget install Task.Task
```

- Linux / macOS

```bash
# Homebrew
brew install go-task/tap/go-task
# Install Script (outputs to ./bin)
sh -c "$(curl --location https://taskfile.dev/install.sh)" -- -d
```

> For additional installation methods, check the [Task docs](https://taskfile.dev/#/installation)
> and [Task GitHub](https://github.com/go-task/task)

## Using Task {collapsible="true" default-state="expanded"}

After installing Task, you can `cd` into the project directory and run `task --list-all` to see all available tasks.

```bash
~$cd memospot
~$task --list-all

* dev:                     Run the app in development mode
* dev:posix:               Use just `dev`
* dev:windows:             Use just `dev`
* build:                   Build the app
* build:posix:             Use just `build`
* build:windows:           Use just `build`
* format:                  Run all formatters in parallel
* format:backend:          format backend
* format:config:           format config
* format:frontend:         format frontend
* lint:                    Run all checkers/linters in parallel      (aliases: check)
* lint:backend:            lint backend
* lint:config:             lint config
* lint:frontend:           lint frontend
* gen:icons:               Generate app icons from ./assets/app-icon.png
* update-deps:             Update project dependencies
* update-toolchain:        Update project toolchain
* setup:                   Setup the project tooling. WARNING: not tested
* setup:pre:               Use just `setup`
* setup:pre:darwin:        Use just `setup`
* setup:pre:linux:         Use just `setup`
* setup:pre:windows:       Use just `setup`
```

For example, to run the app in development mode, just run `task dev`.
