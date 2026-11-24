# AGENTS.md

Canonical instructions for AI assistants working on this codebase.

## Overview

Tauri v2 desktop app bundling [Memos](https://github.com/usememos/memos) as a sidecar binary.

## Sources

- `openspec/specs/**/spec.md` — intended behavior. Discover with `openspec list --specs`.
- `justfile` — normative for workflow commands.
- `docs/SECURITY.md` — Security policy when reporting vulnerabilities.

When sources disagree: specs for intent, code/config for current behavior, this file for conventions.

## Specs & Domain Language

- Specs own observable _what_ + _why_; code owns _how_. Mechanics in `openspec/config.yaml: rules.specs`.
- One owning capability per behavior. When code changes behavior, update the owning spec in the same change; limit cleanup to touched capabilities.
- Terms follow `docs/GLOSSARY.md` (overrules default `domain-modeling` skill root `GLOSSARY.md`; domain-only, never general programming concepts). Architecture terms (`module`, `interface`, `seam`, `adapter`, `depth`, `leverage`, `locality`) follow `codebase-design` skill and never enter `docs/GLOSSARY.md`; durable _why_ goes to `docs/adr/` only when hard to reverse, surprising, and traded off. This triple gate overrules any skill suggestion to record more. If `docs/GLOSSARY.md` or `docs/adr/` does not yet exist, create lazily on the first resolved term or ADR per `domain-modeling` and do not block work on their absence.
- Never weaken a spec to match a bad implementation. Push back and grill the user when specs cannot be reconciled without degrading safety, correctness, or predictability.
- Spec change is done only when `openspec validate --specs` passes and every touched requirement passes the refactor test in `openspec/config.yaml` (`rules.specs`).

## Rules

- Do NOT run builds directly. If a build is needed, instruct the user briefly on how to do it.
- Deliver code changes only after `just gate` succeeds. When changing markdown files (docs or specs), run `dprint check` instead of defaulting to `just gate`; if flagged, run `dprint fmt` and re-check.
- Do NOT add general product or user-facing documentation to the docs/ folder — `docs/` holds only redirects to the separate `memospot.github.io` repository plus `docs/CODING_STANDARDS.md`, `docs/SECURITY.md`, and future `docs/GLOSSARY.md`/`docs/adr/` when created per the Specs gate.
- When running long commands (e.g. `just test`) be patient and never kill them via PID. Rust lock contention can slow execution — this is expected.
- Tests and features must support Linux, macOS, and Windows unless the feature is explicitly OS-specific.
- Prefer unit tests close to code and integration tests under `tests/`.

## Coding Standards

**MUST** read `docs/CODING_STANDARDS.md` **before writing or editing any code** for any code-writing or code-review task: General for all code, plus the Rust or TypeScript section for the touched language, and cite the applicable sections in your summary. If that file is missing or unreadable, continue with this file alone and flag the missing reference.

## Contributing

Governs agent-driven GitHub contributions on the user's behalf. No `.github/` templates; `docs/CONTRIBUTING.md` is a redirect stub only, do not add a substantive in-repo guide.

- Communication style (contributions, issues, and PRs only): plain high-signal English, no emojis. Clear, brief, consistent. No filler, sycophancy, canned phrases, generic conclusions, semantic repetition, or telegraphic prose. Complete sentences by default; headings, lists, or tables when clearer; never force complex material into one paragraph. Vary rhythm with short and longer sentences. Target concise responses (<200 lines when possible); never sacrifice formatting, evidence, or material reporting to hit it. Completeness wins over brevity.
- Never create an issue or PR unless the user directly requests that specific issue/PR in this session. Inferred intent ("this is broken", "this should be a PR") is not permission. This gates `gh issue/pr create`, API, and UI paths alike.
- Issues: state the problem simply, then the suggested solution briefly if any. Omit exhaustive implementation detail unless materially relevant. Upload screenshots/evidence to GitHub.
- PRs: subject `type(scope): imperative summary`, no longer than 72 characters, short lowercase scope; state why the change matters over how. Body: problem in a sentence or two, then the fix. Compress positive checks; omit inventories already covered by the diff. Report material outcomes, decisions, failures, caveats, assumptions, limits, validation, risks, uncertainties, privacy/safety issues, and blockers. UI changes need before/after images. Upload all PR evidence to GitHub; never commit PR-only screenshots or assets. Split PRs solving multiple independent problems. Adjacent cleanup, refactor, or second fix needs its own PR unless required for the same problem. No broad dependency bumps; bump only what a bug fix or feature requires.

## Commands

Everything runs through `just`. Do NOT run raw `cargo`/`npm`/`bun` commands for build/test/lint — use just recipes. `justfile` is normative; run bare `just` for the full list.

## Structure

- Rust workspace `crates/*`: `memospot` is the Tauri app; `config`, `dialog`, `homedir`, `migration`, `portpicker`, `sidecar`, `writable` are local path-dependency crates; `memospot-build-internals` contains project-specific build and workspace helpers.
- `src-ui/` — SvelteKit frontend (`@memospot/src-ui` bun workspace package).
- `build-scripts/` — bun/TS build tooling; bin scripts live in `build-scripts/bin/` (e.g. `download-memos.ts`, `is-stale.ts`).
- `server-dist/` — downloaded Memos server binaries (gitignored), wired into the app as Tauri `externalBin` sidecar; per-target names like `memos-x86_64-unknown-linux-gnu`.
- `memos-server-updater.sh` / `memos-server-updater.ps1` — user-facing scripts that replace the bundled Memos server binary.

## Rust

### Rust Workspace

- `todo!` and `dbg!` macros fail clippy (`todo = "deny"`, `dbg_macro = "deny"` in root `Cargo.toml`); remove them before `just lint`.
- CI fails on unused deps via `cargo machete` (`.github/workflows/check-rust.yml`); record exceptions in `crates/memospot/Cargo.toml` (`[package.metadata.cargo-machete]`).
- The `unittest` feature in `crates/memospot` gates `build.rs` code that should only run during test/lint (not during normal builds). The `memospot` crate dev-depends on itself with that feature intentionally.
- Pinned `native-dialog = "0.6.4"` — newer versions broken on KDE.
- i18n: `i18n-embed` with Fluent system, translations in `crates/memospot/i18n/{locale}/memospot.ftl`.

Toolchain and profile values live in config; change the config, not this file.

## TypeScript & Frontend

- Biome is linter-only for TS/JS (formatter disabled in `biome.jsonc`); run via `just lint-ts`.
- For `.svelte` files, biome disables `useConst`, `useImportType`, `noUnusedVariables`, `noUnusedImports`, `noUndeclaredVariables` — these rules are broken for Svelte components and are intentionally off.
- i18n: `@inlang/paraglide-js`; message sources are `src-ui/i18n/{locale}.json`, generated code goes to `src-ui/src/lib/paraglide/` (gitignored, rebuilt automatically by the vite plugin during dev/build).

### TypeScript Bindings

- TypeScript types are generated from Rust structs via `ts-rs`. Output goes to `src-ui/src/lib/types/gen/` and IS committed to the repo — commit regenerated bindings together with the Rust changes that produced them. Do NOT hand-edit `gen/*`; change the Rust source and regenerate instead.
- Do NOT hand-edit `src-ui/src/lib/paraglide/`; it is gitignored and rebuilt by the vite paraglide plugin.
- Regenerate: `just gen-bindings` (runs `cargo test export_bindings` inside `crates/memospot`), or `cargo test export_bindings` with the justfile's `TS_RS_EXPORT_DIR` env var set. Without it, ts-rs writes to the default, gitignored `crates/memospot/bindings/` instead of `src-ui/src/lib/types/gen/`.
- `gen-bindings` regenerates when it detects git changes under `crates/config` or `crates/memospot/src/runtime_config.rs`. When git metadata is unavailable, it regenerates and reports the condition.

### SvelteKit Frontend

- **Run `bunx --bun svelte-kit sync` before typechecking/linting Svelte files.** The `dev` and `check` scripts run it first; `postinstall` does not (bunfig.toml sets `ignoreScripts = true`).
- Dev server runs on port **1420** (hardcoded in `Tauri.toml` and `src-ui/vite.config.ts`).
- Svelte 5 with runes mode auto-detected per file (no global `runes` compiler option; a global `true` would force legacy `bits-ui` components into runes mode and break the build).
- `adapter-static`, no SSR, fully prerendered.
- Tailwind CSS v4 via the `@tailwindcss/vite` plugin in `src-ui/vite.config.ts`, not the CLI.
- Uses `lucide-svelte`, `bits-ui`, `mode-watcher`, `svelte-radix`, `svelte-sonner` for UI components.
- See `src-ui/AGENTS.md` for Svelte MCP server instructions when editing `.svelte` files.

## Formatting & Linting

- dprint is the ONE formatter (config normative). Never run `biome format` or `prettier` in this repo.

## Environment & Build Caching

- `build-ui` and `gen-icons` use stamp-file caching; unchanged UI/icons skip rebuild.
- Bun install uses `exact = true` and `minimumReleaseAge = 604800` (7 days, in `bunfig.toml`) — new packages must be pinned exactly and published ≥7 days ago.
- Docker signing keys are sealed as docker secrets, never passed as build args; see `just bake`. The updater key falls back to `~/.tauri/memospot_updater.key`; see the justfile.

## Known Gotchas

- Dev mode (`just dev`) auto-kills leftover `memospot`/`memos` processes on start and exit.
- The root `package.json` exposes only the `tauri` script; build tooling runs as `bun run ./build-scripts/bin/<file>.ts` from the root; frontend scripts run as `bun --cwd=src-ui run <script>`.
- `just bumpversion` updates `Cargo.toml` and `package.json` versions; see the recipe.
