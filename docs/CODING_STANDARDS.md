# Coding Standards

Canonical coding standards for AI agents working on this codebase.

- Apply on every code-writing and code-review task.
- Keep project-specific rules, tooling, paths, and structure in `AGENTS.md`.
- Each rule below changes a decision, execution order, tool use, error response, or completion judgment.

## General

### Modules and interfaces

- Give one clear responsibility per module. Entry points handle wiring and startup flow only.
- Place logic in its domain module. Do not create or expand catch-all `utils` or `helpers` modules.
- Separate pure domain logic from I/O and platform integration. Make dependencies explicit through signatures.
- Accept dependencies, do not create them. Pass I/O, config, and environment-derived values through the interface. Never instantiate subprocess, filesystem, or network clients inside domain logic.
- Return results, do not mutate to communicate. Scope all mutation narrowly.
- Split on diverging responsibilities. Extract an exported helper only when reused or independently meaningful through a public contract.
- Design for depth: much behavior behind a small interface. Ask fewer methods, simpler params, more hidden complexity. Apply the deletion test: if deleted complexity vanishes it was pass-through; if complexity reappears across callers it earns its keep.
- Treat seam placement as distinct from what sits behind it. Tier-spanning slices keep pure domain logic separate from I/O.
- Count a test adapter as a second adapter. One adapter is hypothetical, two real, so explicit dependencies at the system seam are real seams. Never expose internal seams through the external interface.
- Treat Interface as everything a caller must know: signature plus invariants, ordering, error modes, config, and performance. Do not narrow it to a type-level signature or a language keyword.
- Treat the interface as the test surface. Callers and tests cross the same seam. Do not reach past the interface to test internals.

### Error handling and state

- Add context when propagating errors. No silent failures.
- Catch at architectural seams, not randomly. Fail fast on unrecoverable paths.
- Emit warnings for recoverable errors and descriptive errors for fatal failures.
- Use a Unicode ellipsis (`…`), not three dots, in logs and status output.
- Prefer immutable data. Scope mutability narrowly and document stop or lock invariants when non-obvious.

### Code style

- Prefer inferred types over annotations where the language allows it. Annotate where inference fails or the contract is unclear.
- Name descriptively over abbreviations, especially exported items. Allow abbreviations only for short-lived variables in scopes of five lines or fewer (single letter preferred) and well-known acronyms.
- Carry meaning with qualifiers: big-endian for non-booleans (`latency_ms_max`, new files sorted by most significant word), little-endian for booleans (`is_enabled`). Prefer `api: Fetcher` over `fetcher: Fetcher`. Follow latest language conventions and fall back to nearby code on ambiguity.
- State invariants positively. Prefer functional style for small iterator logic and imperative style on hot paths. Keep functions focused.
- Prefer early returns or ternaries over reassignment, `else` blocks, and nested `if`s. Collapse nested `if`s with logical conjunction.
- Pass critical options explicitly at call sites.
- Treat `index`, `count`, and `size` as distinct types.
- Do not add exported single-use helpers under about ten lines to shorten a function. Private helpers inside the module implementation are allowed when they do not widen the interface and are never tested directly.
- Prefer correctness and robustness over short-term convenience. Roll back safely and clean up without leaving inconsistent state. Do not circumvent linters or type checkers by default. When a rule is truly not practical, add a scoped ignore in the linter config with a why-comment.

### Comments

- Leave surrounding comments unchanged unless directly related to the task.
- Omit new comments by default. Explain intent, not steps. Use inline comments only for non-obvious logic. Never address the user in code comments.
- Write doc comments for exported contracts where the caller must know invariants, ordering, or error modes.

### Tests

- Prefer deep interface and integration coverage through public seams; keep unit tests only for pure domain logic that needs no I/O.
- Add only high-value tests that prevent regressions or validate correctness. Cover success, failure, and practical (never theoretical) edge cases for new behavior.
- Add no tests for statically defined values and no negative tests for removed logic.
- Test public contracts and behavior only with deep equality over whole objects. Do not test private helpers directly and do not duplicate implementation logic in the test.
- Never change tests to match a wrong implementation. Double-check implementation and expected behavior first and grill the user when expected behavior is unclear. Never weaken existing assertions unless expected behavior genuinely changed and the reason is clear.
- Keep tests deterministic: no wall-clock, network, or environment coupling unless the behavior under test requires it, so they pass locally and on CI.
- Inject environment-derived values as arguments. Do not mutate process environment in tests. Avoid mocks generally. Mock only at the system seam (subprocess, filesystem, network), never domain logic.
- On deepening, replace tests instead of layering. Delete superseded shallow unit tests once deep interface coverage exists, without weakening observable-behavior coverage.
- Use BDD comments (`// GIVEN` / `// WHEN` / `// THEN` in Rust/TS, with the language's comment marker elsewhere, plus `AND` as needed) with a high-signal description per test. Split each test into logical blocks when it improves readability without becoming redundant.

## Rust

- Document newly added traits with the role and how implementations use them.
- Keep modules under about 500 lines excluding tests. Above about 800 lines, add new functionality in a new module and move related tests and module or type docs toward the new implementation.
- Express lifetimes with ownership and borrowing while minimizing clones. Prefer `&T` when the function does not consume the value.
- Let clone-then-move carry a trailing underscore. Prefer cloning outside over cloning inside the new context: clone outside as `let <name>_ = <name>.clone();` and move `<name>_` inside `async move`, `move` closures, `spawn` or `block_on` bodies, and consuming functions. Do not shadow, append `_clone`, or use numeric suffixes.
- Ignore intentionally with `let _ =`. Allow `.ok()` for best-effort cleanup and `Option` chains.
- Prefer method references over closures and inline format args (`format!("{var}")`).
- Never ship `todo!` or `dbg!`. Remove them before linting.
- Prefer `assert_eq!()` on entire objects, not individual fields.

## TypeScript

- Do not use `any` for TypeScript types.
- Keep linter and formatter separate. Never run a second formatter beside the project formatter.
- Never hand-edit generated code: change the source and regenerate.
- When a linter rule is broken for framework components, disable it scoped with a why-comment instead of loosening the whole config.
