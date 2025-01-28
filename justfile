# https://just.systems
#
# Run `just` in the root of the project to see a list of recipes relevant to manual builds.

# Backtick commands and recipes without a shebang are executed with the shell set here.
set shell := ['bash', '-c']
set windows-shell := ['powershell', '-Command']
set dotenv-load := true

bash := if os() == 'windows' { 'env -S bash -euo pipefail' } else { '/usr/bin/env -S bash -euo pipefail' }
powershell := if os() == 'windows' {'powershell.exe'} else {'/usr/bin/env pwsh'}
bun := if os() == 'windows' { 'bun.exe' } else { '/usr/bin/env bun' }

RUST_TOOLCHAIN := 'stable'
RUSTFLAGS := env_var_or_default('RUSTFLAGS','') + if RUST_TOOLCHAIN == 'stable' { '' } else { ' -Z threads='+num_cpus() }
CI := env_var_or_default('CI', 'false')
GITHUB_ENV := env_var_or_default('GITHUB_ENV', '.GITHUB_ENV')
TAURI_SIGNING_PRIVATE_KEY := env_var_or_default('TAURI_SIGNING_PRIVATE_KEY', '')
TAURI_SIGNING_PRIVATE_KEY_PASSWORD := env_var_or_default('TAURI_SIGNING_PRIVATE_KEY_PASSWORD', '')
PATH := if os() == 'windows' {
		env_var_or_default('PROGRAMFILES', 'C:\Program Files') + '\Git\usr\bin;' + env_var_or_default('PATH','')
	} else {
		env_var_or_default('PATH','')
	}
REPO_ROOT := justfile_directory()
DPRINT_CACHE_DIR := absolute_path(join(REPO_ROOT,'.dprint'))
RUST_BACKTRACE := 'full'
RUST_TARGETS := if os() == 'windows' {
    'x86_64-pc-windows-msvc'
} else if os() == "macos" {
    'aarch64-apple-darwin x86_64-apple-darwin'
} else {
    'x86_64-unknown-linux-gnu'
}
RUSTC_WRAPPER := env_var_or_default('RUSTC_WRAPPER', '')
CARGO_INCREMENTAL := if RUSTC_WRAPPER == '' { '1' } else { '0' }
TS_RS_EXPORT_DIR:= absolute_path(join(REPO_ROOT,'src-ui/src/lib/types/gen'))

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
    #!{{bash}}
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
    pushd "src-ui"; bun install; popd
    pushd "build-scripts"; bun install; popd

[private]
deps-rs:
    mkdir -p ./src-ui/build

[private]
dev-ui: deps-ts
    cd "src-ui"; bun x vite

[private]
download-memos-binaries: deps-ts
    bun run ./build-scripts/downloadMemos.ts

[private]
upx:
    #!{{bash}}
    bun run ./build-scripts/upxPack.ts || true

# Tauri hooks
[private]
tauri-before-build: download-memos-binaries gen-icons gen-bindings build-ui

[private]
tauri-before-bundle: deps-ts upx

[private]
tauri-before-dev: download-memos-binaries gen-icons gen-bindings dev-ui
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
    #!{{bash}}
    export CARGO_PROFILE_TEST_BUILD_OVERRIDE_DEBUG=true
    cargo test --workspace --lib -- --nocapture

[group('test')]
[doc('Run all Tauri tests')]
test-tauri: deps-rs
    cargo test --package memospot --lib -- --nocapture

[group('test')]
[doc('Run all TypeScript tests')]
test-ts: deps-ts
    #!{{bun}}
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
    rustup toolchain install $RUST_TOOLCHAIN
    cargo tauri dev
    just dev-killprocesses

[private]
[linux]
[macos]
dev-killprocesses:
    #!{{bash}}
    for process in "memospot" "memos"; do
        killall $process > /dev/null 2>&1 || true
    done
[private]
[windows]
dev-killprocesses:
    #!{{powershell}}
    [System.Diagnostics.Process]::GetProcesses() | Where-Object {
        $_.ProcessName -in "memospot", "memos"
    } | Stop-Process -Force -ErrorAction SilentlyContinue

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
    #!{{bash}}
    pushd "./src-ui"; bun update; popd
    pushd "./build-scripts"; bun update; popd
    just fmt

[group('upgrade')]
[doc('Upgrade project toolchain')]
upgrade: upgrade-rust upgrade-bun

[group('upgrade')]
[doc('Upgrade Rust toolchain')]
upgrade-rust:
    rustup update
    rustup self update
    rustup component add clippy

[group('upgrade')]
[doc('Upgrade bun runtime')]
upgrade-bun:
    bun upgrade

gen-icons-force:
    #!{{bash}}
    cargo tauri icon "assets/app-icon-lossless.webp"
    cp -f "./src-tauri/icons/icon.ico" "./src-ui/static/favicon.ico"
    git add "assets/app-icon-lossless.webp" "src-tauri/icons/*"
    # git commit -m "chore: regenerate icons"

[doc('Generate app icons, if needed')]
gen-icons:
    #!{{bash}}
    if [ "$CI" = "true" ]; then exit 0; fi
    check_files=(
        "assets/app-icon-lossless.webp"
        "src-tauri/icons/**.png"
        "src-tauri/icons/icon.ico"
        "src-ui/static/favicon.ico"
    )
    for file in "${check_files[@]}"; do
        if ! git diff --quiet --exit-code HEAD -- "$file"; then
            echo "${YELLOW}$file was modified since last commit, regenerating icons…${RESET}"
            just gen-icons-force
            exit 0
        fi
    done
    echo -e "${GREEN}App icons are up to date.${RESET}"

gen-bindings:
    #!{{bash}}
    mkdir -p "$TS_RS_EXPORT_DIR"
    echo -e "${CYAN}Generating TypeScript bindings… This might take a while.${RESET}"
    cargo test export_bindings

build-ui-force:
    cd "src-ui"; bun run build

[doc('Build UI, if needed')]
build-ui:
    #!{{bash}}
    if ! git diff --quiet --exit-code HEAD -- "src-ui/src/**" || [ ! -d "./src-ui/build/" ] || [ ! -f "./src-ui/build/index.html" ]; then
        just build-ui-force
    else
        echo -e "${GREEN}UI is up to date.${RESET}"
    fi

[doc('Build app')]
build TARGET='all':
    #!{{bash}}
    if [ "{{TARGET}}" = "no-bundle" ]; then
        cargo tauri build --no-bundle
        just postbuild
        exit 0
    fi
    if [ -z $TAURI_SIGNING_PRIVATE_KEY ] && [ -f $HOME/.tauri/memospot_updater.key ]; then
        export TAURI_SIGNING_PRIVATE_KEY=$(cat $HOME/.tauri/memospot_updater.key 2>/dev/null | tr -d '\n' || echo "")
        echo -e "${CYAN}Setting TAURI_SIGNING_PRIVATE_KEY from $HOME/.tauri/memospot_updater.key${RESET}"
    fi
    if [ -z $TAURI_SIGNING_PRIVATE_KEY ] || [ -z $TAURI_SIGNING_PRIVATE_KEY_PASSWORD ]; then
        echo -e "${YELLOW}Environment not fully configured. Building without updater.${RESET}"
        cargo tauri build -c '{"bundle": {"targets": "{{TARGET}}" }, "plugins": {"updater": {}}}'
    else
        cargo tauri build -c '{"bundle": {"targets": "{{TARGET}}" }}'
    fi
    just postbuild

[linux]
flatpak-lint:
    #!{{bash}}
    flatpak install -y flathub org.flatpak.Builder
    shopt -s expand_aliases
    alias flatpak-builder-lint="flatpak run --command=flatpak-builder-lint org.flatpak.Builder"
    flatpak-builder-lint appstream ./installer/flatpak/io.github.memospot.Memospot.metainfo.xml
    flatpak-builder-lint manifest ./installer/flatpak/io.github.memospot.Memospot.yml
    flatpak-builder-lint repo ./target/flatpak-repo

[linux]
flatpak-build:
    #!{{bash}}
    just build deb
    flatpak-builder --force-clean --user --install-deps-from=flathub --repo=./target/flatpak-repo --install ./target/flatpak ./installer/flatpak/io.github.memospot.Memospot.yml

[linux]
flatpak-run:
    flatpak run io.github.memospot.Memospot

[linux]
debug-env:
    #!{{bash}}
    pid="$(pidof memospot)"
    test -z $pid && echo -e "${RED}Memospot is not running.${RESET}" && exit 1
    tr '\0' '\n' < /proc/$pid/environ | sort

[private]
postbuild:
    #!{{bash}}
    set +e
    echo -e "${CYAN}Moving relevant build files to ./build directory…${RESET}"
    ! test -d "./build" && mkdir -p "./build"
    artifacts=(
        "bundle/appimage/*.AppImage"
        "bundle/deb/*.deb"
        "bundle/msi/*.msi"
        "bundle/nsis/*.exe"
        "bundle/rpm/*.rpm"
        "dist/"
        "memos"
        "memos.exe"
        "memospot"
        "memospot.exe"
    )
    for artifact in "${artifacts[@]}"; do
        resolved_path="$(find ./target/release/$artifact -type f 2>&1 | head -n 1)"
        test -f "$resolved_path" && mv -f "$resolved_path" ./build/. 2>/dev/null
    done
    pushd "./build"
        appimages=($(find *.AppImage -type f 2>&1))
        for appimage in "${appimages[@]}"; do
            ! test -f "${appimage}" && continue
            ! test -d "${appimage}.home" && mkdir -p "${appimage}.home"
        done
    if ls ./memos* 1> /dev/null 2>&1; then
        echo -e "${GREEN}Done.${RESET}"
    else
        echo -e "${RED}Failed to move files.${RESET}"
    fi
    popd

[doc('Clean project artifacts')]
clean:
    #!{{bash}}
    set +e
    for d in "./src-ui" "./build-scripts"; do
        pushd "$d" && bun pm cache rm && popd
    done
    cargo cache -a || true
    dirs=(
        "./.dprint"
        "./.task"
        "./build"
        "./build-scripts/node_modules"
        "./node_modules"
        "./server-dist"
        "./src-ui/.vite"
        "./src-ui/build"
        "./src-ui/node_modules"
        "./target"
    )
    for item in "${dirs[@]}"; do
        test -d "$item" && rm -rf "$item"
    done
    exit 0

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
    #!{{bash}}
    for d in "./build-scripts" "./src-ui"; do
        cd "$REPO_ROOT/$d"
        if ls *.ts 1> /dev/null 2>&1; then
            bun x @biomejs/biome ci .
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
    #!{{bash}}
    for d in "./build-scripts" "./src-ui"; do
        cd "$REPO_ROOT/$d"
        if ls *.ts 1> /dev/null 2>&1; then
            bun x @biomejs/biome lint --apply .
        fi
    done

[group('format')]
[doc('Format code with dprint (json, rust, toml, yaml, html, css, typescript and markdown).')]
fmt:
    dprint fmt --diff

[group('maintainer')]
[doc('Delete all GitHub build cache')]
gh-clean-cache:
    gh cache delete --all

[group('maintainer')]
repo-status:
    #!{{bash}}
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
    #!{{bash}}
    clean="{{trim_start_match(VERSION, "v")}}"
    cargo set-version --locked "$clean"
    cargo generate-lockfile
    just fmt
    git add ./src-tauri/Cargo.toml ./src-tauri/Tauri.toml ./Cargo.lock ./Cargo.toml
    git commit -m "chore: bump version to v$clean"

[group('maintainer')]
[doc('Push a new tag to the repository')]
pushtag TAG:
    #!{{bash}}
    clean="{{trim_start_match(TAG, "v")}}"
    git tag -a "v$clean" -m "chore: push v$clean"
    git push origin --tags

[group('maintainer')]
[doc('Publish a new version (bumpversion + pushtag)')]
publish TAG:
    just bumpversion {{TAG}}
    just pushtag {{TAG}}
