# https://just.systems
#
# Run `just` in the root of the project to see a list of recipes relevant to manual builds.

set shell := ["bash", "-c"]
CI := env_var_or_default("CI", "false")
NPROC := env_var_or_default("NPROC", num_cpus())
GITHUB_ENV := env_var_or_default("GITHUB_ENV", ".GITHUB_ENV")

PATH := if os() == "windows" {
		env_var_or_default('PROGRAMFILES', 'C:\Program Files') + '\Git\usr\bin;' + env_var_or_default('PATH','')
	} else {
		env_var_or_default('PATH','')
	}
bash := if os() == "windows" { "env -S bash -euo pipefail" } else { "/usr/bin/env -S bash -euo pipefail" }
powershell := if os() == 'windows' {'powershell.exe'} else {'/usr/bin/env pwsh'}

REPO_ROOT := justfile_directory()
DPRINT_CACHE_DIR := absolute_path(join(REPO_ROOT,".dprint"))
RUST_BACKTRACE := "full"

SCCACHE_BIN := `which sccache || echo ""`
SCCACHE_ENABLED := if SCCACHE_BIN == "" { "false" } else { path_exists(SCCACHE_BIN) }
RUSTC_WRAPPER := if SCCACHE_ENABLED == "true" { SCCACHE_BIN } else { env_var_or_default("RUSTC_WRAPPER", "") }
SCCACHE_DIR := absolute_path(join(REPO_ROOT,".sccache"))
CARGO_INCREMENTAL := if SCCACHE_ENABLED == "true" { "0" } else { "1" }

RESET := '\033[0m'
BOLD := '\033[1m'
DIM := '\033[2m'
UNDERLINE := '\033[4m'
BLACK := '\033[30m'
RED := '\033[31m'
GREEN := '\033[32m'
YELLOW := '\033[33m'
BLUE := '\033[34m'
MAGENTA := '\033[35m'
CYAN := '\033[36m'
WHITE := '\033[37m'

set export

[private]
default:
    #!/usr/bin/env bash
    echo -e "${BOLD}This justfile contains recipes for building ${UNDERLINE}https://github.com/memospot/memospot${RESET}.\n"
    if [[ "{{os()}}" == "windows" ]]; then
        program_files="{{replace(env_var_or_default('PROGRAMFILES', 'C:\Program Files'), '\\', '\\\\')}}"
        echo -e "To use this justfile on Windows, make sure Git is installed under ${BOLD}${UNDERLINE}$program_files\\Git${RESET}."
        echo -e "${BOLD}${UNDERLINE}https://git-scm.com/download/win${RESET}"
        echo ""
    fi
    deps=(
        "bash"
        "bun"
        "cargo"
        "dprint"
        "git"
        "rustc"
    )
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            echo -e "${RED}ERROR:${RESET} Please install ${MAGENTA}${BOLD}${UNDERLINE}$dep${RESET}."
            echo -e "Try running ${BOLD}${UNDERLINE}just setup${RESET}."
            exit 1
        fi
    done
    echo -e "${GREEN}Found project dependencies: ${deps[@]}${RESET}"
    echo -e "${YELLOW}This quick test does not verify tool versions. If you experience any errors, consider updating the related tool.${RESET}\n"
    just --list

[private]
deps-ts:
    bun install || bun install

[private]
deps-rs:
    mkdir -p ./dist-ui

[private]
dev-ui: deps-ts
    cd "src-ui"; bun x vite

[private]
download-memos-binaries: deps-ts
    bun run ./build-scripts/downloadMemosBuildsHook.ts

# Tauri hooks
[private]
tauri-before-build: download-memos-binaries gen-icons build-ui

[private]
tauri-before-bundle: deps-ts
    bun run ./build-scripts/upxPackHook.ts; exit 0

[private]
tauri-before-dev: download-memos-binaries dev-ui
# /Tauri hooks

[group('test')]
[doc('Run all tests')]
test: test-ts test-crates test-rs test-tauri

[group('test')]
[doc('Run all crate tests')]
test-crates: deps-rs
    cargo test --workspace --exclude memospot --lib -- --nocapture

[group('test')]
[doc('Run all Rust tests')]
test-rs: deps-rs
    cargo test --workspace --lib -- --nocapture

[group('test')]
[doc('Run all Tauri tests')]
test-tauri: deps-rs
    cargo test --package memospot --lib -- --nocapture

[group('test')]
[doc('Run all TypeScript tests')]
test-ts: deps-ts
    #!/usr/bin/env bun
    import fs from "node:fs";
    const dirs = ["./build-scripts", "./src-ui"];
    let results = [];
    for (const dir of dirs) {
        const files = fs.readdirSync(dir, {recursive: true}).filter(fn => fn.endsWith('.test.ts'));
        if (files.length === 0) {
            continue;
        }
        console.log(`> Running tests in ${dir}`);
        await Bun.spawn({
            cmd: ["bun", "test"],
            cwd: dir,
            onExit(proc, exitCode, signalCode, error) {
                results.push(exitCode === 0);
            }
        }).exited;
    }
    const allPassed = results.every((result) => result);
    allPassed ? console.log("All tests passed.") : console.error("Some tests failed.");
    process.exit(allPassed ? 0 : 1);

[doc('Run app in development mode')]
dev: dev-killprocesses
    cargo tauri dev
    just dev-killprocesses sccache-stats

[private]
[linux]
[macos]
dev-killprocesses:
    #!/usr/bin/env bash
    processes=("memospot" "memos")
    for process in "${processes[@]}"; do
        killall $process > /dev/null 2>&1 || true
    done
[private]
[windows]
dev-killprocesses:
    #!{{powershell}}
    [System.Diagnostics.Process]::GetProcesses() | Where-Object { $_.ProcessName -eq "memospot" -or $_.ProcessName -eq "memos" } | Stop-Process -Force -ErrorAction SilentlyContinue

[group('update')]
[doc('Update project dependencies')]
update: update-dprint update-rs update-ts

[group('update')]
[doc('Update dprint plugins')]
update-dprint:
    dprint config update

[group('update')]
[doc('Update cargo crates')]
update-rs:
    cargo update

[group('update')]
[doc('Update npm packages')]
update-ts:
    bun update
    pushd "./src-ui"; bun update; popd
    pushd "./build-scripts"; bun update; popd
    just fmt

[group('update')]
[doc('Update Rust toolchain via rustup')]
update-rust-toolchain:
    rustup update
    rustup self update
    rustup component add clippy

gen-icons-force:
    #!/usr/bin/env bash
    cargo tauri icon "assets/app-icon-lossless.webp"
    cp -f "./src-tauri/icons/icon.ico" "./src-ui/public/favicon.ico"
    git add "assets/app-icon-lossless.webp" "src-tauri/icons/*"
    # git commit -m "chore: regenerate icons"

[doc('Generate app icons, if needed')]
gen-icons:
    #!/usr/bin/env bash
    if [ "$CI" = "true" ]; then exit 0; fi
    check_files=(
        "assets/app-icon-lossless.webp"
        "src-tauri/icons/**.png"
        "src-tauri/icons/icon.ico"
        "src-ui/public/favicon.ico"
    )
    for file in "${check_files[@]}"; do
        if ! git diff --quiet --exit-code HEAD -- "$file"; then
            echo "${YELLOW}$file was modified, regenerating icons...${RESET}"
            just gen-icons-force
            exit 0
        fi
    done
    echo -e "${GREEN}App icons are up to date.${RESET}"

build-ui-force:
    cd "src-ui"; bun run build

[doc('Build UI, if needed')]
build-ui:
    #!/usr/bin/env bash
    if ! git diff --quiet --exit-code HEAD -- "src-ui/src/**/*"; then
        just build-ui-force && exit 0
    fi
    if ! [ -d "./dist-ui/" ]; then
        just build-ui-force && exit 0
    fi
    echo -e "${GREEN}UI is up to date.${RESET}"

[doc('Build app')]
build:
    #!/usr/bin/env bash
    export RUSTFLAGS="-Ctarget-cpu=native -Copt-level=3 -Cstrip=symbols -Ccodegen-units=8"
    cargo tauri build
    just sccache-stats postbuild

[private]
postbuild:
    #!/usr/bin/env bash
    echo -e "${CYAN}Moving relevant build files to `./build` directory...${RESET}"
    mkdir -p ./build
    artifacts=(
        "bundle/appimage/*.AppImage"
        "bundle/deb/*.deb"
        "bundle/msi/*.msi"
        "bundle/nsis/*.exe"
        "dist/"
        "memos"
        "memos.exe"
        "memospot"
        "memospot.exe"
    )
    for file in "${artifacts[@]}"; do
        resolved_path="$(find ./target/release/$file 2>&1 | head -n 1)"
        if [ -f "$resolved_path" ]; then
            mv -f "$resolved_path" ./build/. 2>/dev/null
        fi
    done
    appimage="$(find ./build/*.AppImage 2>&1 | head -n 1)"
    if [ -f "$appimage" ]; then
        mkdir -p "${appimage}.home"
    fi
    if ls ./build/memos* 1> /dev/null 2>&1; then
        echo -e "${GREEN}Done.${RESET}"
    else
        echo -e "${RED}Failed to move files.${RESET}"
    fi

[doc('Clean project artifacts')]
clean: sccache-clean
    #!/usr/bin/env bash
    bun pm cache rm || true
    cargo cache -a || true
    dirs=(
        "./.dprint"
        "./.sccache"
        "./.task"
        "./build"
        "./build-scripts/node_modules"
        "./dist-ui"
        "./node_modules"
        "./server-dist"
        "./src-ui/.vite"
        "./src-ui/node_modules"
        "./target"
    )
    for item in "${dirs[@]}"; do
        if [ -d "$item" ]; then
            rm -rf "$item"
        fi
    done

[group('sccache')]
sccache-clean:
    #!/usr/bin/env bash
    if [ -z $RUSTC_WRAPPER ]; then exit 0; fi
    sccache --stop-server || true && rm -rf ./.sccache

[group('sccache')]
sccache-stats:
    #!/usr/bin/env bash
    if [ $SCCACHE_ENABLED = "false" ]; then
        echo -e "$YELLOW -- sccache is disabled -- $RESET"
        exit 0
    fi
    echo -e "$CYAN -- sccache stats -- $RESET"
    sccache --show-stats;

[group('lint')]
[doc('Run all code linters')]
lint: lint-dprint lint-ts lint-rs

[group('lint')]
[doc('Check code formatting')]
lint-dprint:
    dprint check

[group('lint')]
[doc('Lint Rust code with cargo fmt and clippy')]
lint-rs: deps-rs
    cargo fmt --all --check
    cargo clippy --all-features --all-targets --workspace --locked

[group('lint')]
[doc('Lint TypeScript code with BiomeJS')]
lint-ts:
    #!/usr/bin/env bash
    dirs=(
        "./build-scripts"
        "./src-ui"
    )
    for d in "${dirs[@]}"; do
        cd "$REPO_ROOT/$d"
        if ls *.ts 1> /dev/null 2>&1; then
            bunx @biomejs/biome ci .
        fi
    done

[group('fix')]
[doc('Run all code fixes')]
fix: fix-ts fix-rs

[group('fix')]
[doc('Run cargo fix (requires clean repo)')]
fix-rs:
    cargo fix || just fix-rs-dirty

[group('fix')]
[confirm('This will run `cargo fix --allow-dirty`. This can perform destructive changes. Are you sure?')]
fix-rs-dirty:
    cargo fix --allow-dirty

[group('fix')]
[doc('Run BiomeJS safe fixes')]
fix-ts:
    #!/usr/bin/env bash
    dirs=(
        "./build-scripts"
        "./src-ui"
    )
    for d in "${dirs[@]}"; do
        cd "$REPO_ROOT/$d"
        if ls *.ts 1> /dev/null 2>&1; then
            bun x @biomejs/biome lint --apply .
        fi
    done

[group('format')]
[doc('Format code with dprint (json, rust, toml, yaml, html, css, typescript and markdown).')]
fmt:
    dprint fmt --diff

[group('setup')]
[doc('Install all project dependencies.')]
setup: setup-platformdeps setup-bun setup-rust setup-toolchain

[group('setup')]
[macos]
setup-platformdeps:
    xcode-select --install

[group('setup')]
[linux]
setup-platformdeps:
    #!/usr/bin/env bash
    sudo apt update -y
    sudo apt install -y \
        build-essential \
        curl \
        file \
        libgtk-3-dev \
        librsvg2-dev \
        libssl-dev \
        libwebkit2gtk-4.0-dev \
        patchelf \
        wget \
        git
    sudo apt install -y libappindicator3-dev 2>/dev/null || true
[group('setup')]
[windows]
setup-platformdeps:
    #!{{powershell}}
    Start-Process -Wait -Verb RunAs -FilePath "winget" -ArgumentList "install Microsoft.VisualStudio.2022.BuildTools"

[group('setup')]
[windows]
setup-bun:
    #!{{powershell}}
    $ErrorActionPreference = "SilentlyContinue"
    if (Get-Command "bun" -ErrorAction SilentlyContinue) {
        Write-Host "Bun is already installed."
    }
    else if (Get-Command "choco" -ErrorAction SilentlyContinue) {
        Write-Host "Installing Bun via Chocolatey..."
        Start-Process -Wait -Verb RunAs -FilePath "choco" -ArgumentList "install bun -y"
    }
    else if (Get-Command "winget" -ErrorAction SilentlyContinue) {
        Write-Host "Installing Bun via Winget..."
        winget install --id Oven-sh.Bun
    }
    else if (Get-Command "scoop" -ErrorAction SilentlyContinue) {
        Write-Host "Installing Bun via Scoop..."
        scoop install bun
    } else {
        Write-Host -ForegroundColor Red "[ERROR] No package manager found. Please install Bun manually."
        Write-Host "Alternatively, install Chocolatey, Winget or Scoop and run this task again."
        Write-Host -ForegroundColor Cyan "`n
        https://bun.sh
        https://chocolatey.org/install
        https://apps.microsoft.com/detail/9NBLGGH4NNS1
        https://scoop.sh/"
        Exit 1
    }
[group('setup')]
[linux]
[macos]
setup-bun:
    #!/usr/bin/env bash
    if ! [ -z $(command -v bun) ]; then
        echo "Bun is already installed." && exit 0
    fi
    if ! [ -z $(command -v brew) ]; then
        brew install oven-sh/bun/bun
    else
        echo -e "${RED}[ERROR] Homebrew not found. Please install Bun manually.${RESET}
        Alternatively, install Homebrew and run this task again.
        ${CYAN}
        https://bun.sh
        https://brew.sh
        ${RESET}"
    fi

[group('setup')]
[linux]
[macos]
setup-rust:
    #!/usr/bin/env bash
    if ! [ -z $(command -v rustup) ] && ! [ -z $(command -v rustc) ]; then
        echo "Rust is already installed." && exit 0
    fi
    if ! [ -z $(command -v brew) ]; then
        brew install rustup-init
        rustup-init -y
        source "$HOME/.cargo/env"
        rustup default stable
    else
        echo -e "${RED}[ERROR] Homebrew not found. Please install Rust manually.${RESET}
        Alternatively, install Homebrew and run this task again.
        ${CYAN}
        https://rustup.rs
        ${RESET}"
    fi

[group('setup')]
[windows]
setup-rust:
    #!{{powershell}}
    $ErrorActionPreference = "SilentlyContinue"
    if ((Get-Command "rustup") -and (Get-Command "rustc")) {
        Write-Host "Rust is already installed."
    } else if (Get-Command "choco") {
        Start-Process -Wait -Verb RunAs -FilePath "choco" -ArgumentList "install rustup.install -y"
    } else if (Get-Command "winget" -ErrorAction SilentlyContinue) {
        winget install --id Rustlang.Rustup
    } else if (Get-Command "scoop" -ErrorAction SilentlyContinue) {
        scoop install rustup
    } else {
        Write-Host -ForegroundColor Red "[ERROR] No package manager found. Please install Rustup manually."
        Write-Host "Alternatively, install Chocolatey, Winget or Scoop and run this task again."
        Write-Host -ForegroundColor Cyan "`n
        https://rustup.rs
        https://chocolatey.org/install
        https://apps.microsoft.com/detail/9NBLGGH4NNS1
        https://scoop.sh/"
        Exit 1
    }

[group('setup')]
setup-toolchain:
    #!/usr/bin/env bash
    rustup component add clippy
    rustup target add aarch64-apple-darwin x86_64-apple-darwin x86_64-pc-windows-msvc x86_64-unknown-linux-gnu
    cargo install cargo-binstall --locked -y
    cargo binstall \
        cargo-cache@0.8.3 \
        cargo-edit@0.12.3 \
        dprint@0.47.2 \
        sccache@0.8.1 \
        tauri-cli@1.6.0 \
        --locked --targets x86_64-unknown-linux-musl -y || exit 1

[group('maintainer')]
[doc('Delete all GitHub build cache')]
gh-clean-cache:
    gh cache delete --all

[group('maintainer')]
repo-status:
    #!/usr/bin/env bash
    if ! git diff-index --quiet HEAD --; then
        echo -e "${MAGENTA}There are unstaged changes.${RESET}"
    elif ! git diff-files --quiet; then
        echo -e "${MAGENTA}There are unstaged changes.${RESET}"
    elif ! git status; then
        echo -e "${MAGENTA}There are untracked files.${RESET}"
    elif ! [ -z "$(git ls-files --deleted)" ]; then
        echo -e "${MAGENTA}There are deleted files.${RESET}"
    elif ! [ -z "$(git ls-files --modified)" ]; then
        echo -e "${MAGENTA}There are modified files.${RESET}"
    else
        echo -e "${GREEN}Repository is clean.${RESET}"
        exit 0
    fi
    exit 1

[group('maintainer')]
[doc('Bump version in Cargo.toml and src-tauri/Cargo.toml')]
bumpversion VERSION:
    #!/usr/bin/env bash
    clean="{{trim_start_match(VERSION, "v")}}"
    pushd ./src-tauri; cargo set-version --package memospot --locked "$clean"; popd
    sed -i "s#Memospot/[0-9]\+\.[0-9]\+\.[0-9]\+\"#Memospot/$clean\"#" ./src-tauri/Tauri.toml || exit 1
    cargo generate-lockfile
    just fmt
    git add ./src-tauri/Cargo.toml ./src-tauri/Tauri.toml ./Cargo.lock
    git commit -m "chore: bump version to v$clean"

[group('maintainer')]
[doc('Push a new tag to the repository')]
pushtag TAG:
    #!/usr/bin/env bash
    clean="{{trim_start_match(TAG, "v")}}"
    git tag -a "v$clean" -m "chore: push v$clean"
    git push origin --tags

[group('maintainer')]
[doc('Publish a new version (bumpversion + pushtag)')]
publish TAG:
    just bumpversion {{TAG}}
    just pushtag {{TAG}}
