# https://just.systems
#
# Run `just` in the root of the project to see a list of recipes relevant to manual builds.

# Backtick commands and recipes without a shebang are executed with the shell set here.
set shell := ['bash', '-c']
set windows-shell := ['powershell', '-Command']
set dotenv-load := true
set script-interpreter := ['bash']
export bash := if os() == 'windows' { 'env -S bash -euo pipefail' } else { '/usr/bin/env -S bash -euo pipefail' }
export powershell := if os() == 'windows' {'powershell.exe'} else {'/usr/bin/env pwsh'}
export BUN := if os() == 'windows' { 'bun.exe' } else { '/usr/bin/env bun' }
export RUST_TOOLCHAIN := env('RUST_TOOLCHAIN', 'stable')
export RUSTFLAGS := env('RUSTFLAGS','')
export CI := env('CI', 'false')
export GITHUB_ENV := env('GITHUB_ENV', '.GITHUB_ENV')

KEY_FILE := join(home_directory(), ".tauri", "memospot_updater.key")
KEY_FILE_CONTENTS :=  if path_exists(KEY_FILE)=="true" { trim_end(read(KEY_FILE))} else {""}
export TAURI_SIGNING_PRIVATE_KEY := env('TAURI_SIGNING_PRIVATE_KEY', KEY_FILE_CONTENTS)

export TAURI_SIGNING_PRIVATE_KEY_PASSWORD := env('TAURI_SIGNING_PRIVATE_KEY_PASSWORD', '')

GIT_WIN := join(env('PROGRAMFILES',''), 'Git','usr','bin')
export PATH := if os() == 'windows' { GIT_WIN +';'+ env('PATH') } else { env('PATH') }

export REPO_ROOT := justfile_directory()
export IS_STALE := BUN + ' run "./build-scripts/bin/is-stale.ts"'
export BIOME_CONFIG_PATH := join(REPO_ROOT,'biome.jsonc')
export DPRINT_CACHE_DIR := join(REPO_ROOT,'.dprint')
export RUST_BACKTRACE := 'full'
export RUST_TARGETS := if os() == 'windows' {
    'x86_64-pc-windows-msvc aarch64-pc-windows-msvc'
} else if os() == "macos" {
    'aarch64-apple-darwin x86_64-apple-darwin'
} else {
    'x86_64-unknown-linux-gnu'
}
export RUSTC_WRAPPER := env('RUSTC_WRAPPER', '')
export CARGO_INCREMENTAL := if RUSTC_WRAPPER == '' { '1' } else { '0' }
export TS_RS_EXPORT_DIR := join(REPO_ROOT,'src-ui','src','lib','types','gen')

[private]
[script('bash')]
default:
    echo "REPO_ROOT is ${REPO_ROOT}"
    echo -e "{{BOLD}}This justfile contains recipes for building {{UNDERLINE}}https://github.com/memospot/memospot{{NORMAL}}.\n"
    if [[ "{{os()}}" == "windows" ]]; then
        git_win="{{replace(GIT_WIN, '\\', '\\\\')}}"
        echo -e "To use this justfile on Windows, make sure Git is installed under {{BOLD}}{{UNDERLINE}}$git_win{{NORMAL}}."
        echo -e "{{BOLD}}{{UNDERLINE}}winget install --id Git.Git -e --source winget{{NORMAL}}"
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
            echo -e "{{RED}}ERROR:{{NORMAL}} Please install {{MAGENTA}}{{BOLD}}{{UNDERLINE}}$dep{{NORMAL}}."
            exit 1
        fi
    done
    echo -e "{{GREEN}}Found project dependencies: ${deps[@]}{{NORMAL}}"
    echo -e "{{YELLOW}}This quick test does not verify tool versions. If you experience any errors, consider updating the related tool.{{NORMAL}}\n"
    just --list

[private]
deps-ts:
    bun install

[private]
[script]
dev-ui: deps-ts
    bun --cwd="src-ui" run dev

[private]
[script]
download-memos: deps-ts
    downloaded=false
    for i in 1 2 3; do
        if bun run ./build-scripts/bin/download-memos.ts; then
            downloaded=true
            break
        fi
        [[ "$i" -lt 3 ]] && sleep 10
    done
    $downloaded || exit 1

# Tauri hooks
[private]
tauri-before-build: download-memos gen-icons gen-bindings build-ui

[private]
tauri-before-bundle: deps-ts

[private]
tauri-before-dev: download-memos gen-icons gen-bindings dev-ui
# /Tauri hooks

[group('test')]
[doc('Run all tests')]
test: test-ts test-crates test-rs test-tauri

[group('test')]
[doc('Run all crate tests')]
test-crates:
    cargo test --workspace --exclude memospot --lib -- --nocapture

[group('test')]
[doc('Run all Rust tests')]
[script]
test-rs:
    export CARGO_PROFILE_TEST_BUILD_OVERRIDE_DEBUG=true
    cargo test --workspace --lib -- --nocapture

[group('test')]
[doc('Run all Tauri tests')]
test-tauri:
    cargo test --package memospot --lib -- --nocapture

[group('test')]
[doc('Run all TypeScript tests')]
[script('bun')]
test-ts: deps-ts
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
    just dev-killprocesses

[private]
[linux]
[macos]
[script]
dev-killprocesses:
    for process in "memospot" "memos"; do
        killall $process > /dev/null 2>&1 || true
    done
[private]
[windows]
[script('powershell')]
dev-killprocesses:
    [System.Diagnostics.Process]::GetProcesses() | Where-Object {
        $_.ProcessName -in "memospot", "memos"
    } | Stop-Process -Force -ErrorAction SilentlyContinue

[group('update')]
[doc('Update project dependencies')]
[confirm('This will update all dependencies. This should be done carefully. Are you sure?')]
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
[script]
update-ts:
    bun update
    just fmt

[group('update')]
[doc('Show outdated npm packages')]
[script]
outdated-ts:
    bun outdated --filter @memospot/build-scripts
    bun outdated --filter @memospot/src-ui

[group('upgrade')]
[doc('Upgrade project toolchain')]
upgrade: upgrade-rust upgrade-bun

[group('upgrade')]
[doc('Upgrade Rust toolchain')]
upgrade-rust:
    rustup update
    rustup self update
    rustup component add clippy rust-analyzer

[group('upgrade')]
[doc('Upgrade bun runtime')]
upgrade-bun:
    bun upgrade

[doc('Generate app icons, if needed')]
[script]
gen-icons:
    [ "$CI" = "true" ] && exit 0
    src_icon="assets/app-icon-lossless.webp"
    task="crates/memospot/icons/gen-icons.task"
    if {{IS_STALE}} $task \
        -s "$src_icon" \
        -g "crates/memospot/icons/**/*.{icns,ico,png}" \
        -g src-ui/static/favicon.ico \
        ; then
        echo "{{YELLOW}}Source icon changed, regenerating icons…{{NORMAL}}"
        if cargo tauri icon "$src_icon"; then
            cp -f "./crates/memospot/icons/icon.ico" "./src-ui/static/favicon.ico"
            {{IS_STALE}} $task --update
        else
            exit 1
        fi
    else
        echo -e "{{GREEN}}App icons are up to date.{{NORMAL}}"
    fi

[script('bash')]
gen-bindings:
    mkdir -p "$TS_RS_EXPORT_DIR"
    # Files listed here may affect the frontend bindings.
    check_files=(
        "crates/config/**/*"
        "crates/memospot/src/runtime_config.rs"
    )
    for file in "${check_files[@]}"; do
        if ! git diff --quiet --exit-code HEAD -- "$file"; then
            echo -e "{{YELLOW}}${file} was modified since last commit, regenerating TypeScript bindings…{{NORMAL}}"
            pushd crates/memospot; cargo test export_bindings; popd
            exit 0
        fi
    done
    echo -e "{{CYAN}}Skipping TypeScript bindings regeneration, no changes detected.{{NORMAL}}"

[doc('Build UI, if needed')]
[script('bash')]
build-ui:
    if {{IS_STALE}} .build-stamps/build-ui.task \
        -s bun.lock \
        -s "src-ui/**" \
        -s "!src-ui/build/**" \
        -g "src-ui/build/**" \
        ; then

        bun run --cwd="$REPO_ROOT/src-ui" build

        {{IS_STALE}} .build-stamps/build-ui.task --update
    else
        echo -e "{{GREEN}}UI is up to date.{{NORMAL}}"
    fi

[doc('Build app')]
[script('bash')]
build TARGET='all':
    if [ "{{TARGET}}" = "no-bundle" ]; then
        cargo tauri build --no-bundle
        just postbuild
        exit 0
    fi
    if [ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" ] && [ -f "${HOME}/.tauri/memospot_updater.key" ]; then
        export TAURI_SIGNING_PRIVATE_KEY=$(cat "$HOME/.tauri/memospot_updater.key" 2>/dev/null | tr -d '\n' || echo "")
        echo -e "{{CYAN}}Setting TAURI_SIGNING_PRIVATE_KEY from $HOME/.tauri/memospot_updater.key{{NORMAL}}"
    fi
    if [ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" ] || [ -z "${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}" ]; then
        echo -e "{{YELLOW}}Environment not fully configured. Building without updater.{{NORMAL}}"
        cargo tauri build -c '{"bundle": {"targets": "{{TARGET}}" }, "plugins": {"updater": {}}}'
    else
        cargo tauri build -c '{"bundle": {"targets": "{{TARGET}}" }}'
    fi
    just postbuild

# Install Memospot to the system (requires sudo)
[group('install from source')]
[linux]
[script('bash')]
install: (build "no-bundle")
    memos_arch="$(uname -m)"
    case "$memos_arch" in
        x86_64) memos_target="x86_64-unknown-linux-gnu" ;;
        aarch64|arm64) memos_target="aarch64-unknown-linux-gnu" ;;
        *)
            echo -e "{{RED}}Unsupported Linux architecture: $memos_arch{{NORMAL}}"
            exit 1
            ;;
    esac
    memos_bin="server-dist/memos-$memos_target"
    if [ ! -f "$memos_bin" ]; then
        echo -e "{{RED}}Missing server binary: $memos_bin{{NORMAL}}"
        exit 1
    fi
    sudo install -Dm 755 target/release/memospot /usr/bin/memospot
    sudo install -Dm 755 "$memos_bin" /usr/bin/memos
    sudo install -Dm 644 crates/memospot/icons/32x32.png /usr/share/icons/hicolor/32x32/apps/memospot.png
    sudo install -Dm 644 crates/memospot/icons/128x128.png /usr/share/icons/hicolor/128x128/apps/memospot.png
    sudo install -Dm 644 crates/memospot/icons/128x128@2x.png /usr/share/icons/hicolor/256x256@2/apps/memospot.png
    sudo bash -c 'printf "%s\n" "[Desktop Entry]" "Categories=Utility;" "Comment=Memospot - a note-taking application" "Exec=memospot" "Icon=memospot" "Name=Memospot" "Terminal=false" "Type=Application" > /usr/share/applications/Memospot.desktop'
    [[ -x "$(command -v update-desktop-database)" ]] && sudo update-desktop-database /usr/share/applications
    [[ -x "$(command -v gtk-update-icon-cache)" ]] && sudo gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true

[group('install from source')]
[linux]
uninstall:
    sudo rm -f /usr/bin/memospot /usr/bin/memos
    sudo rm -f /usr/share/applications/Memospot.desktop
    sudo rm -f /usr/share/icons/hicolor/32x32/apps/memospot.png
    sudo rm -f /usr/share/icons/hicolor/128x128/apps/memospot.png
    sudo rm -f /usr/share/icons/hicolor/256x256@2/apps/memospot.png
    [[ -x "$(command -v update-desktop-database)" ]] && sudo update-desktop-database /usr/share/applications
    [[ -x "$(command -v gtk-update-icon-cache)" ]] && sudo gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true

[doc('Actions: prune, lint, test, linux, windows, linux-no-bundle, windows-no-bundle.')]
[group('Docker Bake')]
[script]
bake ACTION='':
    if [ "{{ACTION}}" = "prune" ]; then
        docker builder du
        docker builder prune -a
    else
        docker buildx bake {{ACTION}}
        test -d "./build" && sudo chown -R $(id -u):$(id -g) ./build
    fi

[linux]
[script]
flatpak-lint:
    flatpak install -y flathub org.flatpak.Builder
    shopt -s expand_aliases
    alias flatpak-builder-lint="flatpak run --command=flatpak-builder-lint org.flatpak.Builder"
    flatpak-builder-lint appstream ./installer/flatpak/io.github.memospot.Memospot.metainfo.xml
    flatpak-builder-lint manifest ./installer/flatpak/io.github.memospot.Memospot.yml
    flatpak-builder-lint repo ./target/flatpak-repo

[linux]
[script]
flatpak-build:
    just build deb
    flatpak-builder --force-clean --user --install-deps-from=flathub --repo=./target/flatpak-repo --install ./target/flatpak ./installer/flatpak/io.github.memospot.Memospot.yml

[linux]
flatpak-run:
    flatpak run io.github.memospot.Memospot

[doc('Debug environment variables of the running Memospot process')]
[linux]
[script]
debug-env:
    read -r pid _ <<< "$(pidof memospot 2>/dev/null || true)"
    [ -z "$pid" ] && echo -e "{{RED}}Memospot is not running.{{NORMAL}}" && exit 1
    tr '\0' '\n' < "/proc/$pid/environ" | sort

[private]
[script]
postbuild:
    set +e
    echo -e "{{CYAN}}Moving relevant build files to ./build directory…{{NORMAL}}"
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
        test -f "$resolved_path" && cp -f "$resolved_path" ./build/. 2>/dev/null
    done
    pushd "./build"
        appimages=($(find *.AppImage -type f 2>&1))
        for appimage in "${appimages[@]}"; do
            ! test -f "${appimage}" && continue
            ! test -d "${appimage}.home" && mkdir -p "${appimage}.home"
        done
    if ls ./memos* 1> /dev/null 2>&1; then
        echo -e "{{GREEN}}Done.{{NORMAL}}"
    else
        echo -e "{{RED}}Failed to move files.{{NORMAL}}"
    fi
    popd

[doc('Clean project artifacts. Use --deep or -d to clean cargo cache as well.')]
[script]
clean deep='':
    set +e
    bun pm cache rm || true
    dirs=(
        "./.dprint"
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
    if [ "{{ deep }}" = "--deep" ] || [ "{{ deep }}" = "-d" ]; then
        cargo cache -a || true
    fi
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
lint-rs:
    cargo fmt --all --check
    cargo clippy --all-features --all-targets --workspace --locked

[group('lint')]
[doc('Lint TypeScript code with BiomeJS')]
lint-ts:
    bun x @biomejs/biome ci --css-parse-tailwind-directives=true .

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
[script]
fix-ts:
    bun x @biomejs/biome lint --write "$REPO_ROOT/build-scripts" "$REPO_ROOT/src-ui"

[group('format')]
[doc('Format code with dprint (json, rust, toml, yaml, html, css, typescript and markdown).')]
fmt:
    dprint fmt

[doc('Run all recommended pre-commit checks')]
pre-commit: fmt lint test

[group('maintainer')]
[doc('Delete all GitHub build cache')]
gh-clean-cache:
    gh cache delete --all

# Delete all GitHub Actions workflow runs for the current repository.
[group('project maintainer')]
[script]
gh-purge-workflow-runs:
    repo=$(gh repo view --json nameWithOwner -q .nameWithOwner)
    run_count=$(gh api "repos/$repo/actions/runs" --paginate --jq '.workflow_runs[].id' | wc -l)
    echo "Repo: $repo"
    echo "Workflow runs found: $run_count"
    gh api "repos/$repo/actions/runs" --paginate --jq '.workflow_runs[].id' | while read -r id; do gh api -X DELETE "repos/$repo/actions/runs/$id" >/dev/null
    echo "deleted $id"; done && remaining=$(gh api "repos/$repo/actions/runs" --paginate --jq '.workflow_runs[].id' | wc -l)
    echo "Remaining runs: $remaining"
    echo "Done!"

[group('maintainer')]
[doc('Bump version in Cargo.toml and crates/memospot/Cargo.toml')]
[script]
bumpversion VERSION:
    clean="{{trim_start_match(VERSION, "v")}}"
    cargo set-version --locked "$clean"
    cargo generate-lockfile
    for d in "." "./build-scripts" "./src-ui"; do
        jq --arg version "$clean" '.version = $version' "$d/package.json" > "$d/package.json.tmp" && mv "$d/package.json.tmp" "$d/package.json"
    done
    just fmt
    bun install --lockfile-only
    git add \
        "./crates/memospot/gen/*" \
        ./crates/memospot/Cargo.toml \
        ./crates/memospot/Tauri.toml \
        ./Cargo.lock \
        ./Cargo.toml \
        ./build-scripts/package.json \
        ./src-ui/package.json \
        ./package.json \
        ./bun.lock
    git commit -m "chore: bump version to v$clean"

[group('maintainer')]
[doc('Push a new tag to the repository')]
[script]
pushtag TAG:
    clean="{{trim_start_match(TAG, "v")}}"
    git tag -a "v$clean" -m "chore: push v$clean"
    git push origin --tags

[group('maintainer')]
[doc('Publish a new version (bumpversion + pushtag)')]
publish TAG:
    just bumpversion {{TAG}}
    just pushtag {{TAG}}
