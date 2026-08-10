# AGENTS.md

Canonical instructions for AI assistants working on this codebase.

Tauri v2 desktop app bundling [Memos](https://github.com/usememos/memos) as a sidecar binary.

## Operating Rules

- Do NOT run builds directly. If a build is needed, instruct the user briefly on how to do it.
- Do NOT default to circumventing linters or type checkers. If you are confident a lint ignore is best, add a scoped ignore in `Cargo.toml`/`biome.jsonc` with a comment explaining why.
- Prefer correctness and robustness over short-term convenience. Never leave the system in an inconsistent state or risk data loss.
- Do not create small helper methods that are referenced only once.
- Deliver code changes only after `just gate` succeeds.
- Do not add general product or user-facing documentation to the docs/ folder — `docs/` only redirects to the separate `memospot.github.io` repository.
- Newly added traits should include doc comments that explain their role and how implementations are expected to use them.
- Avoid large modules: target Rust modules under 500 LoC (excluding tests). If a file exceeds ~800 LoC, add new functionality in a new module instead. When extracting, move related tests and module/type docs toward the new implementation.
- When running Rust commands (e.g. `just test`) be patient and never kill them via PID. Rust lock contention can slow execution — this is expected.
- Never create GitHub issues or pull requests. This project only accepts manual human-curated contributions. If asked, inform and stop.

## Test assertions

- Do not add tests for values that are statically defined.
- Do not add negative tests for logic that was removed.
- Prefer deep equals comparisons. Perform `assert_eq!()` on entire objects, not individual fields.
- Avoid mutating process environment in tests; prefer passing environment-derived flags or dependencies from above.

## Commands

Everything runs through `just`. Do NOT run raw `cargo`/`npm`/`bun` commands for build/test/lint — use just recipes.

| Task                              | Command                                                                                                   |
| --------------------------------- | --------------------------------------------------------------------------------------------------------- |
| Concise agent-oriented validation | `just gate`                                                                                               |
| Full validation (lint → test)     | `just validate`                                                                                           |
| Dev mode                          | `just dev`                                                                                                |
| Build                             | `just build` / `just build no-bundle` (binary only)                                                       |
| Run all tests                     | `just test`                                                                                               |
| TS tests only                     | `just test-ts`                                                                                            |
| Rust tests only                   | `just test-rs` (workspace lib), `just test-tauri` (memospot only), `just test-crates` (excludes memospot) |
| Lint all                          | `just lint` (dprint-check → biome-ci → cargo-fmt-check → clippy)                                          |
| Fix                               | `just fix` (TS safe fixes + `cargo fix`)                                                                  |
| Format all                        | `just fmt`                                                                                                |
| Pre-commit                        | `just pre-commit` (fmt → lint → test — in that order)                                                     |
| Clean                             | `just clean` (add `--deep` for cargo cache)                                                               |
| Single Svelte typecheck           | `(cd src-ui && bun check)`                                                                                |

## Structure

- Rust workspace `crates/*`: `memospot` is the Tauri app; `config`, `dialog`, `homedir`, `migration`, `portpicker`, `sidecar`, `writable` are local path-dependency crates; `build_utils` is build-only.
- `src-ui/` — SvelteKit frontend (`@memospot/src-ui` bun workspace package).
- `build-scripts/` — bun/TS build tooling; bin scripts live in `build-scripts/bin/` (e.g. `download-memos.ts`, `is-stale.ts`).
- `server-dist/` — downloaded Memos server binaries (gitignored), wired into the app as Tauri `externalBin` sidecar; per-target names like `memos-x86_64-unknown-linux-gnu`.
- `openspec/specs/` — committed feature specs (spec-driven workflow); read the relevant spec before implementing feature work.
- `memos-server-updater.sh` / `memos-server-updater.ps1` — user-facing scripts that replace the bundled Memos server binary.

## Formatting & Linting

**dprint is the ONE formatter.** It orchestrates everything: `.rs` → rustfmt, `.js/.ts/.json/.jsonc` → dprint-plugin-biome (NOT prettier), `.css` → malva, `.html/.svelte` → markup_fmt, `.md` → markdown, `.toml` → toml, `.yml` → pretty_yaml.

**Biome is linter-only for TS/JS** (formatter disabled in `biome.jsonc`). `just lint-ts` runs `bun x @biomejs/biome ci --css-parse-tailwind-directives=true .`, not `biome check`.

For `.svelte` files, biome disables `useConst`, `useImportType`, `noUnusedVariables`, `noUnusedImports`, `noUndeclaredVariables` — these rules are broken for Svelte components and are intentionally off.

**dprint `indentWidth: 4` is global**, except explicit overrides for yaml/json (2), html/svelte (2). Line width is 96 everywhere.

Never run `biome format` or `prettier` in this repo.

## Rust Workspace

- Edition **2024**, MSRV **1.88.0**, toolchain `stable` per `rust-toolchain.toml`.
- `cargo clippy --all-features --all-targets --workspace --locked` (just recipe always passes `--locked`).
- `RUST_BACKTRACE=full` is set globally in the justfile.
- `test-rs` exports `CARGO_PROFILE_TEST_BUILD_OVERRIDE_DEBUG=true` so test builds carry debug info; `test-crates` excludes the `memospot` package.
- The `unittest` feature in `crates/memospot` gates `build.rs` code that should only run during test/lint (not during normal builds).
- The `memospot` crate dev-depends on itself with the `unittest` feature. This is intentional.
- Release profile: `panic = "abort"`, `opt-level = "s"`, `lto = "thin"`, `codegen-units = 1`.
- Pinned `native-dialog = "0.6.4"` — newer versions broken on KDE.
- Workspace `publish = false` in root `Cargo.toml`.

## TypeScript Bindings

TypeScript types are generated from Rust structs via `ts-rs`. Output goes to `src-ui/src/lib/types/gen/` and IS committed to the repo — commit regenerated bindings together with the Rust changes that produced them.

- Regenerate: `just gen-bindings` (runs `cargo test export_bindings` inside `crates/memospot`), or `cargo test export_bindings` with the justfile's `TS_RS_EXPORT_DIR` env var set. Without it, ts-rs writes to the default, gitignored `crates/memospot/bindings/` instead of `src-ui/src/lib/types/gen/`.
- `gen-bindings` only regenerates when it detects git changes in `crates/config/**/*` or `crates/memospot/src/runtime_config.rs`.

## SvelteKit Frontend

- **Run `bunx --bun svelte-kit sync` before typechecking/linting Svelte files.** The `dev` and `check` scripts run it first; `postinstall` does not (bunfig.toml sets `ignoreScripts = true`).
- Dev server runs on port **1420** (hardcoded in `Tauri.toml` and `src-ui/vite.config.ts`).
- Svelte 5 with runes mode (`runes: true` in `svelte.config.js`).
- `adapter-static`, no SSR, fully prerendered.
- Tailwind CSS v4 via the `@tailwindcss/vite` plugin in `src-ui/vite.config.ts`, not the CLI.
- Uses `lucide-svelte`, `bits-ui`, `mode-watcher`, `svelte-radix`, `svelte-sonner` for UI components.
- See `src-ui/AGENTS.md` for Svelte MCP server instructions when editing `.svelte` files.

## i18n

- **Frontend**: `@inlang/paraglide-js`; message sources are `src-ui/i18n/{locale}.json`, generated code goes to `src-ui/src/lib/paraglide/` (gitignored, rebuilt automatically by the vite plugin during dev/build).
- **Rust**: `i18n-embed` with Fluent system, translations in `crates/memospot/i18n/{locale}/memospot.ftl`.

## Environment & Build Caching

- `build-ui` recipe uses stamp-file caching (`.build-stamps/`). If you change `src-ui/**` or `bun.lock`, the UI rebuilds; otherwise it skips.
- `gen-icons` similarly caches via `is-stale` tool.
- Bun install uses `exact = true` and `minimumReleaseAge = 604800` (7 days, in `bunfig.toml`) — new packages must be pinned exactly and published ≥7 days ago.
- Docker builds use `docker buildx bake` via the `just bake` recipe. Signing keys are sealed as docker secrets, never passed as build args.
- The justfile auto-loads `.env` (`set dotenv-load := true`); the updater signing key also falls back to `~/.tauri/memospot_updater.key`.

## Known Gotchas

- Dev mode (`just dev`) auto-kills leftover `memospot`/`memos` processes on start and exit.
- The root `package.json` has no scripts: build tooling runs as `bun run ./build-scripts/bin/<file>.ts` from the root; frontend scripts run as `bun --cwd=src-ui run <script>`.
- `just bumpversion` updates version in both `Cargo.toml` files AND all three `package.json` files (root, build-scripts, src-ui).

## Additional Sources

- `src-ui/AGENTS.md` — Svelte MCP server usage instructions.
- `CONTRIBUTING.md` — redirects to external docs at memospot.github.io.

## Platform Support

Tests and features must support Linux, macOS and Windows unless feature is explicitly OS-specific.
